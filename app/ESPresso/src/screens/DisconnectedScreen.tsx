import { useState } from "react";
import { TopBar } from "./TopBar";

type Props = {
  message?: string | null;
  onRetry: () => void;
  onChooseDevice: () => void;
};

export function DisconnectedScreen({ message, onRetry, onChooseDevice }: Props) {
  const [retrying, setRetrying] = useState(false);

  function handleRetry() {
    setRetrying(true);
    onRetry();
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
            This device can't start its pot. Make sure you're connected to a WiFi network
            and try again.
          </p>
          {message && <p className="label-sm disconnected-status">{message}</p>}
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
          <button className="btn-primary-pill btn-secondary-pill" onClick={onChooseDevice}>
            <span className="material-symbols-outlined">router</span>
            View Pots
          </button>
        </div>

        <div className="disc-tip">
          <div className="tip-icon-wrap">
            <span className="material-symbols-outlined">lightbulb</span>
          </div>
          <div>
            <p className="tip-title">Quick Tip</p>
            <p className="tip-body">
              ESPresso works on <strong>any WiFi</strong> — every device with the app open
              hosts a pot and finds the others automatically via mDNS. No hub needed.
            </p>
          </div>
        </div>
      </main>
    </div>
  );
}
