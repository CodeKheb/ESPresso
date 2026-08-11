use serde::{Deserialize, Serialize};

/// A person's coffee profile (app-facing, camelCase).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub device_id: String,
    pub name: String,
    pub role: String,
    pub bio: String,
}

/// Raw profile as broadcast by a pot server (snake_case wire format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    #[serde(rename = "device_id", default)]
    pub device_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub bio: String,
}

impl From<DeviceProfile> for Profile {
    fn from(p: DeviceProfile) -> Self {
        Profile {
            device_id: p.device_id,
            name: p.name,
            role: p.role,
            bio: p.bio,
        }
    }
}

/// Profile sent to a pot (snake_case, matching the ESP32's parser).
#[derive(Debug, Clone, Serialize)]
pub struct OutgoingProfile {
    pub device_id: String,
    pub name: String,
    pub role: String,
    pub bio: String,
}

impl From<Profile> for OutgoingProfile {
    fn from(p: Profile) -> Self {
        OutgoingProfile {
            device_id: p.device_id,
            name: p.name,
            role: p.role,
            bio: p.bio,
        }
    }
}

/// Incoming WebSocket message from a pot.
///
/// Three shapes are supported:
/// - `{ "type": "profiles", "data": [...] }` — full store sync/broadcast
/// - `{ "type": "hello", "device_id": "..." }` — sent by app pots on connect
/// - `{ "device_id", "name", "role", "bio" }` — a single profile (ESP32 style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type", default)]
    pub msg_type: String,
    #[serde(
        rename = "device_id",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub device_id: Option<String>,
    #[serde(default)]
    pub data: Vec<DeviceProfile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: i64,
    pub device_id: String,
    pub name: String,
    pub role: String,
    pub bio: String,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub source: String,
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredDevice {
    pub name: String,
    pub host: String,
    pub ip: Option<String>,
    pub port: u16,
}

/// Info about the pot this device hosts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    pub hostname: String,
    pub port: u16,
    pub instance: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatus {
    pub state: String,
    pub host: Option<String>,
    pub message: Option<String>,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self {
            state: "connecting".into(),
            host: None,
            message: None,
        }
    }
}
