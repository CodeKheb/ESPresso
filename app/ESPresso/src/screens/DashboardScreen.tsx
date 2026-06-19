import { useState } from "react";
import { Profile } from "../types";
import { TopBar } from "./TopBar";
import { Screen } from "../types";

type Props = { 
    profiles: Profile[];
    savedNames: Set<string>;
    onAddContanct: (person: Profile) => void;
    onNavigate: (screen: Screen) => void;
};

const AVATAR_STYLES = ["avatar-cafe", "avatar-init-a", "avatar-init-b"];

function getInitials(name: string) {
  return name
    .split(" ")
    .map((w) => w[0])
    .slice(0, 2)
    .join("")
    .toUpperCase();
}


export function PersonCard({ 
    person, 
    index,
    isSaved,
    onAdd,
    showAddButton = true,
}: { 
    person: Profile; 
    index: number;
    isSaved?: boolean;
    onAdd?: () => void;
    showAddButton?: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  const avatarClass = AVATAR_STYLES[index % AVATAR_STYLES.length];

  return (
    <div
      className={`person-card${expanded ? " expanded" : ""}`}
      onClick={() => setExpanded((e) => !e)}
    >
      <div className="person-card-header">
        <div className="person-card-left">
          <div className={`avatar ${avatarClass}`}>
            {index === 0 ? (
              <span className="material-symbols-outlined icon-filled">local_cafe</span>
            ) : (
              getInitials(person.name)
            )}
          </div>
          <span className="person-name">{person.name}</span>
        </div>
        <span className="material-symbols-outlined expand-icon">expand_more</span>
      </div>

      <div className="person-card-body">
        <div className="person-card-body-inner">
          <p className="person-role">{person.role}</p>
          {person.bio && <p className="person-bio">"{person.bio}"</p>}
          {showAddButton && (
              <button
                className="btn-add-contact"
                disabled={isSaved}
                onClick={(e)=> {
                    e.stopPropagation();
                    onAdd?.();
                }}
            >
            <span className="material-symbols-outlined">
             {isSaved ? "check" : "person_add"}
             </span>
             <span>{isSaved ? "Saved" : "Add"}</span>
             </button>
          )}
        </div>
      </div>
    </div>
  );
}

export function DashboardScreen({ profiles, savedNames, onAddContanct, onNavigate }: Props) {
  return (
    <div className="app-shell">
      <TopBar />
      <main className="dashboard-screen">
        <header className="dashboard-header">
          <h2>Dashboard</h2>
          <div className="active-indicator">
            <div className="pulse-dot" />
            <span className="active-label">Active Pot</span>
          </div>
        </header>

        {profiles.length === 0 ? (
          <div className="empty-state">
            Waiting for others to join the pot...
          </div>
        ) : (
          <div className="people-list">
            {profiles.map((person, i) => (
              <PersonCard 
               key={i}
               person={person}
               index={i}
               isSaved={savedNames.has(person.name)}
               onAdd={() => onAddContanct(person)}
            />
            ))}
          </div>
        )}
      </main>

      {/* Bottom Nav */}
      <nav className="bottom-nav">
        <button className="nav-item" onClick={() => onNavigate("create")}>
          <span className="material-symbols-outlined">local_cafe</span>
          <span>Brew</span>
        </button>
        <button className="nav-item active" onClick={() => onNavigate("dashboard")}>
          <span className="material-symbols-outlined icon-filled">import_contacts</span>
          <span>Profiles</span>
        </button>
        <button className="nav-item" onClick={() => onNavigate("contacts")}>
          <span className="material-symbols-outlined">call</span>
          <span>Contacts</span>
        </button>
        <button className="nav-item">
          <span className="material-symbols-outlined">history</span>
          <span>History</span>
        </button>
      </nav>
    </div>
  );
}
