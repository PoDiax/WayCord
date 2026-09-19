use std::path::{Path, PathBuf};

pub fn get_autostart_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    config_dir.join("autostart/waycord.desktop")
}

pub fn is_autostart_enabled() -> bool {
    let user_path = get_autostart_path();
    if user_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&user_path) {
            if content.lines().any(|l| l.trim().eq_ignore_ascii_case("hidden=true")) {
                return false;
            }
        }
        return true;
    }
    Path::new("/etc/xdg/autostart/waycord.desktop").exists()
}

pub fn set_autostart(enabled: bool) -> std::io::Result<()> {
    let path = get_autostart_path();
    if enabled {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let system_desktop = Path::new("/usr/share/applications/waycord.desktop");
        let is_system_installed = Path::new("/usr/bin/waycord").exists()
            || Path::new("/usr/local/bin/waycord").exists();

        if system_desktop.exists() && is_system_installed {
            let _ = std::fs::copy(system_desktop, &path);
            return Ok(());
        }

        let exec_cmd = if is_system_installed {
            "waycord".to_string()
        } else if let Ok(exe) = std::env::current_exe() {
            if exe.starts_with("/usr/bin") || exe.starts_with("/usr/local/bin") {
                "waycord".to_string()
            } else {
                exe.to_string_lossy().to_string()
            }
        } else {
            "waycord".to_string()
        };

        let desktop_entry = format!(
            "[Desktop Entry]\n\
            Name=WayCord\n\
            GenericName=Discord Voice Overlay\n\
            Comment=Lightweight Discord voice overlay for Linux\n\
            Exec={}\n\
            Icon=waycord\n\
            Terminal=false\n\
            Type=Application\n\
            Categories=AudioVideo;Audio;\n\
            Keywords=discord;overlay;voice;hud;gaming;vesktop;\n\
            StartupNotify=false\n\
            X-GNOME-Autostart-enabled=true\n",
            exec_cmd
        );
        std::fs::write(&path, desktop_entry)?;
    } else {
        if Path::new("/etc/xdg/autostart/waycord.desktop").exists() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let disable_entry = "[Desktop Entry]\nType=Application\nName=WayCord\nHidden=true\n";
            std::fs::write(&path, disable_entry)?;
        } else if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
    Ok(())
}
