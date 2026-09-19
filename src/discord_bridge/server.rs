use std::{
    collections::HashMap,
    io::BufRead,
    net::TcpListener,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

use serde::Deserialize;
use tungstenite::Message;

use crate::{VoiceUser, avatar_cache::AvatarCache};
use super::client::DiscordClientInfo;
use super::hooks::install_universal_bridge;
use super::types::{get_initials, get_runtime_dir, BridgeSource, DiscordVoiceState, InboundMessage};

pub fn process_inbound_message(
    msg: InboundMessage,
    source: BridgeSource,
    state: &Arc<Mutex<DiscordVoiceState>>,
    cache: &AvatarCache,
) {
    if let Ok(mut lock) = state.lock() {
        let client_info = DiscordClientInfo::detect();
        let is_vesktop = source == BridgeSource::WebSocket
            || matches!(&msg, InboundMessage::Channel { client: Some(c), .. } | InboundMessage::Speaking { client: Some(c), .. } | InboundMessage::Ping { client: Some(c), .. } if c.to_lowercase() == "vesktop" || c.to_lowercase() == "vencord");

        if lock.enforce_vesktop_priority && (client_info.vesktop_running || lock.active_source == Some(BridgeSource::WebSocket)) && source != BridgeSource::WebSocket {
            return;
        }

        if let Some(current_source) = lock.active_source {
            if lock.connected && source < current_source {
                return;
            }
        }

        lock.active_source = Some(source);
        if is_vesktop || source == BridgeSource::WebSocket {
            lock.client_name = Some("Vesktop".to_string());
        } else if lock.client_name.is_none() {
            lock.client_name = Some("Legcord".to_string());
        }

        match msg {
            InboundMessage::Channel { channel, users, current_user, client } => {
                let user_count = users.len();
                if lock.channel_name != channel {
                    if let Some(ref name) = channel {
                        println!("🎙 Inbound Voice Channel ({}): {} ({} users in call)", lock.client_name.as_deref().unwrap_or("Discord"), name, user_count);
                    } else if lock.channel_name.is_some() {
                        println!("👋 Left Voice Channel");
                    }
                } else if channel.is_some() && lock.users.len() != user_count {
                    println!("🎙 Inbound Voice Channel ({}): {} ({} users in call)", lock.client_name.as_deref().unwrap_or("Discord"), channel.as_deref().unwrap_or(""), user_count);
                }

                lock.channel_name = channel;
                lock.connected = true;

                if let Some(c) = client {
                    if !c.trim().is_empty() {
                        if c.eq_ignore_ascii_case("vesktop") {
                            lock.client_name = Some("Vesktop".to_string());
                        } else if c.eq_ignore_ascii_case("legcord") {
                            lock.client_name = Some("Legcord".to_string());
                        } else {
                            lock.client_name = Some(c);
                        }
                    }
                }

                if let Some(user) = current_user {
                    if !user.trim().is_empty() && lock.current_user.as_deref() != Some(&user) {
                        println!("👤 Connected to Discord as {}", user);
                        lock.current_user = Some(user);
                    }
                }

                // Preserve speaking flags if user was already speaking
                let old_speaking: HashMap<String, (bool, f32)> = lock
                    .users
                    .iter()
                    .map(|u| (u.id.clone(), (u.is_speaking, u.volume_level)))
                    .collect();

                lock.users.clear();
                lock.user_map.clear();

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
                        cache.queue_download(u.id.clone(), url.clone());
                    }

                    let (was_speaking, old_vol) = old_speaking.get(&u.id).copied().unwrap_or((false, 0.0));
                    let is_speaking = u.speaking || was_speaking;
                    let volume_level = if is_speaking {
                        if old_vol > 0.0 { old_vol } else { 0.8 }
                    } else {
                        0.0
                    };

                    lock.users.push(VoiceUser {
                        id: u.id,
                        name: u.name,
                        avatar_initials: initials,
                        avatar_url,
                        is_speaking,
                        is_muted: u.muted,
                        is_deafened: u.deafened,
                        volume_level,
                    });
                }
            }
            InboundMessage::Speaking { user_id, speaking, .. } => {
                if let Some(&idx) = lock.user_map.get(&user_id) {
                    if let Some(u) = lock.users.get_mut(idx) {
                        u.is_speaking = speaking;
                        u.volume_level = if speaking { 0.8 } else { 0.0 };
                    }
                }
            }
            InboundMessage::Ping { current_user, client } => {
                lock.connected = true;
                if let Some(c) = client {
                    if !c.trim().is_empty() {
                        if c.eq_ignore_ascii_case("vesktop") {
                            lock.client_name = Some("Vesktop".to_string());
                        } else if c.eq_ignore_ascii_case("legcord") {
                            lock.client_name = Some("Legcord".to_string());
                        } else {
                            lock.client_name = Some(c);
                        }
                    }
                }
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

pub struct DiscordBridge {
    pub state: Arc<Mutex<DiscordVoiceState>>,
    pub avatar_cache: AvatarCache,
}

impl DiscordBridge {
    pub fn start() -> Self {
        let state = Arc::new(Mutex::new(DiscordVoiceState::default()));
        let avatar_cache = AvatarCache::new();
        let ws_connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let unix_connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        install_universal_bridge();

        // Unix Domain Socket (Secondary / Legcord fallback)
        {
            let state_clone = state.clone();
            let cache_clone = avatar_cache.clone();
            let ws_conns = ws_connections.clone();
            let unix_conns = unix_connections.clone();
            let sock_path = get_runtime_dir().join("waycord.sock");

            thread::spawn(move || {
                let _ = std::fs::remove_file(&sock_path);
                match std::os::unix::net::UnixListener::bind(&sock_path) {
                    Ok(listener) => {
                        println!("✨ WayCord native Unix domain socket listening at {}", sock_path.display());
                        for stream in listener.incoming() {
                            let stream = match stream {
                                Ok(s) => s,
                                Err(_) => continue,
                            };

                            let state_clone = state_clone.clone();
                            let cache_clone = cache_clone.clone();
                            let ws_conns = ws_conns.clone();
                            let unix_conns = unix_conns.clone();

                            thread::spawn(move || {
                                unix_conns.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                let info = DiscordClientInfo::detect();
                                let enforce = state_clone.lock().map(|l| l.enforce_vesktop_priority).unwrap_or(true);
                                let is_vesktop_busy = enforce && (info.vesktop_running || ws_conns.load(std::sync::atomic::Ordering::SeqCst) > 0);
                                if !is_vesktop_busy {
                                    println!("🟢 Connected to Discord Voice client via Unix domain socket!");
                                    if let Ok(mut lock) = state_clone.lock() {
                                        if lock.active_source.is_none() || lock.active_source == Some(BridgeSource::FallbackFile) {
                                            lock.connected = true;
                                            lock.active_source = Some(BridgeSource::UnixSocket);
                                            lock.client_name = Some("Legcord".to_string());
                                        }
                                    }
                                }

                                let reader = std::io::BufReader::new(stream);
                                for line in reader.lines() {
                                    let line = match line {
                                        Ok(l) => l,
                                        Err(_) => break,
                                    };
                                    if line.trim().is_empty() {
                                        continue;
                                    }
                                    let info = DiscordClientInfo::detect();
                                    let enforce = state_clone.lock().map(|l| l.enforce_vesktop_priority).unwrap_or(true);
                                    let is_vesktop_busy = enforce && (info.vesktop_running || ws_conns.load(std::sync::atomic::Ordering::SeqCst) > 0);
                                    if is_vesktop_busy {
                                        // Vesktop has priority - drop secondary messages
                                        continue;
                                    }
                                    if let Ok(msg) = serde_json::from_str::<InboundMessage>(&line) {
                                        process_inbound_message(msg, BridgeSource::UnixSocket, &state_clone, &cache_clone);
                                    }
                                }

                                if unix_conns.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) <= 1 {
                                    if let Ok(mut lock) = state_clone.lock() {
                                        if lock.active_source == Some(BridgeSource::UnixSocket) && ws_conns.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                                            lock.connected = false;
                                            lock.channel_name = None;
                                            for u in &mut lock.users {
                                                u.is_speaking = false;
                                                u.volume_level = 0.0;
                                            }
                                            lock.active_source = None;
                                        }
                                    }
                                }
                                println!("🔴 Disconnected from Unix domain socket.");
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to bind Unix domain socket at {}: {e}", sock_path.display());
                    }
                }
            });
        }

        // State File Watcher (Secondary / Lowest Fallback)
        {
            let state_clone = state.clone();
            let cache_clone = avatar_cache.clone();
            let ws_conns = ws_connections.clone();
            let state_path = get_runtime_dir().join("waycord_voice.json");
            let _ = std::fs::remove_file(&state_path);

            thread::spawn(move || {
                let mut last_mtime = None;
                let mut last_content = String::new();

                loop {
                    let client_info = DiscordClientInfo::detect();
                    let enforce = state_clone.lock().map(|l| l.enforce_vesktop_priority).unwrap_or(true);
                    // If Vesktop is running or connected via WebSocket, completely silence state file!
                    if enforce && (client_info.vesktop_running || ws_conns.load(std::sync::atomic::Ordering::SeqCst) > 0) {
                        let _ = std::fs::remove_file(&state_path);
                        last_mtime = None;
                        last_content.clear();
                        thread::sleep(std::time::Duration::from_millis(500));
                        continue;
                    }

                    if let Ok(metadata) = std::fs::metadata(&state_path) {
                        if let Ok(mtime) = metadata.modified() {
                            if last_mtime != Some(mtime) {
                                last_mtime = Some(mtime);
                                if let Ok(content) = std::fs::read_to_string(&state_path) {
                                    if content != last_content {
                                        last_content = content.clone();

                                        #[derive(Deserialize)]
                                        struct StateMeta {
                                            #[serde(default)]
                                            pid: Option<u32>,
                                        }

                                        let is_stale = if let Ok(meta) = serde_json::from_str::<StateMeta>(&content) {
                                            if let Some(pid) = meta.pid {
                                                !Path::new(&format!("/proc/{}", pid)).exists()
                                            } else {
                                                let info = DiscordClientInfo::detect();
                                                !info.legcord_running && !info.vesktop_running && !info.stock_discord_running
                                            }
                                        } else {
                                            false
                                        };

                                        if is_stale {
                                            let _ = std::fs::remove_file(&state_path);
                                            continue;
                                        }

                                        if let Ok(msg) = serde_json::from_str::<InboundMessage>(&content) {
                                            process_inbound_message(msg, BridgeSource::FallbackFile, &state_clone, &cache_clone);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            });
        }

        // WebSocket Listener (Primary - Vesktop / Vencord)
        {
            let state_clone = state.clone();
            let cache_clone = avatar_cache.clone();
            let ws_conns = ws_connections.clone();

            thread::spawn(move || {
                let listener = match TcpListener::bind("127.0.0.1:9876") {
                    Ok(l) => {
                        println!("Discord Voice Bridge WebSocket listening on ws://127.0.0.1:9876");
                        l
                    }
                    Err(_) => match TcpListener::bind("[::]:9876") {
                        Ok(l) => {
                            println!("Discord Voice Bridge WebSocket listening on ws://[::]:9876");
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
                    let ws_conns = ws_conns.clone();

                    thread::spawn(move || {
                        let _ = stream.set_read_timeout(None);
                        let mut ws = match tungstenite::accept(stream) {
                            Ok(ws) => {
                                ws_conns.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                println!("🟢 Connected to Vesktop / Discord Voice client via WebSocket!");
                                if let Ok(mut lock) = state_clone.lock() {
                                    lock.connected = true;
                                    lock.active_source = Some(BridgeSource::WebSocket);
                                    lock.client_name = Some("Vesktop".to_string());
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
                                        process_inbound_message(msg, BridgeSource::WebSocket, &state_clone, &cache_clone);
                                    }
                                }
                                Ok(Message::Close(_)) | Err(_) => {
                                    println!("🔴 Disconnected from Vesktop / Discord voice client.");
                                    if ws_conns.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) <= 1 {
                                        if let Ok(mut lock) = state_clone.lock() {
                                            if lock.active_source == Some(BridgeSource::WebSocket) {
                                                lock.connected = false;
                                                lock.channel_name = None;
                                                for u in &mut lock.users {
                                                    u.is_speaking = false;
                                                    u.volume_level = 0.0;
                                                }
                                                lock.active_source = None;
                                            }
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
        }

        Self {
            state,
            avatar_cache,
        }
    }
}
