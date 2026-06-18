import { TopBar } from "./TopBar";

type Props = {
  name: string;
  role: string;
  bio: string;
  onNameChange: (v: string) => void;
  onRoleChange: (v: string) => void;
  onBioChange:  (v: string) => void;
  onSubmit: () => void;
};

export function CreateScreen({ name, role, bio, onNameChange, onRoleChange, onBioChange, onSubmit }: Props) {
  return (
    <div className="app-shell">
      <TopBar />
      <main className="create-screen">
        <div className="create-inner">
          <div className="create-header">
            <h2 className="headline-lg-mobile">Join the Brew</h2>
            <p className="body-md">Start your journey into specialty profile sharing and coffee craft.</p>
          </div>

          <div className="brew-card">
            {/* Name */}
            <div className="field-group">
              <label className="field-label" htmlFor="name">Name</label>
              <div className="field-input-wrap">
                <input
                  id="name"
                  className="coffee-input"
                  type="text"
                  placeholder="E.g., Kherbin Buenaventura"
                  value={name}
                  onChange={(e) => onNameChange(e.target.value)}
                />
                <span className="material-symbols-outlined field-icon">person</span>
              </div>
            </div>

            {/* Role */}
            <div className="field-group">
              <label className="field-label" htmlFor="role">Role</label>
              <div className="field-input-wrap">
                <input
                  id="role"
                  className="coffee-input"
                  type="text"
                  placeholder="E.g., Startup Founder, Student"
                  value={role}
                  onChange={(e) => onRoleChange(e.target.value)}
                />
                <span className="material-symbols-outlined field-icon">work</span>
              </div>
            </div>

            {/* Bio */}
            <div className="field-group">
              <label className="field-label" htmlFor="bio">Bio</label>
              <textarea
                id="bio"
                className="coffee-input"
                rows={3}
                placeholder="Tell a little about yourself..."
                value={bio}
                onChange={(e) => onBioChange(e.target.value)}
              />
            </div>

            <button
              className="btn-join"
              onClick={onSubmit}
              disabled={!name || !role}
            >
              <span>Join</span>
              <span className="material-symbols-outlined">arrow_forward</span>
            </button>
          </div>

          <p className="create-note label-sm">
            By joining, you agree to share your coffee profiles with the local ESPresso network.
          </p>
        </div>
      </main>
    </div>
  );
}
