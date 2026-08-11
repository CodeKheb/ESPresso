use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension, Result};
use uuid::Uuid;

use crate::models::{Contact, Device, Profile};

pub const META_DEVICE_ID: &str = "device_id";

/// v1 schema. `user_version` gates future migrations.
const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS app_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS profiles (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id  TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    role       TEXT NOT NULL DEFAULT '',
    bio        TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS contacts (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id  TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    role       TEXT NOT NULL DEFAULT '',
    bio        TEXT NOT NULL DEFAULT '',
    saved_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS devices (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL DEFAULT 'ESPresso',
    host       TEXT NOT NULL UNIQUE,
    port       INTEGER NOT NULL DEFAULT 80,
    source     TEXT NOT NULL DEFAULT 'auto',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_seen  TEXT
);
CREATE INDEX IF NOT EXISTS idx_profiles_updated ON profiles(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_contacts_saved   ON contacts(saved_at DESC);
"#;

/// Tables created by the old (buggy) plugin-sql schema — removed on migration.
const LEGACY_TABLES: &[&str] = &[
    "profiles",
    "contacts",
    "contacts_new",
    "profiles_new",
    "device",
    "_sqlx_migrations",
];

pub struct Db {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}

impl Db {
    pub fn open(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
            db_path,
        };
        db.prepare_schema()?;
        Ok(db)
    }

    /// Detects the legacy (plugin-sql) database, backs it up, and installs the
    /// clean schema. The old database is frequently broken: its migration chain
    /// aborts on duplicate `device_id` values (a side effect of the frontend
    /// sending `deviceId` while the firmware parsed `device_id`), so a backup +
    /// clean rebuild is more reliable than a best-effort migration.
    fn prepare_schema(&self) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();

        let version: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let legacy = version == 0 && table_exists(&conn, "profiles");

        if legacy {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let backup = self
                .db_path
                .with_extension(format!("db.bak-{stamp}"));
            let backup_sql = format!(
                "VACUUM INTO '{}'",
                backup.to_string_lossy().replace('\'', "''")
            );
            eprintln!("[db] migrating legacy database, backing up to {}", backup.display());
            // Never drop the legacy data unless we have a verified backup first.
            if let Err(err) = conn.execute_batch(&backup_sql) {
                eprintln!("[db] backup failed, aborting migration: {err}");
                return Err(err);
            }

            for table in LEGACY_TABLES {
                let _ = conn.execute_batch(&format!("DROP TABLE IF EXISTS {table};"));
            }
            let _ = conn.execute_batch("PRAGMA user_version = 0;");
        }

        // Apply migrations up to the current version.
        if version < 1 {
            let tx = conn.transaction()?;
            tx.execute_batch(SCHEMA_V1)?;
            tx.pragma_update(None, "user_version", 1)?;
            tx.commit()?;
        }

        Ok(())
    }

    // ── meta ────────────────────────────────────────────────────────────────

    pub fn get_or_create_device_id(&self) -> String {
        if let Some(id) = self.get_meta(META_DEVICE_ID) {
            return id;
        }
        let id = Uuid::new_v4().to_string();
        self.set_meta(META_DEVICE_ID, &id);
        id
    }

    pub fn get_meta(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value FROM app_meta WHERE key = ?1",
            [key],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten()
    }

    pub fn set_meta(&self, key: &str, value: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO app_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        );
    }

    // ── profiles ────────────────────────────────────────────────────────────

    pub fn upsert_profile(&self, p: &Profile) -> Result<()> {
        if p.device_id.trim().is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO profiles (device_id, name, role, bio) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(device_id) DO UPDATE SET
                name = excluded.name,
                role = excluded.role,
                bio  = excluded.bio,
                updated_at = datetime('now')",
            params![p.device_id, p.name, p.role, p.bio],
        )
        .map(|_| ())
    }

    pub fn get_profiles(&self) -> Vec<Profile> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT device_id, name, role, bio FROM profiles ORDER BY updated_at DESC",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = stmt.query_map([], |r| {
            Ok(Profile {
                device_id: r.get(0)?,
                name: r.get(1)?,
                role: r.get(2)?,
                bio: r.get(3)?,
            })
        });
        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        }
    }

    pub fn get_my_profile(&self, device_id: &str) -> Option<Profile> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT device_id, name, role, bio FROM profiles WHERE device_id = ?1",
            [device_id],
            |r| {
                Ok(Profile {
                    device_id: r.get(0)?,
                    name: r.get(1)?,
                    role: r.get(2)?,
                    bio: r.get(3)?,
                })
            },
        )
        .optional()
        .ok()
        .flatten()
    }

    // ── contacts ────────────────────────────────────────────────────────────

    pub fn add_contact(&self, p: &Profile) -> Result<()> {
        if p.device_id.trim().is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO contacts (device_id, name, role, bio) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(device_id) DO UPDATE SET
                name = excluded.name,
                role = excluded.role,
                bio  = excluded.bio,
                saved_at = datetime('now')",
            params![p.device_id, p.name, p.role, p.bio],
        )
        .map(|_| ())
    }

    pub fn get_contacts(&self) -> Vec<Contact> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT id, device_id, name, role, bio, saved_at FROM contacts ORDER BY saved_at DESC",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = stmt.query_map([], |r| {
            Ok(Contact {
                id: r.get(0)?,
                device_id: r.get(1)?,
                name: r.get(2)?,
                role: r.get(3)?,
                bio: r.get(4)?,
                saved_at: r.get(5)?,
            })
        });
        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        }
    }

    // ── devices ─────────────────────────────────────────────────────────────

    pub fn upsert_device(&self, name: &str, host: &str, port: u16, source: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO devices (name, host, port, source, last_seen) VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(host) DO UPDATE SET
                name = excluded.name,
                last_seen = datetime('now'),
                source = CASE WHEN devices.source = 'manual' THEN 'manual' ELSE excluded.source END",
            params![name, host, port, source],
        );
    }

    pub fn get_devices(&self) -> Vec<Device> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT id, name, host, port, source, last_seen FROM devices ORDER BY last_seen DESC",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = stmt.query_map([], |r| {
            Ok(Device {
                id: r.get(0)?,
                name: r.get(1)?,
                host: r.get(2)?,
                port: r.get(3)?,
                source: r.get(4)?,
                last_seen: r.get(5)?,
            })
        });
        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        }
    }

    pub fn remove_device(&self, id: i64) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM devices WHERE id = ?1", [id]);
    }
}

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}
