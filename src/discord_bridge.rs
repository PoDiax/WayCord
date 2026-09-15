use std::{
    collections::HashMap,
    net::TcpListener,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use serde::{Deserialize, Serialize};
use tungstenite::Message;

use crate::{VoiceUser, avatar_cache::AvatarCache};

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
    },
    #[serde(rename = "speaking")]
    Speaking {
        #[serde(rename = "userId")]
        user_id: String,
        speaking: bool,
    },
    #[serde(rename = "ping")]
    Ping {
        #[serde(default)]
        current_user: Option<String>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct DiscordClientInfo {
    pub stock_discord_installed: bool,
    pub stock_discord_running: bool,
    pub vesktop_installed: bool,
    pub vesktop_running: bool,
    pub vencord_installed: bool,
    pub legcord_installed: bool,
    pub legcord_running: bool,
}

impl DiscordClientInfo {
    pub fn detect() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let home_path = PathBuf::from(home);

        let stock_dirs = [
            home_path.join(".config/discord"),
            home_path.join(".config/discordcanary"),
            home_path.join(".config/discordptb"),
            home_path.join(".config/discorddevelopment"),
            home_path.join(".var/app/com.discordapp.Discord"),
            home_path.join(".var/app/com.discordapp.DiscordCanary"),
        ];
        let stock_discord_installed = stock_dirs.iter().any(|p| p.exists())
            || std::path::Path::new("/usr/bin/discord").exists()
            || std::path::Path::new("/usr/bin/discord-canary").exists()
            || std::path::Path::new("/usr/bin/discord-ptb").exists();

        let vesktop_dirs = [
            home_path.join(".config/vesktop"),
            home_path.join(".var/app/dev.vencord.Vesktop"),
        ];
        let vesktop_installed = vesktop_dirs.iter().any(|p| p.exists())
            || std::path::Path::new("/usr/bin/vesktop").exists()
            || std::path::Path::new("/usr/lib/vesktop").exists();

        let vencord_dirs = [
            home_path.join(".config/Vencord"),
            home_path.join(".local/share/vencord"),
        ];
        let vencord_installed = vencord_dirs.iter().any(|p| p.exists());

        let legcord_dirs = [
            home_path.join(".config/legcord"),
            home_path.join(".var/app/app.legcord.Legcord"),
        ];
        let legcord_installed = legcord_dirs.iter().any(|p| p.exists())
            || std::path::Path::new("/usr/bin/legcord").exists()
            || std::path::Path::new("/usr/lib/legcord").exists();

        let stock_discord_running = is_process_running(&["discord", "discordcanary", "discord-ptb"], &["vesktop", "legcord"]);
        let vesktop_running = is_process_running(&["vesktop"], &[]);
        let legcord_running = is_process_running(&["legcord"], &[]);

        Self {
            stock_discord_installed,
            stock_discord_running,
            vesktop_installed,
            vesktop_running,
            vencord_installed,
            legcord_installed,
            legcord_running,
        }
    }

    pub fn is_stock_discord_active_or_only_client(&self) -> bool {
        self.stock_discord_running || (!self.vesktop_running && !self.legcord_running && self.stock_discord_installed)
    }

    pub fn should_prompt_for_vesktop(&self, connected: bool) -> bool {
        if connected || self.vesktop_running || self.legcord_running || self.legcord_installed {
            return false;
        }
        self.stock_discord_running || self.stock_discord_installed
    }
}

pub fn is_process_running(target_names: &[&str], exclude_names: &[&str]) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let pid_str = file_name.to_string_lossy();
        if !pid_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let comm_path = entry.path().join("comm");
        if let Ok(comm) = std::fs::read_to_string(comm_path) {
            let comm_lower = comm.trim().to_lowercase();
            let is_target = target_names.iter().any(|t| comm_lower.contains(&t.to_lowercase()));
            let is_excluded = exclude_names.iter().any(|e| comm_lower.contains(&e.to_lowercase()));
            if is_target && !is_excluded {
                return true;
            }
        }
    }
    false
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
        }
    }
}

pub struct DiscordBridge {
    pub state: Arc<Mutex<DiscordVoiceState>>,
    pub avatar_cache: AvatarCache,
}

