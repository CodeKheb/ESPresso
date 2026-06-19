import { useState } from "react";
import { TopBar } from "./TopBar";

export function DisconnectedScreen() {
  const [retrying, setRetrying] = useState(false);

  function handleRetry() {
    setRetrying(true);
    setTimeout(() => setRetrying(false), 2500);
  }

  return (
    <div className="app-shell">
      <TopBar />
      <main className="screen-centered">
        <div className="disconnected-illustration">
          <div className="disconnected-halo" />
          <div className="disc-cup">
            <div className="cup-body">
              <div className="cup-handle" />
              <div className="disc-badge">
                <span className="material-symbols-outlined">wifi_off</span>
              </div>
            </div>
          </div>
        </div>

        <div style={{ maxWidth: 320 }}>
          <h2 className="headline-lg-mobile disconnected-title">Off the Grid</h2>
          <p className="body-md disconnected-sub">
            Please connect to the <strong>ESPresso</strong> WiFi network to continue.
          </p>
        </div>

        <div className="disc-actions">
          <button className="btn-primary-pill" onClick={handleRetry} disabled={retrying}>
            <span
              className="material-symbols-outlined"
              style={retrying ? { animation: "spin 1s linear infinite" } : {}}
            >
              {retrying ? "progress_activity" : "refresh"}
            </span>
            {retrying ? "Connecting..." : "Retry Connection"}
          </button>
        </div>

        <div className="disc-tip">
          <div className="tip-icon-wrap">
            <span className="material-symbols-outlined">lightbulb</span>
          </div>
          <div>
            <p className="tip-title">Quick Tip</p>
            <p className="tip-body">
            Connect to the ESPresso wifi and turn off mobile data
            <p>Password: espresso</p>
            </p>
          </div>
        </div>
      </main>
    </div>
  );
}
