import { useEffect, useRef, useState } from "react";
import "./App.css";
import "./screens/screens.css";
import { Profile, WSMessage, Status, Screen } from "./types";
import { ConnectingScreen }    from "./screens/ConnectingScreen";
import { DisconnectedScreen }  from "./screens/DisconnectedScreen";
import { CreateScreen }        from "./screens/CreateScreen";
import { DashboardScreen }     from "./screens/DashboardScreen";

function App() {
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus]   = useState<Status>("connecting");
  const [screen, setScreen]   = useState<Screen>("create");
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [name, setName] = useState("");
  const [role, setRole] = useState("");
  const [bio,  setBio]  = useState("");

  // WebSocket logic
  useEffect(() => {
    let cancelled = false;
    function connect() {
      const ws = new WebSocket("ws://192.168.4.1/ws");
      const timeout = setTimeout(() => { ws.close(); }, 5000);
      ws.onopen  = () => { clearTimeout(timeout); if (!cancelled) setStatus("connected"); };
      ws.onclose = () => {
        clearTimeout(timeout);
        if (!cancelled) { setStatus("disconnected"); setTimeout(connect, 3000); }
      };
      ws.onerror = () => ws.close();
      ws.onmessage = (json) => {
        const msg: WSMessage = JSON.parse(json.data);
        if (msg.type === "profiles") setProfiles(msg.data);
      };
      wsRef.current = ws;
    }
    connect();
    return () => { cancelled = true; wsRef.current?.close(); };
  }, []);

  function submitProfile() {
    if (!name || !role) return;
    const profile = { name, role, bio };
    wsRef.current?.send(JSON.stringify(profile));
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
  return <DashboardScreen profiles={profiles} />;
}

export default App;
