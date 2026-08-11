import { Contact, Screen } from "../types";
import { TopBar } from "./TopBar";
import { PersonCard } from "./DashboardScreen";

type Props = {
  contacts: Contact[];
  onNavigate: (screen: Screen) => void;
  onSettings: () => void;
};

export function ContactsScreen({ contacts, onNavigate, onSettings }: Props) {
  return (
    <div className="app-shell">
      <TopBar onSettings={onSettings} />
      <main className="dashboard-screen">
        <header className="dashboard-header">
          <h2>Contacts</h2>
        </header>
        {contacts.length === 0 ? (
          <div className="empty-state">
            No saved contacts yet. Add someone from the Dashboard.
          </div>
        ) : (
          <div className="people-list">
            {contacts.map((person, i) => (
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
        <button className="nav-item active" onClick={() => onNavigate("contacts")}>
          <span className="material-symbols-outlined">call</span>
          <span>Contacts</span>
        </button>
        <button className="nav-item" onClick={() => onNavigate("history")}>
          <span className="material-symbols-outlined">history</span>
          <span>History</span>
        </button>
      </nav>
    </div>
  );
}
