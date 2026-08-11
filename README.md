# ESPresso (Tauri App)

Tauri 2 + React + TypeScript desktop app for sharing coffee profiles over any
local network. **The WiFi is the DNS** — every device with ESPresso open hosts
a pot, and pots on the same network find each other automatically via mDNS.

## How it works

There is no hub device (the ESP32 is optional). Every app instance:

1. **Hosts a pot server** — a WebSocket server on port 8080 (8081–8086 if
   busy), advertised as `_espresso._tcp.local.` over mDNS with a unique
   hostname like `espresso-<id>.local`.
2. **Discovers peers** — every 15s it browses mDNS for other pots on the
   current WiFi and joins each one (a mesh).
3. **Exchanges profiles** — on connect, pots greet with `hello` + a full
   store snapshot. Profile updates are broadcast to the whole mesh and
   persisted to local SQLite (`profiles.db`).

Profiles converge across the network: you see everyone's profile, can add
them as contacts, and your history keeps everything you've ever seen.

## Architecture

```
src-tauri/src/
├── lib.rs      # Tauri commands, events, backend setup
├── models.rs   # serde data models (Profile, Contact, Device, HostInfo)
├── db.rs       # SQLite (rusqlite) with PRAGMA user_version migrations
├── mdns.rs     # mDNS discovery + advertising (mdns-sd)
├── server.rs   # PotServer: WS pot server, store, broadcast, hello+snapshot
└── ws.rs       # PotHub: hosts the server, joins peers, source-tagged store

src/
├── App.tsx          # thin React shell consuming commands/events
├── lib/espresso.ts  # typed invoke() + event wrappers
└── screens/         # presentational screens
```

- **Database** — `rusqlite` (bundled SQLite) at the app data dir. The old
  plugin-sql schema is detected and backed up to `profiles.db.bak-<ts>`
  before installing the clean schema.
- **Profile store** — every profile entry is tagged with its source
  (local / incoming client / outgoing peer), so when a peer's connection
  drops, only the profiles it contributed are removed. Full-store syncs use
  replace-by-source semantics, which makes the mesh converge — including
  removals.
- **Events** — the backend emits `connection://status`, `profiles://updated`
  (live mesh), `contacts://updated`, `devices://updated` and
  `discovery://done`.

## Wire protocol (port 8080)

- **Server → client on connect:** `{ "type": "hello", "device_id": "…" }`
  then `{ "type": "profiles", "data": [ { "device_id", "name", "role",
  "bio" } ] }` (full store snapshot).
- **Broadcasts:** `{ "type": "profiles", "data": [...] }` whenever the store
  changes.
- **Client → server:** either a single profile `{ "device_id", "name",
  "role", "bio" }` (also compatible with an ESP32-style pot) or a full-store
  `profiles` message for mesh sync.

## Development

```bash
npm install
npm run tauri dev
```

Open ESPresso on two devices on the same WiFi to see profiles sync. On a
single machine you can also run two instances to test the mesh.
