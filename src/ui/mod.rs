pub mod avatar;
pub mod bubble;
pub mod card;
pub mod settings;
pub mod transparent;

pub use avatar::{render_avatar, VoiceUser};
pub use bubble::render_bubble_overlay;
pub use card::render_card_overlay;
pub use settings::render_settings_window;
pub use transparent::render_transparent_overlay;

use egui::{Color32, CornerRadius, Margin, Stroke};

use crate::{
    avatar_cache::AvatarCache,
    presets::{find_active_monitor, MonitorBounds, OverlayConfig, OverlayStyle},
    theme::ThemePalette,
};

pub fn render_voice_overlay(
    ctx: &egui::Context,
    palette: &ThemePalette,
    users: &[VoiceUser],
    channel_name: &str,
    time_elapsed_secs: f64,
    config: &mut OverlayConfig,
    alt_held: bool,
    show_settings: &mut bool,
    screen_width: f32,
    screen_height: f32,
    discord_connected: bool,
    _force_demo: &mut bool,
    avatar_cache: &AvatarCache,
    monitors: &[MonitorBounds],
) {
    let active_monitor = find_active_monitor(monitors, config.x, config.y);
    let effective_w = match config.style {
        OverlayStyle::Transparent => (config.avatar_size + 160.0).max(180.0),
        OverlayStyle::Bubble => config.avatar_size + 24.0,
        OverlayStyle::Card => config.card_width,
    };
    let is_right_side = active_monitor.is_right_half(config.x + effective_w / 2.0);

    if alt_held {
        let banner_center_x = active_monitor.midpoint_x();
        egui::Area::new(egui::Id::new("alt_instruction_banner"))
            .fixed_pos(egui::pos2(banner_center_x, active_monitor.y + 14.0))
            .pivot(egui::Align2::CENTER_TOP)
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgba_premultiplied(15, 15, 25, 230))
                    .stroke(Stroke::new(1.2, palette.accent))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(14, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("[ALT] HELD:")
                                    .strong()
                                    .color(palette.accent),
                            );
                            let side_str = if is_right_side { "Right Docked" } else { "Left Docked" };
                            ui.label(
                                egui::RichText::new(format!(
                                    "Drag window to move • Resize ⌟ • ↺ Reset size • Click ⚙ settings • {} • Release ALT to lock",
                                    side_str
                                ))
                                .size(12.0)
                                .color(Color32::WHITE),
                            );
                        });
                    });
            });
    }

    let mut is_resizing = false;

    let area = egui::Area::new(egui::Id::new("discord_voice_hud_area"))
        .fixed_pos(egui::pos2(config.x, config.y))
        .constrain(false)
        .order(egui::Order::Foreground);
    let area = if alt_held {
        area.sense(egui::Sense::drag())
    } else {
        area
    };

    let area_resp = area.show(ctx, |ui| {
        match config.style {
            OverlayStyle::Transparent => {
                render_transparent_overlay(
                    ui,
                    palette,
                    users,
                    channel_name,
                    time_elapsed_secs,
                    config,
                    alt_held,
                    show_settings,
                    &active_monitor,
                    avatar_cache,
                    discord_connected,
                    &mut is_resizing,
                    is_right_side,
                );
            }
            OverlayStyle::Bubble => {
                render_bubble_overlay(
                    ui,
                    palette,
                    users,
                    channel_name,
                    time_elapsed_secs,
                    config,
                    alt_held,
                    show_settings,
                    &active_monitor,
                    avatar_cache,
                    discord_connected,
                    &mut is_resizing,
                    is_right_side,
                );
            }
            OverlayStyle::Card => {
                render_card_overlay(
                    ui,
                    palette,
                    users,
                    channel_name,
                    time_elapsed_secs,
                    config,
                    alt_held,
                    show_settings,
                    avatar_cache,
                    discord_connected,
                    &mut is_resizing,
                );
            }
        }
    });

    if alt_held {
        if area_resp.response.dragged() && !is_resizing {
            config.x += area_resp.response.drag_delta().x;
            config.y += area_resp.response.drag_delta().y;
            config.save();
        }
        if area_resp.response.hovered() && !is_resizing {
            ctx.set_cursor_icon(if area_resp.response.dragged() {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::Grab
            });
        }
    }

    let overlay_w = area_resp.response.rect.width().max(effective_w);
    let overlay_h = area_resp.response.rect.height().max(30.0);
    let mon = find_active_monitor(monitors, config.x + overlay_w / 2.0, config.y + overlay_h / 2.0);

    let max_x = (mon.right() - overlay_w).max(mon.x);
    let min_x = mon.x;
    let max_y = (mon.bottom() - overlay_h).max(mon.y);
    let min_y = mon.y;

    let clamped_x = config.x.clamp(min_x, max_x);
    let clamped_y = config.y.clamp(min_y, max_y);

    if (config.x - clamped_x).abs() > 0.5 || (config.y - clamped_y).abs() > 0.5 {
        config.x = clamped_x;
        config.y = clamped_y;
        config.save();
    }

    render_settings_window(
        ctx,
        palette,
        channel_name,
        config,
        alt_held,
        show_settings,
        screen_width,
        screen_height,
        discord_connected,
        &active_monitor,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeKind;

    #[test]
    fn test_transparent_overlay_size_and_screen_snapping() {
        let ctx = egui::Context::default();
        let palette = ThemePalette::from_kind(ThemeKind::Mocha);
        let users: Vec<VoiceUser> = vec![VoiceUser {
            id: "1".to_string(),
            name: "Podiax".to_string(),
            is_speaking: true,
            volume_level: 0.8,
            avatar_url: None,
            avatar_initials: "P".to_string(),
            is_muted: false,
            is_deafened: false,
        }];
        let channel_name = "Voice Call";
        let mut config = OverlayConfig::default();
        config.x = 2500.0;
        config.y = 48.0;
        config.style = OverlayStyle::Transparent;
        let mut show_settings = false;
        let mut force_demo = false;
        let avatar_cache = AvatarCache::new();
        let monitors = vec![MonitorBounds::new(0.0, 0.0, 2560.0, 1440.0)];

        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2560.0, 1440.0),
        ));

        let mut output = ctx.run_ui(raw_input.clone(), |ctx| {
            render_voice_overlay(
                ctx,
                &palette,
                &users,
                channel_name,
                0.0,
                &mut config,
                true, 
                &mut show_settings,
                2560.0,
                1440.0,
                true, 
                &mut force_demo,
                &avatar_cache,
                &monitors,
            );
        });
        output.textures_delta.clear();

        let mut output2 = ctx.run_ui(raw_input, |ctx| {
            render_voice_overlay(
                ctx,
                &palette,
                &users,
                channel_name,
                0.1,
                &mut config,
                true,
                &mut show_settings,
                2560.0,
                1440.0,
                true, 
                &mut force_demo,
                &avatar_cache,
                &monitors,
            );
        });
        println!("--- RECT SHAPES ---");
        for (i, s) in output2.shapes.iter().enumerate() {
            if let egui::epaint::Shape::Rect(r) = &s.shape {
                println!("Rect {}: rect={:?}, stroke={:?}", i, r.rect, r.stroke);
                assert!(
                    r.rect.height() < 150.0,
                    "Rect {} stretched excessively! height={}",
                    i,
                    r.rect.height()
                );
            }
        }
        output2.textures_delta.clear();

        let effective_w = (config.avatar_size + 160.0).max(180.0);
        assert!(config.x + effective_w <= 2560.0, "Overlay must not exceed screen width! config.x={}, w={}", config.x, effective_w);
        assert!(config.x >= 0.0, "Overlay must not be negative");

        // Test reset size
        config.avatar_size = 64.0;
        config.card_width = 400.0;
        config.reset_size();
        assert_eq!(config.avatar_size, 36.0);
        assert_eq!(config.card_width, 220.0);
    }

    #[test]
    fn test_empty_overlay_never_stretches() {
        let ctx = egui::Context::default();
        let palette = ThemePalette::from_kind(ThemeKind::Mocha);
        let users: Vec<VoiceUser> = Vec::new();
        let channel_name = "";
        let mut config = OverlayConfig::default();
        config.x = 2341.5;
        config.y = 48.0;
        config.style = OverlayStyle::Transparent;
        let mut show_settings = false;
        let mut force_demo = false;
        let avatar_cache = AvatarCache::new();
        let monitors = vec![MonitorBounds::new(0.0, 0.0, 2560.0, 1440.0)];

        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2560.0, 1440.0),
        ));

        let mut output1 = ctx.run_ui(raw_input.clone(), |ctx| {
            render_voice_overlay(
                ctx,
                &palette,
                &users,
                channel_name,
                0.0,
                &mut config,
                true, 
                &mut show_settings,
                2560.0,
                1440.0,
                true,
                &mut force_demo,
                &avatar_cache,
                &monitors,
            );
        });
        output1.textures_delta.clear();

        let mut output2 = ctx.run_ui(raw_input, |ctx| {
            render_voice_overlay(
                ctx,
                &palette,
                &users,
                channel_name,
                0.1,
                &mut config,
                true, 
                &mut show_settings,
                2560.0,
                1440.0,
                true,
                &mut force_demo,
                &avatar_cache,
                &monitors,
            );
        });

        for (i, s) in output2.shapes.iter().enumerate() {
            if let egui::epaint::Shape::Rect(r) = &s.shape {
                assert!(
                    r.rect.height() < 80.0,
                    "Shape {} height {} exceeds max allowable empty overlay height!",
                    i,
                    r.rect.height()
                );
            }
        }
        output2.textures_delta.clear();
    }

    #[test]
    fn test_explicit_row_height() {
        let ctx = egui::Context::default();
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2560.0, 1440.0),
        ));
        let mut out = ctx.run_ui(raw_input, |ctx| {
            egui::Area::new(egui::Id::new("test"))
                .fixed_pos(egui::pos2(2300.0, 48.0))
                .constrain(false)
                .show(ctx, |ui| {
                    egui::Frame::new().show(ui, |ui| {
                        ui.set_width(200.0);
                        let av_size = 36.0;
                        let f = egui::Frame::new()
                            .stroke(egui::Stroke::new(1.0, egui::Color32::GREEN))
                            .inner_margin(egui::Margin::symmetric(6, 4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let font_size = 14.0f32;
                                    let v_space = ((av_size - font_size) / 2.0 - 1.0f32).max(0.0f32);

                                    ui.vertical(|ui| {
                                        ui.add_space(v_space);
                                        ui.horizontal(|ui| {
                                            ui.colored_label(egui::Color32::RED, "deaf •");
                                            ui.label(egui::RichText::new("Podiax").size(font_size).strong());
                                        });
                                    });

                                    ui.add_space(4.0);
                                    let (r, _) = ui.allocate_exact_size(egui::vec2(av_size, av_size), egui::Sense::hover());
                                    println!("avatar rect: {:?}, center_y={}", r, r.center().y);
                                });
                            });
                        println!("f rect: {:?}", f.response.rect);
                        assert!(f.response.rect.height() < 60.0, "Frame height must be small! got {}", f.response.rect.height());
                    });
                });
        });
        out.textures_delta.clear();
    }
}
