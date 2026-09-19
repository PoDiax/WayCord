use egui::{Color32, CornerRadius, Margin, Stroke, Vec2};

use crate::{
    avatar_cache::AvatarCache,
    presets::{MonitorBounds, OverlayConfig},
    theme::ThemePalette,
    ui::avatar::{render_avatar, VoiceUser},
};

pub fn render_bubble_overlay(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    users: &[VoiceUser],
    channel_name: &str,
    time_elapsed_secs: f64,
    config: &mut OverlayConfig,
    alt_held: bool,
    show_settings: &mut bool,
    active_monitor: &MonitorBounds,
    avatar_cache: &AvatarCache,
    discord_connected: bool,
    is_resizing: &mut bool,
    is_right_side: bool,
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
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::symmetric(8, 8))
        .show(ui, |ui| {
            let users_to_display: Vec<_> = users
                .iter()
                .filter(|u| !config.only_speaking || u.is_speaking)
                .collect();

            if alt_held {
                let sp_icon = if config.only_speaking { "🗣" } else { "👥" };
                let sp_tip = if config.only_speaking {
                    "Showing only speaking users (Click to show all)"
                } else {
                    "Showing all users (Click to show only speaking)"
                };

                ui.horizontal(|ui| {
                    if is_right_side {
                        if ui
                            .button(egui::RichText::new("◀").size(11.0))
                            .on_hover_text("Snap Left")
                            .clicked()
                        {
                            config.snap_left_on_monitor(active_monitor);
                        }
                        if ui
                            .button(egui::RichText::new("▶").size(11.0))
                            .on_hover_text("Snap Right")
                            .clicked()
                        {
                            config.snap_right_on_monitor(active_monitor);
                        }
                        if ui
                            .button(egui::RichText::new("↺").size(11.0))
                            .on_hover_text("Reset Size (Default 36px)")
                            .clicked()
                        {
                            config.reset_size();
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
                            .button(egui::RichText::new("\u{f013}").size(12.0))
                            .on_hover_text("Open Settings")
                            .clicked()
                        {
                            *show_settings = !*show_settings;
                        }
                    } else {
                        if ui
                            .button(egui::RichText::new("\u{f013}").size(12.0))
                            .on_hover_text("Open Settings")
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
                            .on_hover_text("Reset Size (Default 36px)")
                            .clicked()
                        {
                            config.reset_size();
                        }
                        if ui
                            .button(egui::RichText::new("◀").size(11.0))
                            .on_hover_text("Snap Left")
                            .clicked()
                        {
                            config.snap_left_on_monitor(active_monitor);
                        }
                        if ui
                            .button(egui::RichText::new("▶").size(11.0))
                            .on_hover_text("Snap Right")
                            .clicked()
                        {
                            config.snap_right_on_monitor(active_monitor);
                        }
                    }
                });
                ui.add_space(3.0);
            }

            if users_to_display.is_empty() {
                if !channel_name.is_empty() && users.is_empty() {
                    let av_size = config.avatar_size;
                    let v_space = ((av_size - 18.0) / 2.0).max(0.0f32);
                    if !discord_connected {
                        let pulse = ((time_elapsed_secs * 4.0).sin() * 0.5 + 0.5) as f32;
                        let dot_color = Color32::from_rgba_premultiplied(
                            250,
                            180,
                            50,
                            (160.0 + pulse * 95.0) as u8,
                        );

                        ui.horizontal(|ui| {
                            if is_right_side {
                                ui.vertical(|ui| {
                                    ui.add_space(v_space);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 220))
                                        .stroke(Stroke::new(
                                            1.0,
                                            Color32::from_rgba_premultiplied(250, 180, 50, 140),
                                        ))
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 3))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(channel_name)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                });
                                ui.add_space(2.0);

                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::splat(av_size),
                                    egui::Sense::hover(),
                                );
                                let painter = ui.painter();
                                painter.circle_filled(
                                    rect.center(),
                                    av_size / 2.0,
                                    palette.surface_elevated,
                                );
                                painter.circle_stroke(
                                    rect.center(),
                                    av_size / 2.0,
                                    Stroke::new(1.2, dot_color),
                                );
                                painter.circle_filled(
                                    rect.center(),
                                    4.0 + pulse * 1.5,
                                    dot_color,
                                );
                            } else {
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::splat(av_size),
                                    egui::Sense::hover(),
                                );
                                let painter = ui.painter();
                                painter.circle_filled(
                                    rect.center(),
                                    av_size / 2.0,
                                    palette.surface_elevated,
                                );
                                painter.circle_stroke(
                                    rect.center(),
                                    av_size / 2.0,
                                    Stroke::new(1.2, dot_color),
                                );
                                painter.circle_filled(
                                    rect.center(),
                                    4.0 + pulse * 1.5,
                                    dot_color,
                                );

                                ui.add_space(2.0);
                                ui.vertical(|ui| {
                                    ui.add_space(v_space);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 220))
                                        .stroke(Stroke::new(
                                            1.0,
                                            Color32::from_rgba_premultiplied(250, 180, 50, 140),
                                        ))
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 3))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(channel_name)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                });
                            }
                        });
                    } else {
                        let green_color = Color32::from_rgb(35, 165, 90);
                        ui.horizontal(|ui| {
                            if is_right_side {
                                ui.vertical(|ui| {
                                    ui.add_space(v_space);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 220))
                                        .stroke(Stroke::new(
                                            1.0,
                                            Color32::from_rgba_premultiplied(35, 165, 90, 140),
                                        ))
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 3))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(channel_name)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                });
                                ui.add_space(2.0);

                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::splat(av_size),
                                    egui::Sense::hover(),
                                );
                                let painter = ui.painter();
                                painter.circle_filled(
                                    rect.center(),
                                    av_size / 2.0,
                                    palette.surface_elevated,
                                );
                                painter.circle_stroke(
                                    rect.center(),
                                    av_size / 2.0,
                                    Stroke::new(1.2, green_color),
                                );
                                painter.circle_filled(
                                    rect.center(),
                                    4.5,
                                    green_color,
                                );
                            } else {
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::splat(av_size),
                                    egui::Sense::hover(),
                                );
                                painter_circle(ui, palette, rect, av_size, green_color);

                                ui.add_space(2.0);
                                ui.vertical(|ui| {
                                    ui.add_space(v_space);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 220))
                                        .stroke(Stroke::new(
                                            1.0,
                                            Color32::from_rgba_premultiplied(35, 165, 90, 140),
                                        ))
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 3))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(channel_name)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                });
                            }
                        });
                    }
                }
            } else {
                for user in users_to_display {
                    let av_size = config.avatar_size;
                    let radius = av_size / 2.0;
                    let v_space = ((av_size - 18.0) / 2.0).max(0.0f32);
                    let resp = ui.horizontal(|ui| {
                        if is_right_side {
                            let row_id = ui.make_persistent_id(format!("bubble_row_{}", user.id));
                            if let Some(w) = ui.data(|d| d.get_temp::<f32>(row_id)) {
                                let pad = (ui.available_width() - w).max(0.0);
                                ui.add_space(pad);
                            }
                        }
                        if is_right_side {
                            if user.is_speaking && config.show_names {
                                ui.vertical(|ui| {
                                    ui.add_space(v_space);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 220))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(35, 165, 90)))
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(&user.name)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                });
                                ui.add_space(2.0);
                            }

                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::splat(av_size),
                                egui::Sense::hover(),
                            );
                            let center = rect.center();
                            let painter = ui.painter();

                            if user.is_speaking {
                                let pulse = ((time_elapsed_secs * 7.0).sin() * 0.5 + 0.5) as f32;
                                let ring_radius = radius + 2.0 + pulse * 2.5;
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

                            if user.is_deafened {
                                let badge_pos = egui::pos2(center.x - radius * 0.6, center.y + radius * 0.6);
                                painter.circle_filled(badge_pos, 4.5, palette.danger);
                                painter.circle_stroke(badge_pos, 4.5, Stroke::new(1.0, palette.background));
                            } else if user.is_muted {
                                let badge_pos = egui::pos2(center.x - radius * 0.6, center.y + radius * 0.6);
                                painter.circle_filled(badge_pos, 4.5, palette.border);
                                painter.circle_stroke(badge_pos, 4.5, Stroke::new(1.0, palette.background));
                            }
                        } else {
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::splat(av_size),
                                egui::Sense::hover(),
                            );
                            let center = rect.center();
                            let painter = ui.painter();

                            if user.is_speaking {
                                let pulse = ((time_elapsed_secs * 7.0).sin() * 0.5 + 0.5) as f32;
                                let ring_radius = radius + 2.0 + pulse * 2.5;
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

                            if user.is_deafened {
                                let badge_pos = egui::pos2(center.x + radius * 0.6, center.y + radius * 0.6);
                                painter.circle_filled(badge_pos, 4.5, palette.danger);
                                painter.circle_stroke(badge_pos, 4.5, Stroke::new(1.0, palette.background));
                            } else if user.is_muted {
                                let badge_pos = egui::pos2(center.x + radius * 0.6, center.y + radius * 0.6);
                                painter.circle_filled(badge_pos, 4.5, palette.border);
                                painter.circle_stroke(badge_pos, 4.5, Stroke::new(1.0, palette.background));
                            }

                            if user.is_speaking && config.show_names {
                                ui.add_space(2.0);
                                ui.vertical(|ui| {
                                    ui.add_space(v_space);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 220))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(35, 165, 90)))
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(&user.name)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                });
                            }
                        }
                    });
                    if is_right_side {
                        let row_id = ui.make_persistent_id(format!("bubble_row_{}", user.id));
                        ui.data_mut(|d| d.insert_temp(row_id, resp.response.rect.width()));
                    }
                    ui.add_space(3.0);
                }
            }

            if alt_held {
                ui.horizontal(|ui| {
                    if is_right_side {
                        let (r_rect, r_resp) = ui.allocate_exact_size(
                            Vec2::splat(12.0),
                            egui::Sense::drag(),
                        );
                        let glyph = "⌞";
                        ui.painter().text(
                            r_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            glyph,
                            egui::FontId::proportional(12.0),
                            palette.accent,
                        );
                        if r_resp.hovered() || r_resp.dragged() {
                            *is_resizing = true;
                        }
                        if r_resp.dragged() {
                            let delta = -r_resp.drag_delta().x;
                            config.avatar_size = (config.avatar_size + delta * 0.25)
                                .clamp(20.0, 72.0);
                            config.save();
                        }
                    } else {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let (r_rect, r_resp) = ui.allocate_exact_size(
                                    Vec2::splat(12.0),
                                    egui::Sense::drag(),
                                );
                                let glyph = "⌟";
                                ui.painter().text(
                                    r_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    glyph,
                                    egui::FontId::proportional(12.0),
                                    palette.accent,
                                );
                                if r_resp.hovered() || r_resp.dragged() {
                                    *is_resizing = true;
                                }
                                if r_resp.dragged() {
                                    let delta = r_resp.drag_delta().x;
                                    config.avatar_size = (config.avatar_size + delta * 0.25)
                                        .clamp(20.0, 72.0);
                                    config.save();
                                }
                            },
                        );
                    }
                });
            }
        });
}

fn painter_circle(ui: &mut egui::Ui, palette: &ThemePalette, rect: egui::Rect, av_size: f32, green_color: Color32) {
    let painter = ui.painter();
    painter.circle_filled(
        rect.center(),
        av_size / 2.0,
        palette.surface_elevated,
    );
    painter.circle_stroke(
        rect.center(),
        av_size / 2.0,
        Stroke::new(1.2, green_color),
    );
    painter.circle_filled(
        rect.center(),
        4.5,
        green_color,
    );
}
