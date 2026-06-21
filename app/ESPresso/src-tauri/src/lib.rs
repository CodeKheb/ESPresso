// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
            .add_migrations(
                "sqlite:profiles.db",
                vec![
                tauri_plugin_sql::Migration {
                    version: 1,
                    description: "create profiles table",
                    sql: "CREATE TABLE profiles (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        role TEXT NOT NULL,
                        bio TEXT,
                        created_at TEXT DEFAULT CURRENT_TIMESTAMP
                    );",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 2,
                    description: "add unique constraint on name",
                    sql: "CREATE UNIQUE INDEX idx_profiles_name ON profiles(name);",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 3,
                    description: "create contacts table",
                    sql: "CREATE TABLE contacts (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          name TEXT NOT NULL,
                          role TEXT NOT NULL,
                          bio TEXT,
                          saved_at TEXT DEFAULT CURRENT_TIMESTAMP,
                          UNIQUE(name)
                          );",
                          kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 4,
                    description: "create device table",
                    sql: "CREATE TABLE IF NOT EXISTS device (id TEXT PRIMARY KEY);",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 5,
                    description: "add device_id to profiles",
                    sql: "ALTER TABLE profiles ADD COLUMN device_id TEXT;",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 6,
                    description: "drop old name uniqueness on profiles",
                    sql: "DROP INDEX IF EXISTS idx_profiles_name;",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 7,
                    description: "add unique index on profiles.device_id",
                    sql: "CREATE UNIQUE INDEX idx_profiles_device_id ON profiles(device_id);",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 8,
                    description: "rebuild contacts table with device_id",
                    sql: "CREATE TABLE contacts_new (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT UNIQUE,
        name TEXT NOT NULL,
        role TEXT NOT NULL,
        bio TEXT,
        saved_at TEXT DEFAULT CURRENT_TIMESTAMP
    );",
    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 9,
                    description: "copy contacts into new table",
                    sql: "INSERT INTO contacts_new (id, name, role, bio, saved_at)
        SELECT id, name, role, bio, saved_at FROM contacts;",
        kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 10,
                    description: "drop old contacts table",
                    sql: "DROP TABLE contacts;",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },
                tauri_plugin_sql::Migration {
                    version: 11,
                    description: "rename contacts_new to contacts",
                    sql: "ALTER TABLE contacts_new RENAME TO contacts;",
                    kind: tauri_plugin_sql::MigrationKind::Up,
                },        ],
                )
                    .build(),
                )   
                    .invoke_handler(tauri::generate_handler![greet])
                    .run(tauri::generate_context!())
                    .expect("error while running tauri application");
}
