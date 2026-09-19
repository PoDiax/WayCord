use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};

use crate::VoiceUser;
use super::client::DiscordClientInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InboundUser {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub speaking: bool,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub deafened: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InboundMessage {
    #[serde(rename = "channel")]
    Channel {
        channel: Option<String>,
        users: Vec<InboundUser>,
        #[serde(default)]
        current_user: Option<String>,
        #[serde(default)]
        client: Option<String>,
    },
    #[serde(rename = "speaking")]
    Speaking {
        #[serde(rename = "userId")]
        user_id: String,
        speaking: bool,
        #[serde(default)]
        client: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping {
        #[serde(default)]
        current_user: Option<String>,
        #[serde(default)]
        client: Option<String>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BridgeSource {
    FallbackFile = 1,
    UnixSocket = 2,
    WebSocket = 3,
}

#[derive(Clone, Debug)]
pub struct DiscordVoiceState {
    pub channel_name: Option<String>,
    pub users: Vec<VoiceUser>,
    pub user_map: HashMap<String, usize>,
    pub connected: bool,
    pub client_name: Option<String>,
    pub current_user: Option<String>,
    pub client_info: DiscordClientInfo,
    pub active_source: Option<BridgeSource>,
    pub enforce_vesktop_priority: bool,
}

impl Default for DiscordVoiceState {
    fn default() -> Self {
        Self {
            channel_name: None,
            users: Vec::new(),
            user_map: HashMap::new(),
            connected: false,
            client_name: None,
            current_user: None,
            client_info: DiscordClientInfo::detect(),
            active_source: None,
            enforce_vesktop_priority: true,
        }
    }
}

pub fn get_runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("/tmp")
}

pub fn get_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() >= 2 {
        let first = parts[0].chars().next().unwrap_or('?');
        let second = parts[1].chars().next().unwrap_or('?');
        format!("{first}{second}").to_uppercase()
    } else {
        name.chars().take(2).collect::<String>().to_uppercase()
    }
}
