import { useEffect, useRef, useState } from "react";
import reactLogo from "./assets/react.svg";
// import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {

  const wsRef = useRef<WebSocket | null>(null);
  const [message, setMessage] = useState("");


  useEffect(() => {
      const ws = new WebSocket('ws://192.168.4.1/ws');
      ws.onopen = () => console.log('connected');
      ws.onmessage = (e) => console.log('message:', e.data);
      ws.onerror = (e) => console.error('error:', e);
      wsRef.current = ws;

      return () => ws.close();
  }, []); 

  function send() {
      if (wsRef.current?.readyState == WebSocket.OPEN) {
          wsRef.current.send(message);
          setMessage("");
      }
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

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
          placeholder="Type a to ESPresso..."
        />
        <button type="submit">Send</button>
      </form>
    </main>
  );
}

export default App;
