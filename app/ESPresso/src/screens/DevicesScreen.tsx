import { useState } from "react";
import type { Device, DiscoveredDevice, HostInfo, Screen } from "../types";
import { TopBar } from "./TopBar";

type Props = {
  hostInfo: HostInfo | null;
  devices: Device[];
  discovered: DiscoveredDevice[];
  currentHost: string | null;
  onConnect: (host: string) => void;
  onAdd: (host: string) => void;
  onRemove: (id: number) => void;
  onScan: () => Promise<void>;
  onNavigate: (screen: Screen) => void;
};

export function DevicesScreen({
  hostInfo,
  devices,
  discovered,
  currentHost,
  onConnect,
  onAdd,
  onRemove,
  onScan,
  onNavigate,
}: Props) {
  const [hostInput, setHostInput] = useState("");
  const [scanning, setScanning] = useState(false);
  const [addError, setAddError] = useState<string | null>(null);

  async function handleScan() {
    setScanning(true);
    try {
      await onScan();
    } finally {
      setScanning(false);
    }
  }

  async function handleAdd() {
    const host = hostInput.trim();
    if (!host) {
      setAddError("Enter a hostname or IP address.");
      return;
    }
    setAddError(null);
    await onAdd(host);
    setHostInput("");
  }

  return (
    <div className="app-shell">
      <TopBar />
      <main className="devices-screen">
        <header className="dashboard-header devices-header">
          <h2>Pots</h2>
          <p className="body-md devices-sub">
            Every device with ESPresso open hosts a pot. Everyone on the same WiFi can see each other automatically.
          </p>
        </header>

        {/* Your pot */}
        {hostInfo && hostInfo.port > 0 && (
          <section className="devices-section">
            <h3 className="label-md section-title">Your pot</h3>
            <div className="host-card">
              <div className="host-card-icon">
                <span className="material-symbols-outlined">local_cafe</span>
              </div>
              <div className="host-card-info">
                <div className="host-card-name">
                  {hostInfo.hostname}
                  <span className="host-card-live">
                    <span className="pulse-dot" />
                    live
                  </span>
                </div>
                <p className="host-card-desc label-sm">
                  Listening on port {hostInfo.port}. Others on this WiFi can see you as{" "}
                  <strong>{hostInfo.hostname}</strong>.
                </p>
              </div>
            </div>
          </section>
        )}

        {/* Discovered pots */}
        <section className="devices-section">
          <div className="scan-head">
            <h3 className="label-md section-title">Pots on this network</h3>
            <button className="btn-scan" onClick={handleScan} disabled={scanning}>
              <span
                className="material-symbols-outlined"
                style={scanning ? { animation: "spin 1s linear infinite" } : {}}
              >
                {scanning ? "progress_activity" : "radar"}
              </span>
              {scanning ? "Scanning…" : "Scan"}
            </button>
          </div>
          {discovered.length === 0 ? (
            <div className="empty-state devices-empty">
              No pots found yet — they appear automatically. Make sure other devices
              have ESPresso open on the same WiFi.
            </div>
          ) : (
            <div className="device-list">
              {discovered.map((d, i) => {
                const isCurrent = currentHost === d.host || currentHost === d.ip;
                return (
                  <div key={`${d.host}-${i}`} className={`device-row${isCurrent ? " active" : ""}`}>
                    <div className="device-info">
                      <div className="device-host">
                        <span className="material-symbols-outlined device-icon">radar</span>
                        <span className="device-host-text">{d.host}</span>
                        {isCurrent && <span className="device-dot" />}
                      </div>
                      <div className="device-meta">
                        <span className="device-tag discovered">discovered</span>
                        {d.ip && <span className="device-seen">{d.ip}</span>}
                      </div>
                    </div>
                    <div className="device-actions">
                      <button className="btn-device-connect" onClick={() => onConnect(d.host)}>
                        Join
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </section>

        {/* Saved pots */}
        <section className="devices-section">
          <h3 className="label-md section-title">Saved pots</h3>
          {devices.length === 0 ? (
            <div className="empty-state devices-empty">
              Pots you've joined are remembered here for quick access.
            </div>
          ) : (
            <div className="device-list">
              {devices.map((d) => {
                const isCurrent = currentHost === d.host;
                return (
                  <div key={d.id} className={`device-row${isCurrent ? " active" : ""}`}>
                    <div className="device-info">
                      <div className="device-host">
                        <span className="material-symbols-outlined device-icon">router</span>
                        <span className="device-host-text">{d.host}</span>
                        {isCurrent && <span className="device-dot" title="Joined" />}
                      </div>
                      <div className="device-meta">
                        <span className={`device-tag ${d.source}`}>{d.source}</span>
                        {d.lastSeen && <span className="device-seen">seen {d.lastSeen}</span>}
                      </div>
                    </div>
                    <div className="device-actions">
                      {!isCurrent && (
                        <button className="btn-device-connect" onClick={() => onConnect(d.host)}>
                          Join
                        </button>
                      )}
                      <button
                        className="btn-device-delete"
                        aria-label={`Remove ${d.host}`}
                        onClick={() => onRemove(d.id)}
                      >
                        <span className="material-symbols-outlined">delete</span>
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </section>

        {/* Add manually */}
        <section className="devices-section">
          <h3 className="label-md section-title">Join by address</h3>
          <div className="add-device-form">
            <div className="field-input-wrap add-device-input">
              <input
                className="coffee-input"
                type="text"
                placeholder="e.g. espresso-abc123.local or 192.168.1.50"
                value={hostInput}
                onChange={(e) => setHostInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleAdd();
                }}
              />
              <span className="material-symbols-outlined field-icon">link</span>
            </div>
            <button className="btn-device-connect btn-add" onClick={handleAdd}>
              <span className="material-symbols-outlined">add</span>
              <span>Join</span>
            </button>
          </div>
          {addError && <p className="add-error">{addError}</p>}
          <p className="devices-hint label-sm">
            Works on any WiFi — mDNS pots are found automatically; use this for hosts
            that don't advertise.
          </p>
        </section>
      </main>

      <nav className="bottom-nav">
        <button className="nav-item" onClick={() => onNavigate("create")}>
          <span className="material-symbols-outlined">local_cafe</span>
          <span>Brew</span>
        </button>
        <button className="nav-item active" onClick={() => onNavigate("dashboard")}>
          <span className="material-symbols-outlined icon-filled">import_contacts</span>
          <span>Profiles</span>
        </button>
        <button className="nav-item" onClick={() => onNavigate("contacts")}>
          <span className="material-symbols-outlined">call</span>
          <span>Contacts</span>
        </button>
        <button className="nav-item" onClick={() => onNavigate("history")}>
          <span className="material-symbols-outlined">history</span>
          <span>History</span>
        </button>
      </nav>
    </div>
  );
}
