pub mod avatar_cache;
pub mod discord_bridge;
pub mod overlay_window;
pub mod presets;
pub mod theme;
pub mod tray;
pub mod ui;
pub mod window_finder;

pub use avatar_cache::AvatarCache;
pub use discord_bridge::{DiscordBridge, DiscordVoiceState};
pub use presets::{find_active_monitor, MonitorBounds, OverlayConfig, OverlayStyle};
pub use theme::{
    apply_theme, contrast_text_for, init_fonts, selectable_pill, ThemeKind, ThemePalette,
};
pub use ui::{render_avatar, render_voice_overlay, VoiceUser};

pub static WAYCORD_LOGO_BYTES: &[u8] = include_bytes!("../assets/waycord.png");
