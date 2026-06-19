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
            ],
        )
        .build(),
    )   
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
