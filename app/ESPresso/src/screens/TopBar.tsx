export function TopBar() {
  return (
    <header className="top-bar">
      <div className="top-bar-brand">
        <span className="material-symbols-outlined" style={{ color: "var(--primary)" }}>coffee</span>
        <span className="brand-name">ESPresso</span>
      </div>
      <button className="top-bar-icon-btn" aria-label="Settings">
        <span className="material-symbols-outlined">settings</span>
      </button>
    </header>
  );
}
