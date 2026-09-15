use egui::{Color32, CornerRadius, Margin, Stroke, Vec2};

use crate::{
    avatar_cache::AvatarCache,
    presets::OverlayConfig,
    theme::ThemePalette,
    ui::avatar::{render_avatar, VoiceUser},
};

pub fn render_card_overlay(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    users: &[VoiceUser],
    channel_name: &str,
    time_elapsed_secs: f64,
    config: &mut OverlayConfig,
    alt_held: bool,
    show_settings: &mut bool,
    avatar_cache: &AvatarCache,
    discord_connected: bool,
    is_resizing: &mut bool,
) {
    let frame_stroke = if alt_held {
        Stroke::new(1.5, palette.accent)
    } else {
        Stroke::new(1.0, palette.border_subtle.gamma_multiply(0.7))
    };
    let frame_fill = palette.background.gamma_multiply(config.opacity);

    egui::Frame::new()
        .fill(frame_fill)
        .stroke(frame_stroke)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(config.card_width.max(180.0));

            ui.horizontal(|ui| {
                let dot_color = if discord_connected {
                    palette.accent
                } else {
                    let pulse = ((time_elapsed_secs * 4.0).sin() * 0.5 + 0.5) as f32;
                    Color32::from_rgba_premultiplied(
                        250,
                        180,
                        50,
                        (160.0 + pulse * 95.0) as u8,
                    )
                };
                ui.label(
                    egui::RichText::new("●")
                        .size(12.0)
                        .color(dot_color),
                );
                ui.label(
                    egui::RichText::new(channel_name)
                        .size(13.0)
                        .strong()
                        .color(palette.text_primary),
                );

                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if alt_held {
                            let sp_icon = if config.only_speaking { "🗣" } else { "👥" };
                            let sp_tip = if config.only_speaking {
                                "Showing only speaking users (Click to show all)"
                            } else {
                                "Showing all users (Click to show only speaking)"
                            };

                            if ui
                                .button(egui::RichText::new("\u{f013}").size(12.0))
                                .on_hover_text("Settings")
                                .clicked()
                            {
                                *show_settings = !*show_settings;
                            }
                            if ui
                                .button(egui::RichText::new(sp_icon).size(11.0))
                                .on_hover_text(sp_tip)
                                .clicked()
                            {
                                config.only_speaking = !config.only_speaking;
                                config.save();
                            }
                            if ui
                                .button(egui::RichText::new("↺").size(11.0))
                                .on_hover_text("Reset Size (Default)")
                                .clicked()
                            {
                                config.reset_size();
                            }
                        } else {
                            let count_text = format!("{} in voice", users.len());
                            ui.label(
                                egui::RichText::new(count_text)
                                    .size(11.0)
                                    .color(palette.text_muted),
                            );
                        }
                    },
                );
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            let users_to_display: Vec<_> = users
                .iter()
                .filter(|u| !config.only_speaking || u.is_speaking)
                .collect();

            if users_to_display.is_empty() {
                let empty_msg = if !discord_connected {
                    "Waiting for Discord (Vesktop / Vencord)..."
                } else if users.is_empty() {
                    "Waiting for voice call..."
                } else {
                    "No one is speaking..."
                };
                ui.label(
                    egui::RichText::new(empty_msg)
                        .size(12.0)
                        .color(palette.text_muted),
                );

                if !discord_connected && alt_held {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Recommended:")
                                .size(10.5)
                                .color(palette.accent),
                        );
                        ui.hyperlink_to("Install Vesktop", "https://vesktop.dev");
                    });
                }
            }

            for user in users_to_display {
                let avatar_size = config.avatar_size.clamp(20.0, 56.0);
                let radius = avatar_size / 2.0;
                let v_space = ((avatar_size - 28.0) / 2.0).max(0.0f32);
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::splat(avatar_size),
                        egui::Sense::hover(),
                    );
                    let center = rect.center();
                    let painter = ui.painter();

                    if user.is_speaking {
                        let pulse = ((time_elapsed_secs * 6.0).sin() * 0.5 + 0.5) as f32;
                        let ring_radius = radius + 2.0 + pulse * 2.0;
                        let ring_color = Color32::from_rgba_premultiplied(
                            35,
                            165,
                            90,
                            (140.0 + pulse * 115.0) as u8,
                        );
                        painter.circle_stroke(
                            center,
                            ring_radius,
                            Stroke::new(2.0, ring_color),
                        );
                    }

                    render_avatar(ui.ctx(), painter, palette, user, center, radius, avatar_cache);

                    ui.add_space(4.0);

                    ui.vertical(|ui| {
                        ui.add_space(v_space);
                        ui.label(
                            egui::RichText::new(&user.name)
                                .size(13.0)
                                .strong()
                                .color(if user.is_speaking {
                                    Color32::WHITE
                                } else {
                                    palette.text_primary
                                }),
                        );

                        ui.horizontal(|ui| {
                            if user.is_speaking {
                                let bar_width = 3.0;
                                let bar_count = 5;
                                let current_vol = user.volume_level.clamp(0.0, 1.0);
                                let active_bars = (current_vol * bar_count as f32).ceil() as usize;

                                for i in 0..bar_count {
                                    let bar_height = 4.0 + (i as f32) * 2.2;
                                    let is_active = i < active_bars;
                                    let color = if is_active {
                                        palette.accent
                                    } else {
                                        palette.surface_elevated
                                    };

                                    let (b_rect, _) = ui.allocate_exact_size(
                                        Vec2::new(bar_width, bar_height),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        b_rect,
                                        CornerRadius::same(1),
                                        color,
                                    );
                                }
                            } else if user.is_deafened {
                                ui.label(
                                    egui::RichText::new("Deafened")
                                        .size(10.0)
                                        .color(palette.danger),
                                );
                            } else if user.is_muted {
                                ui.label(
                                    egui::RichText::new("Muted")
                                        .size(10.0)
                                        .color(palette.text_muted),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("Connected")
                                        .size(10.0)
                                        .color(palette.text_muted),
                                );
                            }
                        });
                    });
                });
                ui.add_space(6.0);
            }

            if alt_held {
                ui.horizontal(|ui| {
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let (r_rect, r_resp) = ui.allocate_exact_size(
                                Vec2::splat(14.0),
                                egui::Sense::drag(),
                            );
                            ui.painter().text(
                                r_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "⌟",
                                egui::FontId::proportional(14.0),
                                palette.accent,
                            );
                            if r_resp.hovered() || r_resp.dragged() {
                                *is_resizing = true;
                            }
                            if r_resp.dragged() {
                                config.card_width = (config.card_width + r_resp.drag_delta().x)
                                    .clamp(180.0, 480.0);
                                config.save();
                            }
                        },
                    );
                });
            }
        });
}
