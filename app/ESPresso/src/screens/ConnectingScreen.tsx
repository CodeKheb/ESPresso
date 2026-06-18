import { useEffect, useRef, useState } from "react";
import { TopBar } from "./TopBar";

const MESSAGES = [
  "Brewing a connection...",
  "Warming up the sensors...",
  "Grinding the data...",
  "Tamping the signal...",
];

export function ConnectingScreen() {
  const [msgIndex, setMsgIndex] = useState(0);
  const [fade, setFade] = useState(true);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    timerRef.current = setInterval(() => {
      setFade(false);
      setTimeout(() => {
        setMsgIndex((i) => (i + 1) % MESSAGES.length);
        setFade(true);
      }, 400);
    }, 4000);
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, []);

  return (
    <div className="app-shell">
      <TopBar />
      <main className="screen-centered">
        <div className="connecting-icon-wrap">
          <div className="connecting-icon-circle">
            <span className="material-symbols-outlined">local_cafe</span>
          </div>
        </div>

        <div style={{ textAlign: "center" }}>
          <h1
            className="headline-lg-mobile connecting-title"
            style={{
              transition: "opacity 0.4s",
              opacity: fade ? 1 : 0,
              marginBottom: "var(--space-sm)",
            }}
          >
            {MESSAGES[msgIndex]}
          </h1>
          <p className="connecting-sub body-md">
            Setting up your barista-grade environment. Please keep your device near the ESP32 node.
          </p>
        </div>

        <div className="spinner-wrap">
          <div className="spinner-ring" />
          <div className="dot-row">
            <span /><span /><span />
          </div>
        </div>

        <div className="connecting-badge">
          <span className="material-symbols-outlined" style={{ fontSize: 14 }}>router</span>
          <span>Scanning Bluetooth LE</span>
        </div>
      </main>
    </div>
  );
}
