use egui::{Color32, CornerRadius, Margin, Stroke, Vec2};

use crate::{
    avatar_cache::AvatarCache,
    presets::{MonitorBounds, OverlayConfig},
    theme::ThemePalette,
    ui::avatar::{render_avatar, VoiceUser},
};

pub fn render_transparent_overlay(
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
        Stroke::NONE
    };

    let av_size = config.avatar_size.clamp(20.0, 72.0);
    let font_size = (av_size * 0.38).clamp(11.0, 18.0);
    let radius = av_size / 2.0;

    let users_to_display: Vec<_> = users
        .iter()
        .filter(|u| !config.only_speaking || u.is_speaking)
        .collect();

    let area_w = (av_size + 160.0).max(180.0);
    let frame_fill = if alt_held {
        Color32::from_rgba_unmultiplied(15, 15, 25, 140)
    } else {
        Color32::TRANSPARENT
    };

    egui::Frame::new()
        .fill(frame_fill)
        .stroke(frame_stroke)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(if alt_held { Margin::same(6) } else { Margin::ZERO })
        .show(ui, |ui| {
            ui.set_width(area_w);

            if alt_held {
                let sp_icon = if config.only_speaking { "🗣" } else { "👥" };
                let sp_tip = if config.only_speaking {
                    "Showing only speaking users (Click to show all)"
                } else {
                    "Showing all users (Click to show only speaking)"
                };

                ui.horizontal(|ui| {
                    if is_right_side {
                        ui.label(
                            egui::RichText::new("Transparent (Right)")
                                .size(11.0)
                                .color(palette.accent),
                        );

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

                        ui.label(
                            egui::RichText::new("Transparent (Left)")
                                .size(11.0)
                                .color(palette.accent),
                        );
                    }
                });
                ui.add_space(3.0);
            }

            if users_to_display.is_empty() {
                if !channel_name.is_empty() {
                    if !discord_connected {
                        let tint_bg = Color32::from_rgba_unmultiplied(
                            10,
                            10,
                            15,
                            config.bg_tint_alpha.max(100),
                        );
                        let pulse = ((time_elapsed_secs * 4.0).sin() * 0.5 + 0.5) as f32;
                        let dot_color = Color32::from_rgba_premultiplied(
                            250,
                            180,
                            50,
                            (160.0 + pulse * 95.0) as u8,
                        );

                        ui.horizontal(|ui| {
                            let pill_id = ui.make_persistent_id("trans_channel_status_pill");
                            if is_right_side {
                                let w = ui.data(|d| d.get_temp::<f32>(pill_id)).unwrap_or_else(|| {
                                    let ch_len = channel_name.chars().count() as f32;
                                    ch_len * (font_size * 0.6) + 40.0
                                });
                                let pad = (ui.available_width() - w).max(0.0);
                                ui.add_space(pad);
                            }

                            let f = egui::Frame::new()
                                .fill(tint_bg)
                                .stroke(Stroke::new(
                                    1.0,
                                    Color32::from_rgba_premultiplied(250, 180, 50, 130),
                                ))
                                .corner_radius(CornerRadius::same(((av_size + 8.0) / 2.0).round() as u8))
                                .inner_margin(Margin::symmetric(8, 5))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let dot_size = 12.0f32;
                                        let v_space = ((dot_size - font_size) / 2.0).max(0.0f32);
                                        if is_right_side {
                                            ui.vertical(|ui| {
                                                ui.add_space(v_space);
                                                ui.label(
                                                    egui::RichText::new(channel_name)
                                                        .size(font_size)
                                                        .strong()
                                                        .color(Color32::from_gray(230)),
                                                );
                                            });
                                            ui.add_space(2.0);
                                            let (rect, _) = ui.allocate_exact_size(
                                                Vec2::splat(dot_size),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                3.5 + pulse * 1.0,
                                                dot_color,
                                            );
                                        } else {
                                            let (rect, _) = ui.allocate_exact_size(
                                                Vec2::splat(dot_size),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                3.5 + pulse * 1.0,
                                                dot_color,
                                            );
                                            ui.add_space(2.0);
                                            ui.vertical(|ui| {
                                                ui.add_space(v_space);
                                                ui.label(
                                                    egui::RichText::new(channel_name)
                                                        .size(font_size)
                                                        .strong()
                                                        .color(Color32::from_gray(230)),
                                                );
                                            });
                                        }
                                    });
                                });

                            if is_right_side {
                                ui.data_mut(|d| d.insert_temp(pill_id, f.response.rect.width()));
                            }
                        });
                    } else {
                        let tint_bg = Color32::from_rgba_unmultiplied(
                            10,
                            10,
                            15,
                            config.bg_tint_alpha.max(100),
                        );
                        let dot_color = Color32::from_rgb(35, 165, 90);

                        ui.horizontal(|ui| {
                            let pill_id = ui.make_persistent_id("trans_channel_status_pill");
                            if is_right_side {
                                let w = ui.data(|d| d.get_temp::<f32>(pill_id)).unwrap_or_else(|| {
                                    let ch_len = channel_name.chars().count() as f32;
                                    ch_len * (font_size * 0.6) + 40.0
                                });
                                let pad = (ui.available_width() - w).max(0.0);
                                ui.add_space(pad);
                            }

                            let f = egui::Frame::new()
                                .fill(tint_bg)
                                .stroke(Stroke::new(
                                    1.0,
                                    Color32::from_rgba_premultiplied(35, 165, 90, 130),
                                ))
                                .corner_radius(CornerRadius::same(((av_size + 8.0) / 2.0).round() as u8))
                                .inner_margin(Margin::symmetric(8, 5))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let dot_size = 12.0f32;
                                        let v_space = ((dot_size - font_size) / 2.0).max(0.0f32);
                                        if is_right_side {
                                            ui.vertical(|ui| {
                                                ui.add_space(v_space);
                                                ui.label(
                                                    egui::RichText::new(channel_name)
                                                        .size(font_size)
                                                        .strong()
                                                        .color(Color32::from_gray(230)),
                                                );
                                            });
                                            ui.add_space(2.0);
                                            let (rect, _) = ui.allocate_exact_size(
                                                Vec2::splat(dot_size),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                4.0,
                                                dot_color,
                                            );
                                        } else {
                                            let (rect, _) = ui.allocate_exact_size(
                                                Vec2::splat(dot_size),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                4.0,
                                                dot_color,
                                            );
                                            ui.add_space(2.0);
                                            ui.vertical(|ui| {
                                                ui.add_space(v_space);
                                                ui.label(
                                                    egui::RichText::new(channel_name)
                                                        .size(font_size)
                                                        .strong()
                                                        .color(Color32::from_gray(230)),
                                                );
                                            });
                                        }
                                    });
                                });

                            if is_right_side {
                                ui.data_mut(|d| d.insert_temp(pill_id, f.response.rect.width()));
                            }
                        });
                    }
                }
            } else {
                for user in users_to_display {
                    let tint_bg = Color32::from_rgba_unmultiplied(
                        10,
                        10,
                        15,
                        config.bg_tint_alpha,
                    );

                    ui.horizontal(|ui| {
                        let pill_id = ui.make_persistent_id(format!("trans_pill_{}", user.id));
                        if is_right_side {
                            let w = ui.data(|d| d.get_temp::<f32>(pill_id)).unwrap_or_else(|| {
                                let name_len = user.name.chars().count() as f32;
                                let name_est = if config.show_names { name_len * (font_size * 0.55) + 4.0 } else { 0.0 };
                                let badge_est = if user.is_deafened || user.is_muted { 36.0 } else { 0.0 };
                                av_size + name_est + badge_est + 12.0
                            });
                            let pad = (ui.available_width() - w).max(0.0);
                            ui.add_space(pad);
                        }

                        let f = egui::Frame::new()
                            .fill(tint_bg)
                            .stroke(if user.is_speaking {
                                Stroke::new(
                                    1.0,
                                    Color32::from_rgba_premultiplied(
                                        35, 165, 90, 160,
                                    ),
                                )
                            } else {
                                Stroke::NONE
                            })
                            .corner_radius(CornerRadius::same(((av_size + 8.0) / 2.0).round() as u8))
                            .inner_margin(Margin::symmetric(6, 4))
                            .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let name_color = if user.is_speaking {
                                    Color32::WHITE
                                } else {
                                    Color32::from_gray(225)
                                };
                                let v_space = ((av_size - font_size) / 2.0 - 1.0f32).max(0.0f32);

                                if is_right_side {
                                    if config.show_names {
                                        ui.vertical(|ui| {
                                            ui.add_space(v_space);
                                            ui.horizontal(|ui| {
                                                if user.is_deafened {
                                                    ui.colored_label(
                                                        palette.danger,
                                                        "deaf •",
                                                    );
                                                } else if user.is_muted {
                                                    ui.colored_label(
                                                        palette.text_muted,
                                                        "mute •",
                                                    );
                                                }

                                                ui.label(
                                                    egui::RichText::new(&user.name)
                                                        .size(font_size)
                                                        .strong()
                                                        .color(name_color),
                                                );
                                            });
                                        });

                                        ui.add_space(4.0);
                                    }

                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::splat(av_size),
                                        egui::Sense::hover(),
                                    );
                                    let center = rect.center();
                                    let painter = ui.painter();

                                    if user.is_speaking {
                                        let pulse = ((time_elapsed_secs * 7.0)
                                            .sin()
                                            * 0.5
                                            + 0.5)
                                            as f32;
                                        let ring_radius = radius + 2.0 + pulse * 2.0;
                                        let ring_color =
                                            Color32::from_rgba_premultiplied(
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

                                    render_avatar(
                                        ui.ctx(),
                                        painter,
                                        palette,
                                        user,
                                        center,
                                        radius,
                                        avatar_cache,
                                    );

                                    if user.is_deafened {
                                        let badge_pos = egui::pos2(
                                            center.x - radius * 0.6,
                                            center.y + radius * 0.6,
                                        );
                                        painter.circle_filled(
                                            badge_pos,
                                            4.0,
                                            palette.danger,
                                        );
                                        painter.circle_stroke(
                                            badge_pos,
                                            4.0,
                                            Stroke::new(1.0, Color32::BLACK),
                                        );
                                    } else if user.is_muted {
                                        let badge_pos = egui::pos2(
                                            center.x - radius * 0.6,
                                            center.y + radius * 0.6,
                                        );
                                        painter.circle_filled(
                                            badge_pos,
                                            4.0,
                                            palette.border,
                                        );
                                        painter.circle_stroke(
                                            badge_pos,
                                            4.0,
                                            Stroke::new(1.0, Color32::BLACK),
                                        );
                                    }
                                } else {
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::splat(av_size),
                                        egui::Sense::hover(),
                                    );
                                    let center = rect.center();
                                    let painter = ui.painter();

                                    if user.is_speaking {
                                        let pulse = ((time_elapsed_secs * 7.0)
                                            .sin()
                                            * 0.5
                                            + 0.5)
                                            as f32;
                                        let ring_radius = radius + 2.0 + pulse * 2.0;
                                        let ring_color =
                                            Color32::from_rgba_premultiplied(
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

                                    render_avatar(
                                        ui.ctx(),
                                        painter,
                                        palette,
                                        user,
                                        center,
                                        radius,
                                        avatar_cache,
                                    );

                                    if user.is_deafened {
                                        let badge_pos = egui::pos2(
                                            center.x + radius * 0.6,
                                            center.y + radius * 0.6,
                                        );
                                        painter.circle_filled(
                                            badge_pos,
                                            4.0,
                                            palette.danger,
                                        );
                                        painter.circle_stroke(
                                            badge_pos,
                                            4.0,
                                            Stroke::new(1.0, Color32::BLACK),
                                        );
                                    } else if user.is_muted {
                                        let badge_pos = egui::pos2(
                                            center.x + radius * 0.6,
                                            center.y + radius * 0.6,
                                        );
                                        painter.circle_filled(
                                            badge_pos,
                                            4.0,
                                            palette.border,
                                        );
                                        painter.circle_stroke(
                                            badge_pos,
                                            4.0,
                                            Stroke::new(1.0, Color32::BLACK),
                                        );
                                    }

                                    if config.show_names {
                                        ui.add_space(4.0);

                                        ui.vertical(|ui| {
                                            ui.add_space(v_space);
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&user.name)
                                                        .size(font_size)
                                                        .strong()
                                                        .color(name_color),
                                                );

                                                if user.is_deafened {
                                                    ui.colored_label(
                                                        palette.danger,
                                                        "• deaf",
                                                    );
                                                } else if user.is_muted {
                                                    ui.colored_label(
                                                        palette.text_muted,
                                                        "• mute",
                                                    );
                                                }
                                            });
                                        });
                                    }
                                }
                            });
                        });

                        if is_right_side {
                            let pill_id = ui.make_persistent_id(format!("trans_pill_{}", user.id));
                            ui.data_mut(|d| d.insert_temp(pill_id, f.response.rect.width()));
                        }
                    });

                    ui.add_space(3.0);
                }
            }

            if alt_held {
                ui.horizontal(|ui| {
                    if is_right_side {
                        let (r_rect, r_resp) = ui.allocate_exact_size(
                            Vec2::splat(14.0),
                            egui::Sense::drag(),
                        );
                        let glyph = "⌞";
                        ui.painter().text(
                            r_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            glyph,
                            egui::FontId::proportional(14.0),
                            palette.accent,
                        );
                        if r_resp.hovered() || r_resp.dragged() {
                            *is_resizing = true;
                        }
                        if r_resp.dragged() {
                            let delta = -r_resp.drag_delta().x;
                            config.avatar_size = (config.avatar_size
                                + delta * 0.25)
                                .clamp(20.0, 72.0);
                            config.save();
                        }
                    } else {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let (r_rect, r_resp) = ui.allocate_exact_size(
                                    Vec2::splat(14.0),
                                    egui::Sense::drag(),
                                );
                                let glyph = "⌟";
                                ui.painter().text(
                                    r_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    glyph,
                                    egui::FontId::proportional(14.0),
                                    palette.accent,
                                );
                                if r_resp.hovered() || r_resp.dragged() {
                                    *is_resizing = true;
                                }
                                if r_resp.dragged() {
                                    let delta = r_resp.drag_delta().x;
                                    config.avatar_size = (config.avatar_size
                                        + delta * 0.25)
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
