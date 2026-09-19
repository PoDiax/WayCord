use std::path::{Path, PathBuf};

use super::client::{cleanup_stock_discord, DiscordClientInfo};

const LEGCORD_MAIN_JS: &str = r#"// === WAYCORD LEGCORD MAIN PROCESS PLUGIN ===
const net = require("node:net");
const fs = require("node:fs");
const path = require("node:path");

exports.activate = function(api) {
    const runtimeDir = process.env.XDG_RUNTIME_DIR || "/tmp";
    const stateFile = path.join(runtimeDir, "waycord_voice.json");
    const socketPath = path.join(runtimeDir, "waycord.sock");

    let socketClient = null;
    let isConnecting = false;
    let reconnectTimer = null;
    let fullVoiceState = {
        type: "channel",
        channel: null,
        users: [],
        current_user: null
    };

    function connectSocket() {
        if (socketClient || isConnecting) return;
        isConnecting = true;

        try {
            const client = net.createConnection(socketPath, () => {
                isConnecting = false;
                socketClient = client;
                api.logger?.log?.("[WayCord] Connected to WayCord Unix domain socket at " + socketPath);
                // Immediately send current voice state upon connecting
                if (fullVoiceState && fullVoiceState.channel) {
                    try {
                        client.write(JSON.stringify(fullVoiceState) + "\n");
                    } catch (e) {}
                }
            });

            client.on("error", () => {
                isConnecting = false;
                socketClient = null;
                scheduleReconnect();
            });

            client.on("close", () => {
                isConnecting = false;
                socketClient = null;
                scheduleReconnect();
            });
        } catch (e) {
            isConnecting = false;
            socketClient = null;
            scheduleReconnect();
        }
    }

    function scheduleReconnect() {
        if (reconnectTimer) return;
        reconnectTimer = setTimeout(() => {
            reconnectTimer = null;
            connectSocket();
        }, 1500);
    }

    function writeStateFile(data) {
        try {
            const payload = Object.assign({}, data, {
                pid: process.pid,
                timestamp: Date.now()
            });
            const str = JSON.stringify(payload);
            const tmp = stateFile + ".tmp." + process.pid;
            fs.writeFileSync(tmp, str, "utf-8");
            fs.renameSync(tmp, stateFile);
        } catch (e) {
            try {
                fs.writeFileSync(stateFile, JSON.stringify(data), "utf-8");
            } catch (err) {}
        }
    }

    function sendToSocket(data) {
        if (socketClient) {
            try {
                socketClient.write(JSON.stringify(data) + "\n");
            } catch (e) {
                socketClient = null;
                scheduleReconnect();
            }
        } else {
            connectSocket();
        }
    }

    function handleRendererPayload(payload) {
        if (!payload || typeof payload !== "object") return;

        if (payload.type === "channel") {
            fullVoiceState = payload;
            writeStateFile(fullVoiceState);
            sendToSocket(fullVoiceState);
        } else if (payload.type === "speaking") {
            if (Array.isArray(fullVoiceState.users)) {
                for (const u of fullVoiceState.users) {
                    if (u.id === payload.userId) {
                        u.speaking = Boolean(payload.speaking);
                    }
                }
                writeStateFile(fullVoiceState);
            }
            sendToSocket(payload);
        } else if (payload.type === "ping") {
            if (payload.current_user) {
                fullVoiceState.current_user = payload.current_user;
                writeStateFile(fullVoiceState);
            }
            sendToSocket(payload);
        }
    }

    // Intercept Legcord Touchbar IPC from Discord renderer
    const ipcMain = api.electron?.ipcMain;
    if (ipcMain) {
        ipcMain.on("setVoiceTouchbar", (_event, data) => {
            if (data && data.__waycord) {
                handleRendererPayload(data.payload);
            }
        });
    }

    // Connect socket on startup
    connectSocket();

    function cleanup() {
        if (reconnectTimer) clearTimeout(reconnectTimer);
        if (socketClient) {
            try { socketClient.destroy(); } catch (e) {}
            socketClient = null;
        }
        try {
            if (fs.existsSync(stateFile)) {
                fs.unlinkSync(stateFile);
            }
        } catch (e) {}
    }

    process.on("exit", cleanup);
    api.onCleanup?.(cleanup);
};
"#;

