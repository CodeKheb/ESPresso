import { useEffect, useRef, useState } from "react";
import { TopBar } from "./TopBar";

const MESSAGES = [
  "Brewing a connection...",
  "Warming up the sensors...",
  "Grinding the data...",
  "Tamping the signal...",
];

type Props = {
  message?: string | null;
};

export function ConnectingScreen({ message }: Props) {
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
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
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
            Starting your pot and scanning this WiFi for others. Anyone with ESPresso
            open on the same network will appear automatically.
          </p>
          {message && <p className="label-sm connecting-status">{message}</p>}
        </div>

        <div className="spinner-wrap">
          <div className="spinner-ring" />
          <div className="dot-row">
            <span />
            <span />
            <span />
          </div>
        </div>

        <div className="connecting-badge">
          <span className="material-symbols-outlined" style={{ fontSize: 14 }}>radar</span>
          <span>Mesh pot on this WiFi</span>
        </div>
      </main>
    </div>
  );
}
