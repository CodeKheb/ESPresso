import { DBProfile, Screen } from "../types";
import { TopBar } from "./TopBar";
import { PersonCard } from "./DashboardScreen";

type Props = {
  profiles: DBProfile[];
  onNavigate: (screen: Screen) => void;
};

export function HistoryScreen({ profiles, onNavigate }: Props) {
  return (
    <div className="app-shell">
      <TopBar />
      <main className="dashboard-screen">
        <header className="dashboard-header">
          <h2>History</h2>
        </header>
        {profiles.length === 0 ? (
          <div className="empty-state">
            No profiles seen yet. They'll show up here once you join a pot.
          </div>
        ) : (
          <div className="people-list">
            {profiles.map((person, i) => (
              <PersonCard
                key={person.id}
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
