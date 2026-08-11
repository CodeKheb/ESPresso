use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use mdns_sd::ServiceDaemon;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;

use crate::db::Db;
use crate::mdns::{self, MdnsBrowser};
use crate::models::{
    ConnectionStatus, DeviceProfile, HostInfo, OutgoingProfile, Profile, WsMessage,
};
use crate::server::{self, Source, Store, StoreEntry, ServerCmd};

pub const EVT_STATUS: &str = "connection://status";
pub const EVT_PROFILES: &str = "profiles://updated";
pub const EVT_CONTACTS: &str = "contacts://updated";
pub const EVT_DEVICES: &str = "devices://updated";
pub const EVT_DISCOVERY: &str = "discovery://done";

const CONNECT_TIMEOUT: Duration = Duration::from_millis(2500);
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(2);
const RESCAN_INTERVAL: Duration = Duration::from_secs(15);
const ANNOUNCE_INTERVAL: Duration = Duration::from_secs(20);
const MAX_PEER_CONNECT_FAILS: u32 = 12;

pub enum Command {
    SendProfile(Profile),
    JoinHost(String, u16),
    Rescan,
}

pub enum PeerCmd {
    /// Push the full store (app peers — carries removals).
    Sync,
    /// Push a single profile (ESP32-style pots).
    SendProfile(Profile),
}

/// The pot hub. Every ESPresso instance hosts its own pot server (advertised
/// via mDNS) and joins every other pot discovered on the current network —
/// "the WiFi is the DNS". Profiles seen from any source converge into the
/// shared store, are persisted to SQLite, and are served/broadcast back out.
pub struct ConnectionManager {
    app: AppHandle,
    pub(crate) db: Db,
    status: Mutex<ConnectionStatus>,
    tx: Mutex<Option<mpsc::Sender<Command>>>,
    store: Store,
    /// Outgoing peer connections, keyed by `ip:port`.
    peers: Mutex<HashMap<String, mpsc::Sender<PeerCmd>>>,
    /// Peers that speak the full app protocol (sent a `hello`).
    app_peers: Mutex<HashSet<String>>,
    /// Peers currently trying to connect (dedupes respawns).
    connecting_peers: Mutex<HashSet<String>>,
    pub(crate) mdns: Arc<MdnsBrowser>,
    server_tx: Mutex<Option<mpsc::Sender<ServerCmd>>>,
    server_port: Mutex<u16>,
    hostname: Mutex<String>,
    instance: Mutex<String>,
    #[allow(dead_code)]
    advertise: Mutex<Option<ServiceDaemon>>,
}

impl ConnectionManager {
    pub fn new(app: AppHandle, db: Db) -> Self {
        Self {
            app,
            db,
            status: Mutex::new(ConnectionStatus::default()),
            tx: Mutex::new(None),
            store: Arc::new(Mutex::new(HashMap::new())),
            peers: Mutex::new(HashMap::new()),
            app_peers: Mutex::new(HashSet::new()),
            connecting_peers: Mutex::new(HashSet::new()),
            mdns: Arc::new(MdnsBrowser::new()),
            server_tx: Mutex::new(None),
            server_port: Mutex::new(0),
            hostname: Mutex::new(String::new()),
            instance: Mutex::new(String::new()),
            advertise: Mutex::new(None),
        }
    }

    pub fn status(&self) -> ConnectionStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn host_info(&self) -> HostInfo {
        let hostname = self.hostname.lock().unwrap().clone();
        let instance = self.instance.lock().unwrap().clone();
        let port = *self.server_port.lock().unwrap();
        HostInfo {
            hostname: hostname.trim_end_matches('.').to_string(),
            port,
            instance,
        }
    }

    pub fn send_command(&self, cmd: Command) {
        let guard = self.tx.lock().unwrap();
        let Some(tx) = guard.as_ref() else {
            eprintln!("[hub] command dropped: task not started");
            return;
        };
        if let Err(err) = tx.try_send(cmd) {
            eprintln!("[hub] command dropped: {err}");
        }
    }

    fn emit(&self, event: &str, payload: impl serde::Serialize + Clone) {
        let _ = self.app.emit(event, payload);
    }

    fn set_status(&self, state: &str, host: Option<String>, message: Option<String>) {
        let status = ConnectionStatus {
            state: state.into(),
            host,
            message,
        };
        *self.status.lock().unwrap() = status.clone();
        self.emit(EVT_STATUS, status);
    }

    // ── main loop ───────────────────────────────────────────────────────────

