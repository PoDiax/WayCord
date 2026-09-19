use std::path::{Path, PathBuf};

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
            || Path::new("/usr/bin/discord").exists()
            || Path::new("/usr/bin/discord-canary").exists()
            || Path::new("/usr/bin/discord-ptb").exists();

        let vesktop_dirs = [
            home_path.join(".config/vesktop"),
            home_path.join(".var/app/dev.vencord.Vesktop"),
        ];
        let vesktop_installed = vesktop_dirs.iter().any(|p| p.exists())
            || Path::new("/usr/bin/vesktop").exists()
            || Path::new("/usr/lib/vesktop").exists();

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
            || Path::new("/usr/bin/legcord").exists()
            || Path::new("/usr/lib/legcord").exists();

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

        let exe_name = std::fs::read_link(entry.path().join("exe"))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));

        let cmdline = std::fs::read_to_string(entry.path().join("cmdline"))
            .map(|s| s.replace('\0', " ").to_lowercase())
            .unwrap_or_default();

        let matches_target = target_names.iter().any(|name| {
            let lower = name.to_lowercase();
            if let Some(ref exe) = exe_name {
                if exe == &lower || exe.starts_with(&format!("{}-", lower)) || exe.ends_with(&format!("/{}", lower)) {
                    return true;
                }
            }
            if cmdline.contains(&format!("/{}", lower)) || cmdline.starts_with(&format!("{} ", lower)) {
                return true;
            }
            false
        });

        if !matches_target {
            continue;
        }

        let matches_exclude = exclude_names.iter().any(|name| {
            let lower = name.to_lowercase();
            if let Some(ref exe) = exe_name {
                if exe.contains(&lower) {
                    return true;
                }
            }
            cmdline.contains(&lower)
        });

        if !matches_exclude {
            return true;
        }
    }

    false
}

pub fn cleanup_stock_discord(home_path: &Path) {
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
