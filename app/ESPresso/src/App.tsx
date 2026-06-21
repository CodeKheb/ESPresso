import { useEffect, useRef, useState } from "react";
import "./App.css";
import "./screens/screens.css";
import { Contact, DBProfile, Profile, WSMessage, Status, Screen } from "./types";
import { ConnectingScreen }    from "./screens/ConnectingScreen";
import { DisconnectedScreen }  from "./screens/DisconnectedScreen";
import { CreateScreen }        from "./screens/CreateScreen";
import { DashboardScreen }     from "./screens/DashboardScreen";
import { ContactsScreen }      from "./screens/ContactsScreen";
import { HistoryScreen }       from "./screens/HistoryScreen";
import Database from "@tauri-apps/plugin-sql";

function App() {
    const wsRef = useRef<WebSocket | null>(null);
    const dbRef = useRef<Database | null>(null);
    const [status, setStatus]   = useState<Status>("connecting");
    const [screen, setScreen]   = useState<Screen>("create");
    const [profiles, setProfiles] = useState<DBProfile[]>([]);
    const [contacts, setContacts] = useState<Contact[]>([]);
    const [connectedProfiles, setConnectedProfiles] = useState<Profile[]>([]);
    const [name, setName] = useState("");
    const [role, setRole] = useState("");
    const [bio,  setBio]  = useState("");
    const [deviceId, setDeviceId] = useState<string>("");

    // SQLite helpers 
    async function fetchProfiles(db: Database): Promise<DBProfile[]> {
        return db.select<DBProfile[]>(
            "SELECT * FROM profiles ORDER BY created_at DESC"
        );
    }

    async function fetchContacts(db: Database): Promise<Contact[]> {
        return db.select<Contact[]>("SELECT * FROM contacts ORDER BY saved_at DESC"
                                   );
    }

    async function upsertProfile(db: Database, profile: Profile) {
        await db.execute(
            `INSERT INTO profiles (device_id, name, role, bio) VALUES ($1, $2, $3, $4)
            ON CONFLICT(device_id) DO UPDATE SET name = excluded.name, role = excluded.role, bio = excluded.bio`,
                [profile.deviceId, profile.name, profile.role, profile.bio]
        );
    }

    async function upsertManyFromWS(db: Database, incoming: Profile[]) {
        for (const p of incoming) {
            await upsertProfile(db, p);
        }
    }

    async function addContact(db: Database, person: Profile) {
        await db.execute(
            'INSERT INTO contacts (device_id, name, role, bio) VALUES ($1, $2, $3, $4) ON CONFLICT(device_id) DO UPDATE SET name = excluded.name, role = excluded.role, bio = excluded.bio',
            [person.deviceId, person.name, person.role, person.bio]
        );
    }

    async function handleAddContact(person: Profile) {
        if (!dbRef.current) return;
        console.log("Adding contact:", person);
        await addContact(dbRef.current, person);
        const refreshed = await fetchContacts(dbRef.current);
        console.log("Contacts after insert:", refreshed);
        setContacts(await fetchContacts(dbRef.current));
    }

    // DB init before WS 
    useEffect(() => {
        let cancelled = false;
        (async () => {
            const db = await Database.load("sqlite:profiles.db");
            dbRef.current = db;

            const rows = await db.select<{ id: string }[]>("SELECT id FROM device LIMIT 1");

            let id: string;

            if (rows.length > 0) {
                id = rows[0].id;
            } else {
                id = crypto.randomUUID();
                await db.execute("INSERT INTO device (id) VALUES ($1)", [id]);
            }

            if (!cancelled) setDeviceId(id);

            const cachedContacts = await fetchContacts(db);

            if (!cancelled) {
                setContacts(cachedContacts);
            }     
        })();
        return () => { cancelled = true; };
    }, []);

    // WebSocket logic
    useEffect(() => {
        let cancelled = false;
        function connect() {
            const ws = new WebSocket("ws://192.168.4.1/ws");
            const timeout = setTimeout(() => { ws.close(); }, 5000);
            ws.onopen  = () => { clearTimeout(timeout); if (!cancelled) setStatus("connected"); };
            ws.onclose = () => {
                clearTimeout(timeout);
                if (!cancelled) {
                    setStatus("disconnected"); 
                    setConnectedProfiles([]);
                    setTimeout(connect, 3000); 
                }
            };
            ws.onerror = () => ws.close();
            ws.onmessage = async (json) => {
                const msg: WSMessage = JSON.parse(json.data);
                console.log("WS msg:", msg); 
                if (msg.type === "profiles") {
                    if (!cancelled) setConnectedProfiles(msg.data);
                    if (dbRef.current) {
                        await upsertManyFromWS(dbRef.current, msg.data);
                        const refreshed = await fetchProfiles(dbRef.current);
                        if (!cancelled) setProfiles(refreshed);
                    }
                }
            };
            wsRef.current = ws;
        }
        connect();
        return () => { cancelled = true; wsRef.current?.close(); };
    }, []);

    async function submitProfile() {
        if (!name || !role) return;
        const profile: Profile = { deviceId, name, role, bio };
        wsRef.current?.send(JSON.stringify(profile));

        if (dbRef.current) {
            await upsertProfile(dbRef.current, profile);
            const refreshed = await fetchProfiles(dbRef.current);
            setProfiles(refreshed);
        }
        setScreen("dashboard");
    }

    // Screen routing 
    if (status === "disconnected" || status === "error") {
        return <DisconnectedScreen />;
    }
    if (status === "connecting") {
        return <ConnectingScreen />;
    }
    if (screen === "create") {
        return (
            <CreateScreen
            name={name} role={role} bio={bio}
            onNameChange={setName}
            onRoleChange={setRole}
            onBioChange={setBio}
            onSubmit={submitProfile}
            />
        );
    }
    if (screen === "contacts") {
        return <ContactsScreen contacts={contacts} onNavigate={setScreen} />;
    }
    if (screen === "history") {
        return <HistoryScreen profiles={profiles} onNavigate={setScreen} />;
    }
    const savedNames = new Set(contacts.map((c) => c.name));
    return <DashboardScreen 
    profiles={connectedProfiles}
    savedNames={savedNames}
    onAddContact={handleAddContact}
    onNavigate={setScreen}
    />;
}

export default App;
