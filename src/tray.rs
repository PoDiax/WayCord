use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use image::GenericImageView;
use ksni::blocking::TrayMethods;
use ksni::menu::*;

#[derive(Clone)]
pub struct TrayState {
    pub exit_requested: Arc<AtomicBool>,
    pub toggle_settings_requested: Arc<AtomicBool>,
    pub toggle_visibility_requested: Arc<AtomicBool>,
    pub reset_position_requested: Arc<AtomicBool>,
    pub reset_size_requested: Arc<AtomicBool>,
    pub toggle_only_speaking_requested: Arc<AtomicBool>,
    pub is_visible: Arc<AtomicBool>,
    pub only_speaking: Arc<AtomicBool>,
    pub discord_connected: Arc<AtomicBool>,
    pub channel_name: Arc<RwLock<String>>,
    pub current_user: Arc<RwLock<Option<String>>>,
}

impl Default for TrayState {
    fn default() -> Self {
        Self::new()
    }
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            exit_requested: Arc::new(AtomicBool::new(false)),
            toggle_settings_requested: Arc::new(AtomicBool::new(false)),
            toggle_visibility_requested: Arc::new(AtomicBool::new(false)),
            reset_position_requested: Arc::new(AtomicBool::new(false)),
            reset_size_requested: Arc::new(AtomicBool::new(false)),
            toggle_only_speaking_requested: Arc::new(AtomicBool::new(false)),
            is_visible: Arc::new(AtomicBool::new(true)),
            only_speaking: Arc::new(AtomicBool::new(false)),
            discord_connected: Arc::new(AtomicBool::new(false)),
            channel_name: Arc::new(RwLock::new(String::new())),
            current_user: Arc::new(RwLock::new(None)),
        }
    }

    pub fn start_tray(&self) -> Option<ksni::blocking::Handle<WayCordTray>> {
        let tray = WayCordTray {
            state: self.clone(),
        };
        match tray.spawn() {
            Ok(handle) => {
                println!("WayCord System Tray registered (StatusNotifierItem/DBus)");
                Some(handle)
            }
            Err(e) => {
                eprintln!("System tray icon could not be registered on DBus: {e}");
                None
            }
        }
    }
}

pub struct WayCordTray {
    pub state: TrayState,
}

impl ksni::Tray for WayCordTray {
    fn id(&self) -> String {
        "waycord".into()
    }

    fn title(&self) -> String {
        "WayCord".into()
    }

    fn icon_name(&self) -> String {
        "waycord".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        if let Ok(img) = image::load_from_memory(crate::WAYCORD_LOGO_BYTES) {
            let (width, height) = img.dimensions();
            let mut data = img.into_rgba8().into_vec();
            for pixel in data.chunks_exact_mut(4) {
                pixel.rotate_right(1); 
            }
            vec![ksni::Icon {
                width: width as i32,
                height: height as i32,
                data,
            }]
        } else {
            vec![]
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.state
            .toggle_visibility_requested
            .store(true, Ordering::SeqCst);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let connected = self.state.discord_connected.load(Ordering::Relaxed);
        let ch = self.state.channel_name.read().unwrap().clone();
        let current_user = self.state.current_user.read().unwrap().clone();

        let status_label = if connected {
            let user_tag = current_user.as_deref().unwrap_or("");
            if !ch.is_empty() && !ch.starts_with("Connecting") && !ch.starts_with("Connected to Discord") {
                if !user_tag.is_empty() {
                    format!("🟢 {} ({})", user_tag, ch)
                } else {
                    format!("🟢 Call: {}", ch)
                }
            } else if !user_tag.is_empty() {
                format!("🟢 Connected as {}", user_tag)
            } else {
                "🟢 Connected to Discord".to_string()
            }
        } else {
            "🟠 Discord: Waiting...".to_string()
        };

        let visible = self.state.is_visible.load(Ordering::Relaxed);
        let vis_label = if visible {
            "Hide Overlay"
        } else {
            "Show Overlay"
        };

        let exit_atomic = self.state.exit_requested.clone();
        let settings_atomic = self.state.toggle_settings_requested.clone();
        let vis_atomic = self.state.toggle_visibility_requested.clone();
        let reset_pos_atomic = self.state.reset_position_requested.clone();
        let reset_size_atomic = self.state.reset_size_requested.clone();
        let only_sp_atomic = self.state.toggle_only_speaking_requested.clone();

        let only_sp = self.state.only_speaking.load(Ordering::Relaxed);
        let only_sp_label = if only_sp {
            "✓ Show Only Speaking"
        } else {
            "Show Only Speaking"
        };

        let mut items: Vec<MenuItem<Self>> = vec![
            StandardItem {
                label: format!("WayCord v{}", env!("CARGO_PKG_VERSION")),
                enabled: false,
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: status_label,
                enabled: false,
                ..Default::default()
            }
            .into(),
        ];

        if !connected {
            items.push(MenuItem::Separator);
            items.push(
                StandardItem {
                    label: "Install Vesktop (Recommended)".into(),
                    activate: Box::new(|_| {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://vesktop.dev")
                            .spawn();
                    }),
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                StandardItem {
                    label: "Vencord Installer Script".into(),
                    activate: Box::new(|_| {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://vencord.dev/download")
                            .spawn();
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        items.extend([
            MenuItem::Separator,
            StandardItem {
                label: "Settings (Hold ALT)".into(),
                activate: Box::new(move |_| {
                    settings_atomic.store(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: vis_label.into(),
                activate: Box::new(move |_| {
                    vis_atomic.store(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: only_sp_label.into(),
                activate: Box::new(move |_| {
                    only_sp_atomic.store(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Reset Position".into(),
                activate: Box::new(move |_| {
                    reset_pos_atomic.store(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Reset Size".into(),
                activate: Box::new(move |_| {
                    reset_size_atomic.store(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit WayCord".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_| {
                    exit_atomic.store(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }
            .into(),
        ]);

        items
    }
}