    pub async fn run(self: Arc<Self>) {
        let (tx, mut rx) = mpsc::channel::<Command>(32);
        *self.tx.lock().unwrap() = Some(tx);

        let device_id = self.db.get_or_create_device_id();
        let short: String = device_id.replace('-', "").chars().take(8).collect();
        let hostname = format!("espresso-{short}.local");
        let instance = format!("ESPresso-{short}");
        *self.hostname.lock().unwrap() = hostname.clone();
        *self.instance.lock().unwrap() = instance.clone();

        // Seed the pot with our saved profile (if any).
        if let Some(p) = self.db.get_my_profile(&device_id) {
            let mut store = self.store.lock().unwrap();
            store.entry(p.device_id.clone()).or_insert(StoreEntry {
                profile: p,
                sources: HashSet::from([Source::Local]),
                last_seen: Instant::now(),
            });
        }

        // 1) Host the pot server.
        let (port_tx, port_rx) = tokio::sync::oneshot::channel();
        let (server_tx, server_rx) = mpsc::channel::<ServerCmd>(16);
        *self.server_tx.lock().unwrap() = Some(server_tx);
        {
            let hub = self.clone();
            let store = self.store.clone();
            tauri::async_runtime::spawn(async move {
                server::run_pot_server(hub, store, device_id, server_rx, port_tx).await;
            });
        }
        let port = tokio::time::timeout(Duration::from_secs(5), port_rx)
            .await
            .map(|r| r.unwrap_or(0))
            .unwrap_or(0);
        *self.server_port.lock().unwrap() = port;

        // 2) Advertise our pot over mDNS.
        if port != 0 {
            if let Some(daemon) = mdns::advertise(&format!("{hostname}."), &instance, port) {
                *self.advertise.lock().unwrap() = Some(daemon);
            }
        }

        // 3) We are a pot — the app is live.
        self.set_status(
            "connected",
            Some(hostname.clone()),
            Some(if port != 0 {
                format!("Pot live on port {port}")
            } else {
                "Could not bind a pot port; running as a client only".into()
            }),
        );

        // 4) Command loop + periodic peer discovery.
        let mut scan = tokio::time::interval(RESCAN_INTERVAL);
        scan.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                cmd = rx.recv() => match cmd {
                    Some(Command::SendProfile(p)) => self.handle_send_profile(p),
                    Some(Command::JoinHost(host, port)) => self.join_host(&host, port),
                    Some(Command::Rescan) => self.rescan_peers().await,
                    None => break,
                },
                _ = scan.tick() => self.rescan_peers().await,
            }
        }
    }

    // ── store ───────────────────────────────────────────────────────────────

    /// Merges one profile under `source`. Returns the profile if the store
    /// changed (new entry or content update).
    fn store_merge(&self, p: Profile, source: Source) -> Option<Profile> {
        if p.device_id.trim().is_empty() {
            return None;
        }
        let mut store = self.store.lock().unwrap();
        match store.get_mut(&p.device_id) {
            Some(entry) => {
                entry.sources.insert(source);
                entry.last_seen = Instant::now();
                if entry.profile.name != p.name
                    || entry.profile.role != p.role
                    || entry.profile.bio != p.bio
                {
                    entry.profile = p.clone();
                    Some(p)
                } else {
                    None
                }
            }
            None => {
                store.insert(
                    p.device_id.clone(),
                    StoreEntry {
                        profile: p.clone(),
                        sources: HashSet::from([source]),
                        last_seen: Instant::now(),
                    },
                );
                Some(p)
            }
        }
    }

    /// Replaces everything `source` told us with `profiles` (used for
    /// full-store syncs). Returns changed profiles and whether anything was
    /// removed.
    fn store_replace(&self, source: &Source, profiles: Vec<DeviceProfile>) -> (Vec<Profile>, bool) {
        let mut changed: Vec<Profile> = Vec::new();
        let mut removed = false;
        let mut store = self.store.lock().unwrap();

        let mut empties = Vec::new();
        for (did, entry) in store.iter_mut() {
            if entry.sources.remove(source) {
                if entry.sources.is_empty() {
                    empties.push(did.clone());
                }
            }
        }
        for did in empties {
            store.remove(&did);
            removed = true;
        }

        for raw in profiles {
            let p = Profile::from(raw);
            if p.device_id.trim().is_empty() {
                continue;
            }
            match store.get_mut(&p.device_id) {
                Some(entry) => {
                    let content_changed = entry.profile.name != p.name
                        || entry.profile.role != p.role
                        || entry.profile.bio != p.bio;
                    entry.sources.insert(source.clone());
                    entry.last_seen = Instant::now();
                    if content_changed {
                        entry.profile = p.clone();
                        changed.push(p);
                    }
                }
                None => {
                    store.insert(
                        p.device_id.clone(),
                        StoreEntry {
                            profile: p.clone(),
                            sources: HashSet::from([source.clone()]),
                            last_seen: Instant::now(),
                        },
                    );
                    changed.push(p);
                }
            }
        }
        (changed, removed)
    }

    fn store_remove_source(&self, source: &Source) -> bool {
        let mut store = self.store.lock().unwrap();
        let mut changed = false;
        let mut empties = Vec::new();
        for (did, entry) in store.iter_mut() {
            if entry.sources.remove(source) {
                changed = true;
                if entry.sources.is_empty() {
                    empties.push(did.clone());
                }
            }
        }
        for did in empties {
            store.remove(&did);
        }
        changed
    }

    fn store_snapshot(&self) -> Vec<Profile> {
        let store = self.store.lock().unwrap();
        let mut list: Vec<Profile> = store.values().map(|e| e.profile.clone()).collect();
        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        list
    }

    fn store_message(&self) -> String {
        let profiles: Vec<DeviceProfile> = self
            .store
            .lock()
            .unwrap()
            .values()
            .map(|e| DeviceProfile {
                device_id: e.profile.device_id.clone(),
                name: e.profile.name.clone(),
                role: e.profile.role.clone(),
                bio: e.profile.bio.clone(),
            })
            .collect();
        let msg = WsMessage {
            msg_type: "profiles".into(),
            device_id: None,
            data: profiles,
        };
        serde_json::to_string(&msg).unwrap_or_else(|_| r#"{"type":"profiles","data":[]}"#.into())
    }

    /// Called after any store change: persist, notify the UI, broadcast from
    /// our pot server, and sync/announce to peers.
    pub(crate) fn after_store_change(&self, changed: Vec<Profile>) {
        for p in &changed {
            let _ = self.db.upsert_profile(p);
        }
        self.emit(EVT_PROFILES, self.store_snapshot());

        if let Some(tx) = self.server_tx.lock().unwrap().as_ref() {
            let _ = tx.try_send(ServerCmd::Broadcast);
        }

        let app_peers = self.app_peers.lock().unwrap().clone();
        let peers = self.peers.lock().unwrap();
        for (key, tx) in peers.iter() {
            if app_peers.contains(key) {
                let _ = tx.try_send(PeerCmd::Sync);
            } else {
                for p in &changed {
                    let _ = tx.try_send(PeerCmd::SendProfile(p.clone()));
                }
            }
        }
    }

    // ── hub ← server bridge ─────────────────────────────────────────────────

    pub(crate) fn on_client_profiles(&self, conn_id: u64, profiles: Vec<DeviceProfile>, full: bool) {
        let changed = if full {
            let (changed, removed) = self.store_replace(&Source::Client(conn_id), profiles);
            if changed.is_empty() && !removed {
                return;
            }
            changed
        } else {
            let mut changed = Vec::new();
            for p in profiles {
                if let Some(profile) = self.store_merge(Profile::from(p), Source::Client(conn_id)) {
                    changed.push(profile);
                }
            }
            if changed.is_empty() {
                return;
            }
            changed
        };
        self.after_store_change(changed);
    }

    pub(crate) fn on_client_left(&self, conn_id: u64) {
        if self.store_remove_source(&Source::Client(conn_id)) {
            self.after_store_change(vec![]);
        }
    }

    // ── hub ← peer bridge ───────────────────────────────────────────────────

    pub(crate) fn on_peer_profiles(&self, key: &str, profiles: Vec<DeviceProfile>) {
        let (changed, removed) = self.store_replace(&Source::Peer(key.to_string()), profiles);
        if changed.is_empty() && !removed {
            return;
        }
        self.after_store_change(changed);
    }

    pub(crate) fn on_peer_left(&self, key: &str) {
        if self.store_remove_source(&Source::Peer(key.to_string())) {
            self.after_store_change(vec![]);
        }
    }

    // ── local profile ───────────────────────────────────────────────────────

    fn handle_send_profile(&self, p: Profile) {
        if p.device_id.trim().is_empty() {
            return;
        }
        let _ = self.db.upsert_profile(&p);
        if let Some(profile) = self.store_merge(p, Source::Local) {
            self.after_store_change(vec![profile]);
        }
    }

    // ── peers ───────────────────────────────────────────────────────────────

    fn join_host(self: &Arc<Self>, host: &str, port: u16) {
        let key = format!("{host}:{port}");
        self.spawn_peer_if_new(key, host.to_string(), port);
    }

    fn spawn_peer_if_new(self: &Arc<Self>, key: String, host: String, port: u16) {
        {
            let connecting = self.connecting_peers.lock().unwrap();
            if connecting.contains(&key) {
                return;
            }
            let peers = self.peers.lock().unwrap();
            if peers.contains_key(&key) {
                return;
            }
        }
        self.connecting_peers.lock().unwrap().insert(key.clone());
        let hub = self.clone();
        tauri::async_runtime::spawn(async move {
            hub.run_peer(key, host, port).await;
        });
    }

    async fn rescan_peers(self: &Arc<Self>) {
        let exclude = {
            let hostname = self.hostname.lock().unwrap().clone();
            let instance = self.instance.lock().unwrap().clone();
            vec![
                hostname.trim_end_matches('.').to_string(),
                instance.to_lowercase(),
            ]
        };
        let mdns = self.mdns.clone();
        let found = tauri::async_runtime::spawn_blocking(move || {
            mdns.browse(DISCOVERY_TIMEOUT, &exclude)
        })
        .await
        .unwrap_or_default();

        for d in found {
            let Some(ip) = d.ip else { continue };
            let key = format!("{ip}:{}", d.port);
            self.spawn_peer_if_new(key, ip, d.port);
        }
    }

    /// A single outgoing connection to one discovered pot. Reconnects with
    /// backoff whenever the connection drops, so a flaky link self-heals
    /// without waiting for the next mDNS scan.
    async fn run_peer(self: Arc<Self>, key: String, ip: String, port: u16) {
        let url = format!("ws://{ip}:{port}/ws");
        let mut backoff = 2u64;
        let mut fails = 0u32;

        // Claim the key for this task's whole lifetime so the rescan never
        // spawns a duplicate connection for the same pot.
        self.connecting_peers.lock().unwrap().insert(key.clone());

        'peer: loop {
            let (stream, _) = loop {
                match tokio::time::timeout(CONNECT_TIMEOUT, connect_async(&url)).await {
                    Ok(Ok(s)) => break s,
                    _ => {
                        fails += 1;
                        if fails >= MAX_PEER_CONNECT_FAILS {
                            eprintln!("[hub] giving up on {url} after {fails} failures");
                            break 'peer;
                        }
                        tokio::time::sleep(Duration::from_secs(backoff)).await;
                        backoff = (backoff * 2).min(30);
                    }
                }
            };
            eprintln!("[hub] joined pot {url}");
            fails = 0;
            backoff = 2;

            let (peer_tx, mut peer_rx) = mpsc::channel::<PeerCmd>(16);
            self.peers.lock().unwrap().insert(key.clone(), peer_tx);
            self.db.upsert_device("ESPresso pot", &ip, port, "auto");
            self.emit(EVT_DEVICES, self.db.get_devices());

            let (mut write, mut read) = stream.split();
            let mut announce = tokio::time::interval(ANNOUNCE_INTERVAL);
            announce.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            self.send_own_profile(&mut write).await;

            let mut app_peer = false;
            let mut reconnect = false;
            loop {
                tokio::select! {
                    cmd = peer_rx.recv() => match cmd {
                        Some(PeerCmd::Sync) if app_peer => self.send_store(&mut write).await,
                        Some(PeerCmd::SendProfile(p)) => {
                            let out = OutgoingProfile::from(p);
                            if let Ok(text) = serde_json::to_string(&out) {
                                if write.send(Message::Text(text.into())).await.is_err() {
                                    reconnect = true;
                                    break;
                                }
                            }
                        }
                        // Hub dropped our command channel (app shutting down).
                        None => break,
                        _ => {}
                    },
                    _ = announce.tick() => self.send_own_profile(&mut write).await,
                    msg = read.next() => match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                                match msg.msg_type.as_str() {
                                    "hello" => {
                                        app_peer = true;
                                        self.app_peers.lock().unwrap().insert(key.clone());
                                        self.send_store(&mut write).await;
                                    }
                                    "profiles" => self.on_peer_profiles(&key, msg.data),
                                    _ => {}
                                }
                            }
                        }
                        // Pings are answered automatically by tungstenite.
                        Some(Ok(_)) => {}
                        Some(Err(err)) => {
                            eprintln!("[hub] peer {url} read error: {err}");
                            reconnect = true;
                            break;
                        }
                        None => {
                            reconnect = true;
                            break;
                        }
                    },
                }
            }
            drop(write);
            drop(read);

            self.app_peers.lock().unwrap().remove(&key);
            self.peers.lock().unwrap().remove(&key);
            self.on_peer_left(&key);
            eprintln!("[hub] left pot {url}");

            if !reconnect {
                break;
            }
        }

        self.connecting_peers.lock().unwrap().remove(&key);
        eprintln!("[hub] peer task ended for {url}");
    }

    async fn send_own_profile(&self, write: &mut (impl futures_util::Sink<Message, Error = tungstenite::Error> + Unpin)) {
        let device_id = self.db.get_or_create_device_id();
        let Some(profile) = self.db.get_my_profile(&device_id) else { return };
        let out = OutgoingProfile::from(profile);
        if let Ok(text) = serde_json::to_string(&out) {
            let _ = write.send(Message::Text(text.into())).await;
        }
    }

    async fn send_store(&self, write: &mut (impl futures_util::Sink<Message, Error = tungstenite::Error> + Unpin)) {
        let msg = self.store_message();
        let _ = write.send(Message::Text(msg.into())).await;
    }
}
