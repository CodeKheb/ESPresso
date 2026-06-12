import { useEffect, useRef, useState } from "react";
// import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type Status = "connecting" | "connected" | "disconnected" | "error";

function App() {

  const wsRef = useRef<WebSocket | null>(null);
  const [message, setMessage] = useState("");
  const [status, setStatus] = useState<Status>("connecting");


  useEffect(() => {
      const ws = new WebSocket('ws://192.168.4.1/ws');
      ws.onopen = () => setStatus("connected");
      ws.onclose = () => setStatus("disconnected");
      ws.onmessage = (message) => console.log('message:', message.data);
      ws.onerror = (error) => console.error('error:', error);
      wsRef.current = ws;

      return () => ws.close();
  }, []); 

  function send() {
      if (wsRef.current?.readyState == WebSocket.OPEN) {
          wsRef.current.send(message);
          setMessage("");
      }
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

  return (
    <main className="container">
      <h1>ESPresso</h1>

      <div className="row">
       <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
     </div>
      <p>Brewed from an ESP32</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          send();
        }}
      >
        <input
          id="greet-input"
          value={message}
          onChange={(e) => setMessage(e.currentTarget.value)}
          placeholder="Type message to ESPresso..."
        />
        <button type="submit">Send</button>
      </form>
    </main>
  );
}

export default App;