impl DiscordBridge {
    pub fn start() -> Self {
        let state = Arc::new(Mutex::new(DiscordVoiceState::default()));
        let state_clone = state.clone();
        let avatar_cache = AvatarCache::new();
        let cache_clone = avatar_cache.clone();

        install_universal_bridge();

        thread::spawn(move || {
            let listener = match TcpListener::bind("[::]:9876") {
                Ok(l) => {
                    println!("Discord Voice Bridge WebSocket listening on [::]:9876 (dual-stack IPv4/IPv6)");
                    l
                }
                Err(_) => match TcpListener::bind("127.0.0.1:9876") {
                    Ok(l) => {
                        println!("Discord Voice Bridge WebSocket listening on ws://127.0.0.1:9876");
                        l
                    }
                    Err(err) => {
                        eprintln!("Failed to bind WebSocket listener on port 9876: {err}");
                        return;
                    }
                },
            };

            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let state_clone = state_clone.clone();
                let cache_clone = cache_clone.clone();

                thread::spawn(move || {
                    let _ = stream.set_read_timeout(None);
                    let mut ws = match tungstenite::accept(stream) {
                        Ok(ws) => {
                            println!("🟢 Connected to Discord Voice client!");
                            if let Ok(mut lock) = state_clone.lock() {
                                lock.connected = true;
                            }
                            ws
                        }
                        Err(e) => {
                            eprintln!("WebSocket accept error: {e}");
                            return;
                        }
                    };

                loop {
                    match ws.read() {
                        Ok(Message::Text(text)) => {
                            if let Ok(msg) = serde_json::from_str::<InboundMessage>(&text) {
                                if let Ok(mut lock) = state_clone.lock() {
                                    match msg {
                                        InboundMessage::Channel { channel, users, current_user } => {
                                            lock.channel_name = channel;
                                            lock.users.clear();
                                            lock.user_map.clear();

                                            if let Some(user) = current_user {
                                                if !user.trim().is_empty() && lock.current_user.as_deref() != Some(&user) {
                                                    println!("Connected to Discord as {}", user);
                                                    lock.current_user = Some(user);
                                                }
                                            }

                                            for (idx, u) in users.into_iter().enumerate() {
                                                lock.user_map.insert(u.id.clone(), idx);
                                                let initials = get_initials(&u.name);
                                                let avatar_url = match u.avatar.as_deref() {
                                                    Some(av) if av.starts_with("http") => Some(av.to_string()),
                                                    Some(hash) if !hash.trim().is_empty() => {
                                                        let ext = if hash.starts_with("a_") { "gif" } else { "png" };
                                                        Some(format!("https://cdn.discordapp.com/avatars/{}/{}.{}?size=128", u.id, hash, ext))
                                                    }
                                                    _ => {
                                                        let index = u.id.parse::<u64>().map(|id| (id >> 22) % 6).unwrap_or(0);
                                                        Some(format!("https://cdn.discordapp.com/embed/avatars/{}.png?size=128", index))
                                                    }
                                                };

                                                if let Some(url) = &avatar_url {
                                                    cache_clone.queue_download(u.id.clone(), url.clone());
                                                }

                                                lock.users.push(VoiceUser {
                                                    id: u.id,
                                                    name: u.name,
                                                    avatar_initials: initials,
                                                    avatar_url,
                                                    is_speaking: u.speaking,
                                                    is_muted: u.muted,
                                                    is_deafened: u.deafened,
                                                    volume_level: if u.speaking { 0.8 } else { 0.0 },
                                                });
                                            }
                                        }
                                        InboundMessage::Speaking { user_id, speaking } => {
                                            if let Some(&idx) = lock.user_map.get(&user_id) {
                                                if let Some(u) = lock.users.get_mut(idx) {
                                                    u.is_speaking = speaking;
                                                    u.volume_level = if speaking { 0.8 } else { 0.0 };
                                                }
                                            }
                                        }
                                        InboundMessage::Ping { current_user } => {
                                            if let Some(user) = current_user {
                                                if !user.trim().is_empty() && lock.current_user.as_deref() != Some(&user) {
                                                    println!("👤 Connected to Discord as {}", user);
                                                    lock.current_user = Some(user);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_)) | Err(_) => {
                            println!("🔴 Disconnected from Discord voice client (retaining last voice state).");
                            if let Ok(mut lock) = state_clone.lock() {
                                lock.connected = false;
                                // Retain last channel_name and users so overlay does not reset or flicker!
                                for u in &mut lock.users {
                                    u.is_speaking = false;
                                    u.volume_level = 0.0;
                                }
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            });
        }
    });

        Self {
            state,
            avatar_cache,
        }
    }
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

// Universal client script that integrates directly with Vencord / Equicord / Shelter / Vesktop / Legcord
const CLIENT_HOOK_JS: &str = r#"
(() => {
    if (window.__waycordBridge_v1) return;
    window.__waycordBridge_v1 = true;
    console.log("[WayCord Client] Voice overlay bridge hook active");

    function getDiscordModules() {
        const vc = window.Vencord || window.Equicord;
        if (vc?.Webpack?.Common?.FluxDispatcher &&
            vc?.Webpack?.Common?.VoiceStateStore &&
            vc?.Webpack?.Common?.SelectedChannelStore &&
            vc?.Webpack?.Common?.ChannelStore &&
            vc?.Webpack?.Common?.UserStore) {
            return {
                Dispatcher: vc.Webpack.Common.FluxDispatcher,
                VoiceStateStore: vc.Webpack.Common.VoiceStateStore,
                SelectedChannelStore: vc.Webpack.Common.SelectedChannelStore,
                ChannelStore: vc.Webpack.Common.ChannelStore,
                UserStore: vc.Webpack.Common.UserStore,
            };
        }

        if (window.shelter?.flux?.dispatcher && window.shelter?.flux?.stores) {
            const stores = typeof window.shelter.flux.stores === 'function' ? window.shelter.flux.stores() : window.shelter.flux.stores;
            if (stores?.VoiceStateStore && stores?.SelectedChannelStore && stores?.ChannelStore && stores?.UserStore) {
                return {
                    Dispatcher: window.shelter.flux.dispatcher,
                    VoiceStateStore: stores.VoiceStateStore,
                    SelectedChannelStore: stores.SelectedChannelStore,
                    ChannelStore: stores.ChannelStore,
                    UserStore: stores.UserStore,
                };
            }
        }
        return null;
    }

    function init() {
        const modules = getDiscordModules();
        if (!modules) {
            setTimeout(init, 1000);
            return;
        }

        const { Dispatcher, VoiceStateStore, SelectedChannelStore, ChannelStore, UserStore } = modules;
        console.log("[WayCord Client] Connected to Discord Flux modules successfully!");

        let ws = null;
        let reconnectTimer = null;

        function getCurrentUserName() {
            try {
                if (typeof UserStore.getCurrentUser === 'function') {
                    const u = UserStore.getCurrentUser();
                    if (u) {
                        return u.globalName || u.username || null;
                    }
                }
            } catch (e) {}
            return null;
        }

        const hosts = ["127.0.0.1:9876", "localhost:9876"];
        let hostIdx = 0;

        function connect() {
            try {
                const target = hosts[hostIdx % hosts.length];
                hostIdx++;
                ws = new WebSocket("ws://" + target);
                ws.onopen = () => {
                    console.log("[WayCord Client] Connected to overlay server (" + target + ")!");
                    sendFullState();
                };
                ws.onclose = () => {
                    ws = null;
                    if (!reconnectTimer) {
                        reconnectTimer = setTimeout(() => {
                            reconnectTimer = null;
                            connect();
                        }, 2000);
                    }
                };
                ws.onerror = () => {
                    if (ws) ws.close();
                };
            } catch (e) {
                if (!reconnectTimer) {
                    reconnectTimer = setTimeout(() => {
                        reconnectTimer = null;
                        connect();
                    }, 2000);
                }
            }
        }

        function getAvatarUrl(user, userId, guildId) {
            if (!user) return `https://cdn.discordapp.com/embed/avatars/0.png`;
            try {
                if (typeof user.getAvatarURL === 'function') {
                    const u = user.getAvatarURL(guildId, 128);
                    if (u) return u;
                }
            } catch(e) {}
            if (user.avatar) {
                const ext = user.avatar.startsWith("a_") ? "gif" : "png";
                return `https://cdn.discordapp.com/avatars/${userId}/${user.avatar}.${ext}?size=128`;
            }
            try {
                const defaultIndex = Number((BigInt(userId) >> 22n) % 6n);
                return `https://cdn.discordapp.com/embed/avatars/${defaultIndex}.png?size=128`;
            } catch(e) {
                return `https://cdn.discordapp.com/embed/avatars/0.png`;
            }
        }

        function sendFullState() {
            if (!ws || ws.readyState !== WebSocket.OPEN) return;
            try {
                const currentUserName = getCurrentUserName();
                const voiceChannelId = SelectedChannelStore.getVoiceChannelId();
                if (!voiceChannelId) {
                    ws.send(JSON.stringify({
                        type: "channel",
                        channel: null,
                        users: [],
                        current_user: currentUserName
                    }));
                    return;
                }
                const chan = ChannelStore.getChannel(voiceChannelId);
                const states = VoiceStateStore.getVoiceStatesForChannel(voiceChannelId) || {};
                const users = Object.values(states).map(st => {
                    const user = UserStore.getUser(st.userId) || {};
                    const avatarUrl = getAvatarUrl(user, st.userId, chan?.guild_id);
                    return {
                        id: st.userId,
                        name: user.globalName || user.username || "Unknown",
                        avatar: avatarUrl,
                        speaking: false,
                        muted: !!(st.mute || st.selfMute),
                        deafened: !!(st.deaf || st.selfDeaf),
                    };
                });

                let channelTitle = "Voice Call";
                if (chan) {
                    if (chan.name) {
                        channelTitle = chan.name;
                    } else if (chan.rawRecipients && chan.rawRecipients.length > 0) {
                        channelTitle = chan.rawRecipients.map(r => r.global_name || r.username).join(", ");
                    }
                }

                ws.send(JSON.stringify({
                    type: "channel",
                    channel: channelTitle,
                    users,
                    current_user: currentUserName
                }));
            } catch (err) {
                console.error("[WayCord] Error sending state:", err);
            }
        }

        Dispatcher.subscribe("SPEAKING", (ev) => {
            if (!ws || ws.readyState !== WebSocket.OPEN) return;
            ws.send(JSON.stringify({
                type: "speaking",
                userId: ev.userId,
                speaking: !!(ev.speakingFlags & 1)
            }));
        });

        Dispatcher.subscribe("VOICE_STATE_UPDATES", () => sendFullState());
        Dispatcher.subscribe("VOICE_CHANNEL_SELECT", () => sendFullState());
        try {
            Dispatcher.subscribe("CONNECTION_OPEN", () => sendFullState());
            Dispatcher.subscribe("CURRENT_USER_UPDATE", () => sendFullState());
        } catch (e) {}

        setInterval(() => {
            if (ws && ws.readyState === WebSocket.OPEN) {
                try {
                    ws.send(JSON.stringify({
                        type: "ping",
                        current_user: getCurrentUserName()
                    }));
                } catch(e) {}
            }
        }, 10000);

        connect();
    }

    setTimeout(init, 1500);
})();
"#;

pub fn install_universal_bridge() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = PathBuf::from(home);

    let targets = [
        home_path.join(".config/vesktop/sessionData/vencordFiles/vencordDesktopRenderer.js"),
        home_path.join(".var/app/dev.vencord.Vesktop/config/vesktop/sessionData/vencordFiles/vencordDesktopRenderer.js"),
        home_path.join(".config/Vencord/dist/renderer.js"),
        home_path.join(".local/share/vencord/dist/renderer.js"),
        // Legcord mods (Equicord, Vencord, Shelter)
        home_path.join(".config/legcord/equicord.js"),
        home_path.join(".config/legcord/vencord.js"),
        home_path.join(".config/legcord/shelter.js"),
        home_path.join(".var/app/app.legcord.Legcord/config/legcord/equicord.js"),
        home_path.join(".var/app/app.legcord.Legcord/config/legcord/vencord.js"),
        home_path.join(".var/app/app.legcord.Legcord/config/legcord/shelter.js"),
    ];

    for path in targets {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let base_content = if let Some(idx) = content.find("// === WAYCORD VOICE OVERLAY BRIDGE") {
                    &content[..idx]
                } else if let Some(idx) = content.find("// === DISCORD VOICE OVERLAY BRIDGE") {
                    &content[..idx]
                } else {
                    &content
                };
                let mut new_content = base_content.trim_end().to_string();
                new_content.push_str("\n\n// === WAYCORD VOICE OVERLAY BRIDGE v1 ===\n");
                new_content.push_str(CLIENT_HOOK_JS);
                new_content.push_str("\n// =======================================\n");
                if std::fs::write(&path, new_content).is_ok() {
                    println!("✨ WayCord bridge installed into {}!", path.display());
                }
            }
        }
    }

    // Also install as native Legcord User Plugin
    install_legcord_user_plugin(&home_path.join(".config/legcord"));
    install_legcord_user_plugin(&home_path.join(".var/app/app.legcord.Legcord/config/legcord"));

    cleanup_stock_discord(&home_path);

    let client_info = DiscordClientInfo::detect();
    if client_info.stock_discord_running || (client_info.stock_discord_installed && !client_info.vesktop_installed && !client_info.legcord_installed) {
        println!("💡 Standard Discord client detected: Stock Discord does not support third-party overlay bridges.");
        println!("   ⭐ Recommended: Install Vesktop (https://vesktop.dev) or Legcord (https://legcord.app)");
        println!("   🔧 Or patch Discord with Vencord: sh -c \"$(curl -sS https://raw.githubusercontent.com/Vendicated/VencordInstaller/main/install.sh)\"");
    }
}

pub fn install_legcord_user_plugin(legcord_dir: &Path) {
    if !legcord_dir.exists() {
        return;
    }

    let plugin_dir = legcord_dir.join("plugins/waycord");
    if let Err(e) = std::fs::create_dir_all(&plugin_dir) {
        eprintln!("Failed to create Legcord plugin dir: {e}");
        return;
    }

    let manifest_path = plugin_dir.join("manifest.json");
    let manifest_content = r#"{
  "id": "waycord",
  "name": "WayCord Voice Overlay",
  "version": "1.0.0",
  "description": "Discord Voice Overlay Bridge for WayCord",
  "renderer": "renderer.js"
}
"#;
    let _ = std::fs::write(&manifest_path, manifest_content);

    let renderer_path = plugin_dir.join("renderer.js");
    let mut renderer_content = String::from("// === WAYCORD LEGCORD PLUGIN ===\n");
    renderer_content.push_str(CLIENT_HOOK_JS);
    renderer_content.push_str("\n\nif (typeof module !== 'undefined' && module.exports) {\n    module.exports = { activate: () => {} };\n}\n");
    if std::fs::write(&renderer_path, renderer_content).is_ok() {
        println!("✨ WayCord plugin installed into Legcord at {}!", plugin_dir.display());
    }

    // Automatically enable plugin and bypass proxy for localhost in storage/settings.json
    let settings_path = legcord_dir.join("storage/settings.json");
    if settings_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.as_object_mut() {
                    let mut modified = false;
                    let plugin_states = obj.entry("pluginStates").or_insert_with(|| serde_json::json!({}));
                    if let Some(states_obj) = plugin_states.as_object_mut() {
                        if states_obj.get("waycord") != Some(&serde_json::Value::Bool(true)) {
                            states_obj.insert("waycord".to_string(), serde_json::Value::Bool(true));
                            modified = true;
                            println!("✨ Enabled WayCord plugin in Legcord settings!");
                        }
                    }
                    if let Some(bypass) = obj.get_mut("proxyBypassRules") {
                        if let Some(s) = bypass.as_str() {
                            if !s.contains("127.0.0.1") {
                                *bypass = serde_json::Value::String(format!("{s},127.0.0.1,localhost"));
                                modified = true;
                            }
                        }
                    }
                    if modified {
                        if let Ok(new_json) = serde_json::to_string_pretty(&json) {
                            let _ = std::fs::write(&settings_path, new_json);
                        }
                    }
                }
            }
        }
    }
}

fn cleanup_stock_discord(home_path: &PathBuf) {
    let discord_config_dirs = [
        home_path.join(".config/discord"),
        home_path.join(".config/discordcanary"),
        home_path.join(".config/discordptb"),
        home_path.join(".config/discorddevelopment"),
        home_path.join(".var/app/com.discordapp.Discord/config/discord"),
        home_path.join(".var/app/com.discordapp.DiscordCanary/config/discordcanary"),
    ];

    for config_dir in discord_config_dirs {
        if !config_dir.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&config_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("app-") || name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    let modules_dir = entry.path().join("modules");
                    if let Ok(mod_entries) = std::fs::read_dir(&modules_dir) {
                        for mod_entry in mod_entries.flatten() {
                            let mod_name = mod_entry.file_name().to_string_lossy().to_string();
                            if mod_name.starts_with("discord_desktop_core") {
                                let p1 = mod_entry.path().join("index.js");
                                let p2 = mod_entry.path().join("discord_desktop_core/index.js");
                                let index_path = if p2.exists() { p2 } else { p1 };
                                if index_path.exists() {
                                    let backup_path = index_path.with_extension("js.orig");
                                    if backup_path.exists() {
                                        let _ = std::fs::copy(&backup_path, &index_path);
                                        let _ = std::fs::remove_file(&backup_path);
                                    } else if let Ok(c) = std::fs::read_to_string(&index_path) {
                                        if c.contains("WAYCORD") {
                                            let _ = std::fs::write(&index_path, "module.exports = require('./core.asar');\n");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_installation() {
        install_universal_bridge();
    }

    #[test]
    fn test_discord_client_detection() {
        let info = DiscordClientInfo::detect();
        let _ = info.is_stock_discord_active_or_only_client();
        let _ = info.should_prompt_for_vesktop(false);
    }
}
