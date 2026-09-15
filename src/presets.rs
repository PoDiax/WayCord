use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::ThemeKind;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum OverlayStyle {
    Transparent,
    Bubble,
    Card,
}

impl Default for OverlayStyle {
    fn default() -> Self {
        OverlayStyle::Transparent
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub x: f32,
    pub y: f32,
    pub avatar_size: f32,
    #[serde(default)]
    pub style: OverlayStyle,
    #[serde(default = "default_tint_alpha")]
    pub bg_tint_alpha: u8,
    pub theme: ThemeKind,
    pub opacity: f32,
    #[serde(default = "default_true")]
    pub show_names: bool,
    #[serde(default = "default_false")]
    pub only_speaking: bool,
    #[serde(default = "default_card_width")]
    pub card_width: f32,
}

fn default_tint_alpha() -> u8 {
    80
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_card_width() -> f32 {
    220.0
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            x: 32.0,
            y: 48.0,
            avatar_size: 36.0,
            style: OverlayStyle::Transparent,
            bg_tint_alpha: 80,
            theme: ThemeKind::Mocha,
            opacity: 0.90,
            show_names: true,
            only_speaking: false,
            card_width: 220.0,
        }
    }
}

impl OverlayConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(&home).join(".config/waycord");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<Self>(&content) {
                return config;
            }
        }
        let default = Self::default();
        default.save();
        default
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn snap_left(&mut self) {
        self.x = 24.0;
        self.save();
    }

    pub fn snap_left_on_monitor(&mut self, monitor: &MonitorBounds) {
        self.x = monitor.x + 24.0;
        self.save();
    }

    pub fn snap_right(&mut self, screen_width: f32) {
        self.snap_right_on_monitor(&MonitorBounds::new(0.0, 0.0, screen_width, 1440.0));
    }

    pub fn snap_right_on_monitor(&mut self, monitor: &MonitorBounds) {
        let w = match self.style {
            OverlayStyle::Transparent => (self.avatar_size + 160.0).max(180.0),
            OverlayStyle::Bubble => self.avatar_size + 24.0,
            OverlayStyle::Card => self.card_width,
        };
        self.x = (monitor.right() - w - 24.0).max(monitor.x + 24.0);
        self.save();
    }

    pub fn reset_size(&mut self) {
        self.avatar_size = 36.0;
        self.card_width = 220.0;
        self.save();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MonitorBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl MonitorBounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn midpoint_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    pub fn contains_x(&self, px: f32) -> bool {
        px >= self.x && px < self.right()
    }

    pub fn is_right_half(&self, px: f32) -> bool {
        px >= self.midpoint_x()
    }
}

pub fn find_active_monitor(monitors: &[MonitorBounds], x: f32, y: f32) -> MonitorBounds {
    if monitors.is_empty() {
        return MonitorBounds::new(0.0, 0.0, 2560.0, 1440.0);
    }
    for m in monitors {
        if m.contains_point(x, y) {
            return *m;
        }
    }
    for m in monitors {
        if m.contains_x(x) {
            return *m;
        }
    }
    monitors
        .iter()
        .min_by(|a, b| {
            let dist_a = (a.midpoint_x() - x).abs();
            let dist_b = (b.midpoint_x() - x).abs();
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
        .unwrap_or(monitors[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_monitor_detection_and_snap() {
        let monitors = vec![
            MonitorBounds::new(0.0, 0.0, 1920.0, 1080.0), 
            MonitorBounds::new(1920.0, 0.0, 2560.0, 1440.0),
        ];

        let m1_left = find_active_monitor(&monitors, 100.0, 100.0);
        assert_eq!(m1_left.x, 0.0);
        assert!(!m1_left.is_right_half(100.0));

        let m1_right = find_active_monitor(&monitors, 1200.0, 100.0);
        assert_eq!(m1_right.x, 0.0);
        assert!(m1_right.is_right_half(1200.0));

        let m2_left = find_active_monitor(&monitors, 2200.0, 200.0);
        assert_eq!(m2_left.x, 1920.0);
        assert!(!m2_left.is_right_half(2200.0));

        let m2_right = find_active_monitor(&monitors, 3600.0, 200.0);
        assert_eq!(m2_right.x, 1920.0);
        assert!(m2_right.is_right_half(3600.0));

        let mut cfg = OverlayConfig::default();
        cfg.snap_left_on_monitor(&m2_right);
        assert_eq!(cfg.x, 1920.0 + 24.0);

        cfg.snap_right_on_monitor(&m2_right);
        let expected_w = cfg.avatar_size + 160.0;
        assert_eq!(cfg.x, 4480.0 - expected_w - 24.0);
        assert!(m2_right.is_right_half(cfg.x + expected_w / 2.0));
    }
}