const CLIENT_HOOK_JS: &str = r#"
(() => {
    if (window.__waycordBridge) return;
    window.__waycordBridge = true;
    console.log("[WayCord Client] Voice overlay bridge active");

    const isLegcord = Boolean(window.legcord?.touchbar?.setVoiceTouchbar);

    // Ensure Vencord / Equicord instance is exposed to window
    try {
        const localVc = (typeof Vencord !== 'undefined' ? Vencord : null)
            || (typeof Equicord !== 'undefined' ? Equicord : null);
        if (localVc) {
            if (!window.Vencord) window.Vencord = localVc;
            if (!window.Equicord) window.Equicord = localVc;
        }
    } catch (e) {}

    function getDiscordModules() {
        // 1. Primary & Fast: Vencord / Equicord Common stores
        const vc = window.Vencord || window.Equicord;
        const common = vc?.Webpack?.Common;
        if (common?.FluxDispatcher &&
            common?.VoiceStateStore &&
            common?.SelectedChannelStore &&
            common?.ChannelStore &&
            common?.UserStore) {
            return {
                Dispatcher: common.FluxDispatcher,
                VoiceStateStore: common.VoiceStateStore,
                SelectedChannelStore: common.SelectedChannelStore,
                ChannelStore: common.ChannelStore,
                UserStore: common.UserStore,
                RTCConnectionStore: common.RTCConnectionStore,
            };
        }

        // 2. Shelter stores fallback (for Legcord with Shelter)
        if (window.shelter?.flux?.dispatcher && window.shelter?.flux?.stores) {
            try {
                const stores = typeof window.shelter.flux.stores === 'function' ? window.shelter.flux.stores() : window.shelter.flux.stores;
                const getStore = s => Array.isArray(s) ? s[0] : s;
                const vss = getStore(stores?.VoiceStateStore);
                const scs = getStore(stores?.SelectedChannelStore);
                const cs = getStore(stores?.ChannelStore);
                const us = getStore(stores?.UserStore);
                if (vss && scs && cs && us) {
                    return {
                        Dispatcher: window.shelter.flux.dispatcher,
                        VoiceStateStore: vss,
                        SelectedChannelStore: scs,
                        ChannelStore: cs,
                        UserStore: us,
                        RTCConnectionStore: getStore(stores?.RTCConnectionStore),
                    };
                }
            } catch (e) {}
        }

        return null;
    }

    let modules = null;
    let ws = null;
    let reconnectTimer = null;
    const speakingUsers = new Set();

    function sendPayload(payload) {
        if (isLegcord && window.legcord?.touchbar?.setVoiceTouchbar) {
            try {
                window.legcord.touchbar.setVoiceTouchbar({
                    __waycord: true,
                    payload: payload
                });
            } catch (e) {}
        }

        if (ws && ws.readyState === WebSocket.OPEN) {
            try {
                ws.send(JSON.stringify(payload));
            } catch (e) {}
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
        if (!modules) return;

        try {
            const { VoiceStateStore, SelectedChannelStore, ChannelStore, UserStore, RTCConnectionStore } = modules;
            const curUser = typeof UserStore.getCurrentUser === 'function' ? UserStore.getCurrentUser() : null;
            const currentUserName = curUser ? (curUser.globalName || curUser.global_name || curUser.username || null) : null;

            let voiceChannelId = null;
            try {
                if (typeof SelectedChannelStore.getVoiceChannelId === 'function') {
                    voiceChannelId = SelectedChannelStore.getVoiceChannelId();
                }
            } catch (e) {}

            if (!voiceChannelId && curUser?.id && typeof VoiceStateStore.getVoiceStateForUser === 'function') {
                try {
                    const st = VoiceStateStore.getVoiceStateForUser(curUser.id);
                    if (st?.channelId) voiceChannelId = st.channelId;
                } catch (e) {}
            }

            if (!voiceChannelId && RTCConnectionStore && typeof RTCConnectionStore.getChannelId === 'function') {
                try {
                    const rtcId = RTCConnectionStore.getChannelId();
                    if (rtcId) voiceChannelId = rtcId;
                } catch (e) {}
            }

            if (!voiceChannelId) {
                sendPayload({
                    type: "channel",
                    channel: null,
                    users: [],
                    current_user: currentUserName,
                    client: isLegcord ? "legcord" : "vesktop"
                });
                return;
            }

            const chan = typeof ChannelStore.getChannel === 'function' ? ChannelStore.getChannel(voiceChannelId) : null;
            let rawStates = null;
            if (typeof VoiceStateStore.getVoiceStatesForChannel === 'function') {
                rawStates = VoiceStateStore.getVoiceStatesForChannel(voiceChannelId);
            } else if (typeof VoiceStateStore.getVoiceStates === 'function' && chan?.guild_id) {
                rawStates = VoiceStateStore.getVoiceStates(chan.guild_id);
            }

            const states = rawStates || {};
            let users = Object.values(states)
                .filter(st => !st.channelId || st.channelId === voiceChannelId)
                .map(st => {
                    const uid = st.userId || st.user_id;
                    const user = (typeof UserStore.getUser === 'function' ? UserStore.getUser(uid) : null)
                        || (curUser && curUser.id === uid ? curUser : null)
                        || {};
                    const avatarUrl = getAvatarUrl(user, uid, chan?.guild_id);
                    return {
                        id: uid,
                        name: user.globalName || user.global_name || user.username || "Unknown",
                        avatar: avatarUrl,
                        speaking: speakingUsers.has(uid),
                        muted: Boolean(st.mute || st.selfMute || st.self_mute),
                        deafened: Boolean(st.deaf || st.selfDeaf || st.self_deaf),
                    };
                });

            // Ensure current user is in the list if connected to channel
            if (curUser && curUser.id && !users.some(u => u.id === curUser.id)) {
                users.unshift({
                    id: curUser.id,
                    name: curUser.globalName || curUser.global_name || curUser.username || "Me",
                    avatar: getAvatarUrl(curUser, curUser.id, chan?.guild_id),
                    speaking: speakingUsers.has(curUser.id),
                    muted: false,
                    deafened: false,
                });
            }

            let channelTitle = "Voice Call";
            if (chan) {
                if (chan.name) {
                    channelTitle = chan.name;
                } else if (chan.rawRecipients && chan.rawRecipients.length > 0) {
                    channelTitle = chan.rawRecipients.map(r => r.global_name || r.globalName || r.username).join(", ");
                } else if (chan.recipients && chan.recipients.length > 0) {
                    channelTitle = chan.recipients.map(id => {
                        const u = typeof UserStore.getUser === 'function' ? UserStore.getUser(id) : null;
                        return u ? (u.globalName || u.username) : "User";
                    }).join(", ");
                }
            }

            sendPayload({
                type: "channel",
                channel: channelTitle,
                users,
                current_user: currentUserName,
                client: isLegcord ? "legcord" : "vesktop"
            });
        } catch (err) {
            console.error("[WayCord] Error sending state:", err);
        }
    }

    function connectWs() {
        if (ws && (ws.readyState === WebSocket.CONNECTING || ws.readyState === WebSocket.OPEN)) {
            return;
        }
        if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
        }

        try {
            ws = new WebSocket("ws://127.0.0.1:9876");
            ws.onopen = () => {
                console.log("[WayCord Client] Connected to overlay server!");
                sendFullState();
            };
            ws.onclose = () => {
                ws = null;
                if (!reconnectTimer) {
                    reconnectTimer = setTimeout(() => {
                        reconnectTimer = null;
                        connectWs();
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
                    connectWs();
                }, 2000);
            }
        }
    }

    function init() {
        modules = getDiscordModules();
        if (!modules) {
            setTimeout(init, 1000);
            return;
        }

        const { Dispatcher, SelectedChannelStore, VoiceStateStore } = modules;
        console.log("[WayCord Client] Connected to Discord Flux modules successfully!");

        try {
            Dispatcher.subscribe("SPEAKING", (ev) => {
                const isSp = !!(ev.speakingFlags & 1);
                if (isSp) {
                    speakingUsers.add(ev.userId);
                } else {
                    speakingUsers.delete(ev.userId);
                }
                sendPayload({
                    type: "speaking",
                    userId: ev.userId,
                    speaking: isSp,
                    client: isLegcord ? "legcord" : "vesktop"
                });
            });
        } catch (e) {}

        const events = [
            "VOICE_STATE_UPDATES",
            "VOICE_CHANNEL_SELECT",
            "CHANNEL_SELECT",
            "RTC_CONNECTION_STATE",
            "AUDIO_TOGGLE_SELF_MUTE",
            "AUDIO_TOGGLE_SELF_DEAF",
            "CONNECTION_OPEN",
            "CURRENT_USER_UPDATE"
        ];
        for (const ev of events) {
            try {
                Dispatcher.subscribe(ev, () => sendFullState());
            } catch (e) {}
        }

        try {
            SelectedChannelStore?.addChangeListener?.(() => sendFullState());
            VoiceStateStore?.addChangeListener?.(() => sendFullState());
        } catch (e) {}

        setInterval(() => {
            sendFullState();
            if (!ws || ws.readyState === WebSocket.CLOSED) {
                connectWs();
            }
        }, 2000);

        connectWs();
    }

    setTimeout(init, 1000);
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
                let base_content = if let Some(idx) = content.find("// === WAYCORD") {
                    &content[..idx]
                } else if let Some(idx) = content.find("// === DISCORD VOICE OVERLAY BRIDGE") {
                    &content[..idx]
                } else {
                    &content
                };
                let mut new_content = base_content.trim_end().to_string();
                new_content.push_str("\n\n// === WAYCORD VOICE OVERLAY BRIDGE ===\n");
                new_content.push_str("try { if (typeof Vencord !== 'undefined') { window.Vencord = Vencord; window.Equicord = Vencord; } } catch(e) {}\n");
                new_content.push_str(CLIENT_HOOK_JS);
                new_content.push_str("\n// =======================================\n");
                if std::fs::write(&path, new_content).is_ok() {
                    println!("✨ WayCord bridge installed into {}!", path.display());
                }
            }
        }
    }

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
  "main": "main.js",
  "renderer": "renderer.js"
}
"#;
    let _ = std::fs::write(&manifest_path, manifest_content);

    let main_path = plugin_dir.join("main.js");
    let _ = std::fs::write(&main_path, LEGCORD_MAIN_JS);

    let renderer_path = plugin_dir.join("renderer.js");
    let mut renderer_content = String::from("// === WAYCORD LEGCORD RENDERER PLUGIN ===\n");
    renderer_content.push_str(CLIENT_HOOK_JS);
    renderer_content.push_str("\n\nif (typeof module !== 'undefined' && module.exports) {\n    module.exports = { activate: () => {} };\n}\n");
    if std::fs::write(&renderer_path, renderer_content).is_ok() {
        println!("✨ WayCord native IPC plugin installed into Legcord at {}!", plugin_dir.display());
    }

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
                    if obj.get("extendedPluginAbilities") != Some(&serde_json::Value::Bool(true)) {
                        obj.insert("extendedPluginAbilities".to_string(), serde_json::Value::Bool(true));
                        modified = true;
                    }
                    let no_updates = obj.entry("noBundleUpdates").or_insert_with(|| serde_json::json!([]));
                    if let Some(arr) = no_updates.as_array_mut() {
                        for mod_name in ["equicord", "vencord", "shelter"] {
                            let val = serde_json::Value::String(mod_name.to_string());
                            if !arr.contains(&val) {
                                arr.push(val);
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
