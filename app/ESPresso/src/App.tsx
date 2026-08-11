import { useEffect, useState } from "react";
import "./App.css";
import "./screens/screens.css";
import type {
  Contact,
  ConnectionStatus,
  Device,
  DiscoveredDevice,
  HostInfo,
  Profile,
  Screen,
} from "./types";
import { ConnectingScreen } from "./screens/ConnectingScreen";
import { DisconnectedScreen } from "./screens/DisconnectedScreen";
import { CreateScreen } from "./screens/CreateScreen";
import { DashboardScreen } from "./screens/DashboardScreen";
import { ContactsScreen } from "./screens/ContactsScreen";
import { HistoryScreen } from "./screens/HistoryScreen";
import { DevicesScreen } from "./screens/DevicesScreen";
import {
  api,
  onContacts,
  onDevices,
  onDiscovery,
  onProfiles,
  onStatus,
} from "./lib/espresso";

function App() {
  const [status, setStatus] = useState<ConnectionStatus>({
    state: "connecting",
    host: null,
    message: null,
  });
  const [screen, setScreen] = useState<Screen>("create");
  /** Live profiles currently in the pot mesh (drives the dashboard). */
  const [liveProfiles, setLiveProfiles] = useState<Profile[]>([]);
  /** Everything ever seen, from SQLite (drives history). */
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [discovered, setDiscovered] = useState<DiscoveredDevice[]>([]);
  const [hostInfo, setHostInfo] = useState<HostInfo | null>(null);
  const [deviceId, setDeviceId] = useState("");
  const [name, setName] = useState("");
  const [role, setRole] = useState("");
  const [bio, setBio] = useState("");

  // Boot: load device identity + cached data, then subscribe to live events.
  // The pot mesh lives in the Rust backend, so this effect is idempotent even
  // under React StrictMode double-mounting.
  useEffect(() => {
    let active = true;
    const unlisteners: (() => void)[] = [];

    (async () => {
      try {
        const id = await api.getDeviceId();
        if (!active) return;
        setDeviceId(id);

        const [st, ps, cs, ds, hi] = await Promise.all([
          api.getStatus(),
          api.getProfiles(),
          api.getContacts(),
          api.getDevices(),
          api.getHostInfo(),
        ]);
        if (!active) return;
        setStatus(st);
        setLiveProfiles(ps);
        setProfiles(ps);
        setContacts(cs);
        setDevices(ds);
        setHostInfo(hi);
      } catch (err) {
        console.error("App init failed", err);
      }

      const subs = await Promise.all([
        onStatus((s) => {
          if (!active) return;
          setStatus(s);
          if (s.state !== "connected") {
            setLiveProfiles([]);
          }
        }),
        onProfiles((p) => {
          if (!active) return;
          setLiveProfiles(p);
          // Every live profile is persisted to SQLite, so refresh history.
          api
            .getProfiles()
            .then((all) => {
              if (active) setProfiles(all);
            })
            .catch((err) => console.error("Failed to refresh history", err));
        }),
        onContacts((c) => {
          if (active) setContacts(c);
        }),
        onDevices((d) => {
          if (active) setDevices(d);
        }),
        onDiscovery((d) => {
          if (active) setDiscovered(d);
        }),
      ]);
      if (!active) {
        subs.forEach((u) => u());
        return;
      }
      unlisteners.push(...subs);
    })();

    return () => {
      active = false;
      unlisteners.forEach((u) => u());
    };
  }, []);

  async function submitProfile() {
    if (!name || !role || !deviceId) return;
    const profile: Profile = { deviceId, name, role, bio };
    try {
      await api.sendProfile(profile);
    } catch (err) {
      console.error("Failed to send profile", err);
    }
    setScreen("dashboard");
  }

  async function handleAddContact(person: Profile) {
    try {
      setContacts(await api.addContact(person));
    } catch (err) {
      console.error("Failed to add contact", err);
    }
  }

  async function handleConnect(host: string) {
    try {
      await api.connectTo(host);
    } catch (err) {
      console.error("Failed to connect", err);
    }
  }

  async function handleAddDevice(host: string) {
    try {
      setDevices(await api.addDevice(host));
    } catch (err) {
      console.error("Failed to add device", err);
    }
  }

  async function handleRemoveDevice(id: number) {
    try {
      setDevices(await api.removeDevice(id));
    } catch (err) {
      console.error("Failed to remove device", err);
    }
  }

  async function handleScan() {
    try {
      setDiscovered(await api.discover());
    } catch (err) {
      console.error("Discovery failed", err);
    }
  }

  // Routing
  if (screen === "devices") {
    return (
      <DevicesScreen
        hostInfo={hostInfo}
        devices={devices}
        discovered={discovered}
        currentHost={status.host}
        onConnect={handleConnect}
        onAdd={handleAddDevice}
        onRemove={handleRemoveDevice}
        onScan={handleScan}
        onNavigate={setScreen}
      />
    );
  }
  if (status.state === "disconnected" || status.state === "error") {
    return (
      <DisconnectedScreen
        message={status.message}
        onRetry={() => api.retry()}
        onChooseDevice={() => setScreen("devices")}
      />
    );
  }
  if (status.state === "connecting") {
    return <ConnectingScreen message={status.message} />;
  }

  if (screen === "contacts") {
    return (
      <ContactsScreen
        contacts={contacts}
        onNavigate={setScreen}
        onSettings={() => setScreen("devices")}
      />
    );
  }
  if (screen === "history") {
    return (
      <HistoryScreen
        profiles={profiles}
        deviceId={deviceId}
        onNavigate={setScreen}
        onSettings={() => setScreen("devices")}
      />
    );
  }
  if (screen === "create") {
    return (
      <CreateScreen
        name={name}
        role={role}
        bio={bio}
        onNameChange={setName}
        onRoleChange={setRole}
        onBioChange={setBio}
        onSubmit={submitProfile}
        onSettings={() => setScreen("devices")}
      />
    );
  }

  const savedNames = new Set(contacts.map((c) => c.name));
  return (
    <DashboardScreen
      profiles={liveProfiles}
      deviceHost={hostInfo?.hostname ?? status.host}
      savedNames={savedNames}
      onAddContact={handleAddContact}
      onNavigate={setScreen}
      onSettings={() => setScreen("devices")}
    />
  );
}

export default App;
