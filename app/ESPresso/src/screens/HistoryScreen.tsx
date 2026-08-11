import { Profile, Screen } from "../types";
import { TopBar } from "./TopBar";
import { PersonCard } from "./DashboardScreen";

type Props = {
  profiles: Profile[];
  deviceId: string;
  onNavigate: (screen: Screen) => void;
  onSettings: () => void;
};

export function HistoryScreen({ profiles, deviceId, onNavigate, onSettings }: Props) {
  // The database already dedupes by device_id (UNIQUE constraint), so we only
  // need to hide our own profile here.
  const history = profiles.filter((p) => p.deviceId !== deviceId);

  return (
    <div className="app-shell">
      <TopBar onSettings={onSettings} />
      <main className="dashboard-screen">
        <header className="dashboard-header">
          <h2>History</h2>
        </header>
        {history.length === 0 ? (
          <div className="empty-state">
            No profiles seen yet. They'll show up here once you join a pot.
          </div>
        ) : (
          <div className="people-list">
            {history.map((person, i) => (
              <PersonCard
                key={person.deviceId}
                person={person}
                index={i}
                showAddButton={false}
              />
            ))}
          </div>
        )}
      </main>
      <nav className="bottom-nav">
        <button className="nav-item" onClick={() => onNavigate("create")}>
          <span className="material-symbols-outlined">local_cafe</span>
          <span>Brew</span>
        </button>
        <button className="nav-item" onClick={() => onNavigate("dashboard")}>
          <span className="material-symbols-outlined icon-filled">import_contacts</span>
          <span>Profiles</span>
        </button>
        <button className="nav-item" onClick={() => onNavigate("contacts")}>
          <span className="material-symbols-outlined">call</span>
          <span>Contacts</span>
        </button>
        <button className="nav-item active" onClick={() => onNavigate("history")}>
          <span className="material-symbols-outlined icon-filled">history</span>
          <span>History</span>
        </button>
      </nav>
    </div>
  );
}
