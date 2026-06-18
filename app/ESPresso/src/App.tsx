import { useEffect, useRef, useState } from "react";
import "./App.css";

type Profile = {
    name: string;
    role: string;
    bio: string;
}

type WSmessage = {
    type: string;
    data: Profile[];
}

type Status = "connecting" | "connected" | "disconnected" | "error";
type Screen = "create" | "dashboard";

function App() {

    const wsRef = useRef<WebSocket | null>(null);
    // const [message, setMessage] = useState("");
    const [status, setStatus] = useState<Status>("connecting");
    const [screen, setScreen] = useState<Screen>("create");
    const [profiles, setProfiles] = useState<Profile[]>([]);

    const [name, setName] = useState("");
    const [role, setRole] = useState("");
    const [bio, setBio] = useState("");

    useEffect(() => {
        let cancelled = false;

        function connect() {
            const ws = new WebSocket('ws://192.168.4.1/ws');

            const timeout = setTimeout(() => {
                ws.close(); // force close, triggers onclose
            }, 5000);

            ws.onopen = () => { 
                clearTimeout(timeout);
                if (!cancelled) setStatus("connected"); 
            };
            ws.onclose = () => {
                clearTimeout(timeout);
                if (!cancelled) {
                    setStatus("disconnected");
                    setTimeout(connect, 3000);
                }
            };
            ws.onerror = () => ws.close();
            ws.onmessage = (json) => {
                const msg: WSmessage = JSON.parse(json.data);
                if (msg.type == "profiles") setProfiles(msg.data);
            };
            wsRef.current = ws;
        }
        connect();
        return () => { cancelled = true; wsRef.current?.close(); };
    }, []);

    function submitProfile() {
        if (!name || !role) return;
        const profile = {name, role, bio}; 
        wsRef.current?.send(JSON.stringify(profile));
        setScreen("dashboard");
    }

    if (status == "disconnected" || status == "error") {
        return (
            <main className="container">
            <h1>ESPresso</h1>
            <p>Not connected to ESPresso network.</p>
            <p>Connect to <strong>ESPresso</strong> WiFi first, then reopen the app.</p>
            </main>
        );
    }

    if (status === "connecting") {
        return (
            <main className="container">
            <h1>ESPresso</h1>
            <p>Connecting...</p>
            </main>
        );
    }

    if (screen == "create") {
        return (
            <main className="container">
            <h1>ESPresso</h1>
            <div className="row">
            <a href="https://tauri.app" target="_blank">
            <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
            </a>
            </div>
            <p>Brewed from an ESP32</p>


            <p>Create Profile</p>
            <input placeholder="Name" value={name} onChange={(e) => setName(e.target.value)} />
            <input placeholder="Role" value={role} onChange={(e) => setRole(e.target.value)} />
            <input placeholder="Tell a little about yourself:" value={bio} onChange={(e) => setBio(e.target.value)} />
            <button onClick={submitProfile}>Join</button>
            </main>
        )
    }

    return (
        <main className="container">
        <h1>ESPresso Dashboard</h1>

        <div className="row">
        <a href="https://tauri.app" target="_blank">
        <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        </div>
        <p>People in the room.</p>
        {profiles.map((person, id) =>
                      <div key={id}>
                      <h2>{person.name}</h2>
                      <p>{person.role}</p>
                      <p>{person.bio}</p>
                      </div>
                     )}
                     </main>
    );
}

export default App;
