use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

use crate::models::{DeviceProfile, Profile, WsMessage};
use crate::ws::ConnectionManager;

pub const POT_PING_INTERVAL: Duration = Duration::from_secs(10);
/// Safety net for clients that disappear without closing the socket.
pub const POT_STALE_TIMEOUT: Duration = Duration::from_secs(90);
const BIND_PORT_RANGE: std::ops::RangeInclusive<u16> = 8080..=8086;

/// Shared profile store. Every entry is tagged with the source(s) that told
/// us about it, so removals can be attributed precisely when a source goes
/// away. The hub is the only writer; server/peer tasks read for broadcasts.
pub type Store = Arc<Mutex<HashMap<String, StoreEntry>>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    Local,
    Client(u64),
    Peer(String),
}

#[derive(Debug, Clone)]
pub struct StoreEntry {
    pub profile: Profile,
    pub sources: HashSet<Source>,
    pub last_seen: Instant,
}

pub enum ServerCmd {
    Broadcast,
}

/// Runs the pot server until told to shut down.
///
/// Every connected client gets a `hello` + full-store snapshot on connect,
/// then live broadcasts whenever the store changes. Single-profile messages
/// (ESP32 style) and full-store `profiles` messages are both accepted from
/// clients.
pub async fn run_pot_server(
    hub: Arc<ConnectionManager>,
    store: Store,
    own_device_id: String,
    mut server_rx: mpsc::Receiver<ServerCmd>,
    port_tx: tokio::sync::oneshot::Sender<u16>,
) {
    let Some((listener, port)) = bind_first_available().await else {
        eprintln!("[pot] could not bind any port in {BIND_PORT_RANGE:?}");
        let _ = port_tx.send(0);
        return;
    };
    let _ = port_tx.send(port);
    eprintln!("[pot] hosting pot on 0.0.0.0:{port}");

    let conns: Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut next_id: u64 = 1;

    let mut ping_tick = tokio::time::interval(POT_PING_INTERVAL);
    ping_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            cmd = server_rx.recv() => match cmd {
                Some(ServerCmd::Broadcast) => broadcast(&conns, &store),
                None => break,
            },
            _ = ping_tick.tick() => {
                ping_clients(&conns);
                prune_stale(&store, &hub);
            }
            res = listener.accept() => {
                if let Ok((tcp, _)) = res {
                    let id = next_id;
                    next_id += 1;
                    let (client_tx, client_rx) = mpsc::channel::<Message>(16);
                    conns.lock().unwrap().insert(id, client_tx);
                    let hub = hub.clone();
                    let store = store.clone();
                    let own = own_device_id.clone();
                    let conns = conns.clone();
                    tauri::async_runtime::spawn(async move {
                        client_task(tcp, hub, store, own, id, client_rx, conns).await;
                    });
                }
            }
        }
    }
}

async fn bind_first_available() -> Option<(TcpListener, u16)> {
    for port in BIND_PORT_RANGE {
        if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)).await {
            return Some((listener, port));
        }
    }
    None
}

async fn client_task(
    tcp: TcpStream,
    hub: Arc<ConnectionManager>,
    store: Store,
    own_device_id: String,
    conn_id: u64,
    mut client_rx: mpsc::Receiver<Message>,
    conns: Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>>,
) {
    let ws: WebSocketStream<TcpStream> = match accept_async(tcp).await {
        Ok(ws) => ws,
        Err(err) => {
            eprintln!("[pot] handshake failed: {err}");
            hub.on_client_left(conn_id);
            return;
        }
    };
    eprintln!("[pot] client {conn_id} connected");
    let (mut write, mut read) = ws.split();

    // Greet with our identity, then a snapshot of the whole pot.
    let hello = WsMessage {
        msg_type: "hello".into(),
        device_id: Some(own_device_id.clone()),
        data: vec![],
    };
    let mut fail = write
        .send(Message::Text(serde_json::to_string(&hello).unwrap().into()))
        .await
        .is_err();
    if !fail {
        let snapshot = snapshot_message(&store);
        fail = write.send(Message::Text(snapshot.into())).await.is_err();
    }
    if fail {
        hub.on_client_left(conn_id);
        return;
    }

    loop {
        tokio::select! {
            msg = client_rx.recv() => match msg {
                Some(msg) => {
                    if write.send(msg).await.is_err() {
                        break;
                    }
                }
                None => break,
            },
            msg = read.next() => match msg {
                Some(Ok(Message::Text(text))) => handle_client_text(&hub, conn_id, &text),
                // Pings are answered automatically by tungstenite.
                Some(Ok(_)) => {}
                _ => break,
            },
        }
    }
    // Unregister so the server stops broadcasting to a dead socket.
    conns.lock().unwrap().remove(&conn_id);
    eprintln!("[pot] client {conn_id} disconnected");
    hub.on_client_left(conn_id);
}

fn handle_client_text(hub: &ConnectionManager, conn_id: u64, text: &str) {
    // Full-store sync message: { "type": "profiles", "data": [...] }
    if let Ok(msg) = serde_json::from_str::<WsMessage>(text) {
        if msg.msg_type == "profiles" {
            hub.on_client_profiles(conn_id, msg.data, true);
            return;
        }
    }
    // Single profile (ESP32 style): { "device_id", "name", "role", "bio" }
    if let Ok(profile) = serde_json::from_str::<DeviceProfile>(text) {
        hub.on_client_profiles(conn_id, vec![profile], false);
    }
}

fn snapshot_message(store: &Store) -> String {
    let profiles: Vec<DeviceProfile> = store
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

fn broadcast(conns: &Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>>, store: &Store) {
    let msg = snapshot_message(store);
    let conns = conns.lock().unwrap();
    for tx in conns.values() {
        let _ = tx.try_send(Message::Text(msg.clone().into()));
    }
}

fn ping_clients(conns: &Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>>) {
    let conns = conns.lock().unwrap();
    for tx in conns.values() {
        let _ = tx.try_send(Message::Ping(vec![]));
    }
}

/// Removes entries that only a vanished client told us about.
fn prune_stale(store: &Store, hub: &ConnectionManager) {
    let mut changed = false;
    {
        let mut store = store.lock().unwrap();
        let mut empties = vec![];
        for (did, entry) in store.iter_mut() {
            let all_clients = !entry.sources.is_empty()
                && entry.sources.iter().all(|s| matches!(s, Source::Client(_)));
            if all_clients && entry.last_seen.elapsed() > POT_STALE_TIMEOUT {
                empties.push(did.clone());
                changed = true;
            }
        }
        for did in empties {
            store.remove(&did);
        }
    }
    if changed {
        hub.after_store_change(vec![]);
    }
}
