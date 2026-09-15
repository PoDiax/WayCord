use egui::{Color32, CornerRadius, Vec2};

use crate::{
    avatar_cache::AvatarCache,
    presets::{self, MonitorBounds, OverlayConfig},
    theme::{selectable_pill, ThemeKind, ThemePalette},
    WAYCORD_LOGO_BYTES,
};

pub fn render_settings_window(
    ctx: &egui::Context,
    palette: &ThemePalette,
    channel_name: &str,
    config: &mut OverlayConfig,
    alt_held: bool,
    show_settings: &mut bool,
    screen_width: f32,
    screen_height: f32,
    discord_connected: bool,
    active_monitor: &MonitorBounds,
) {
    if !*show_settings {
        return;
    }

    if !alt_held {
        *show_settings = false;
        return;
    }

    let mut open = true;
    let win_x = (config.x + 200.0).min((screen_width - 340.0).max(20.0));
    let win_y = config.y.min((screen_height - 420.0).max(20.0));

    egui::Window::new("WayCord Settings")
        .open(&mut open)
        .default_pos(egui::pos2(win_x, win_y))
        .default_width(320.0)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Ok(color_img) = AvatarCache::decode_bytes(WAYCORD_LOGO_BYTES) {
                    let tex = ctx.load_texture(
                        "waycord_logo_badge",
                        color_img,
                        egui::TextureOptions::LINEAR,
                    );
                    ui.image((tex.id(), Vec2::new(24.0, 24.0)));
                }
                ui.heading(egui::RichText::new("WayCord").size(17.0).color(palette.accent).strong());
                ui.label(egui::RichText::new("Overlay Settings").size(13.0).color(palette.text_muted));
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Display Style").size(13.0).color(palette.text_primary).strong());
            ui.horizontal(|ui| {
                if selectable_pill(
                    ui,
                    palette,
                    config.style == presets::OverlayStyle::Transparent,
                    "Transparent",
                )
                .clicked()
                {
                    config.style = presets::OverlayStyle::Transparent;
                    config.save();
                }
                if selectable_pill(
                    ui,
                    palette,
                    config.style == presets::OverlayStyle::Bubble,
                    "Bubble",
                )
                .clicked()
                {
                    config.style = presets::OverlayStyle::Bubble;
                    config.save();
                }
                if selectable_pill(
                    ui,
                    palette,
                    config.style == presets::OverlayStyle::Card,
                    "Card",
                )
                .clicked()
                {
                    config.style = presets::OverlayStyle::Card;
                    config.save();
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("Voice Options")
                    .size(13.0)
                    .color(palette.text_primary)
                    .strong(),
            );
            if ui
                .checkbox(&mut config.only_speaking, "Show only speaking users")
                .changed()
            {
                config.save();
            }
            if ui
                .checkbox(&mut config.show_names, "Show user names")
                .changed()
            {
                config.save();
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Avatar Size").size(13.0).color(palette.text_primary).strong());
            if ui
                .add(
                    egui::Slider::new(&mut config.avatar_size, 20.0..=64.0)
                        .text("px")
                        .step_by(2.0),
                )
                .changed()
            {
                config.save();
            }

            if config.style == presets::OverlayStyle::Transparent {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Background Tint Alpha")
                        .size(13.0)
                        .color(palette.text_primary)
                        .strong(),
                );
                if ui
                    .add(egui::Slider::new(&mut config.bg_tint_alpha, 0..=255).text("alpha"))
                    .changed()
                {
                    config.save();
                }
            }

            if config.style == presets::OverlayStyle::Card {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Card Width").size(13.0).color(palette.text_primary).strong());
                if ui
                    .add(
                        egui::Slider::new(&mut config.card_width, 180.0..=480.0)
                            .text("px")
                            .step_by(10.0),
                    )
                    .changed()
                {
                    config.save();
                }

                ui.add_space(4.0);
                ui.label(egui::RichText::new("Opacity").size(13.0).color(palette.text_primary).strong());
                if ui
                    .add(
                        egui::Slider::new(&mut config.opacity, 0.2..=1.0)
                            .text("opacity")
                            .step_by(0.05),
                    )
                    .changed()
                {
                    config.save();
                }
            }

            ui.add_space(3.0);
            if ui
                .button(egui::RichText::new("↺ Reset Size to Default (36px)").size(12.0))
                .on_hover_text("Reset avatar size and card width to default values")
                .clicked()
            {
                config.reset_size();
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Screen Docking").size(13.0).color(palette.text_primary).strong());
            ui.horizontal(|ui| {
                if ui.button("Snap Left ◀").clicked() {
                    config.snap_left_on_monitor(active_monitor);
                }
                if ui.button("▶ Snap Right").clicked() {
                    config.snap_right_on_monitor(active_monitor);
                }
                if ui.button("↺ Reset Size").clicked() {
                    config.reset_size();
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Theme").size(13.0).color(palette.text_primary).strong());
            ui.horizontal_wrapped(|ui| {
                let themes = [
                    (ThemeKind::Mocha, "Mocha"),
                    (ThemeKind::TokyoNight, "Tokyo Night"),
                    (ThemeKind::Nord, "Nord"),
                    (ThemeKind::Gruvbox, "Gruvbox"),
                    (ThemeKind::RosePine, "Rosé Pine"),
                ];
                for (t_kind, t_name) in themes {
                    let active = config.theme == t_kind;
                    if selectable_pill(ui, palette, active, t_name).clicked() {
                        config.theme = t_kind;
                        config.save();
                    }
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Discord Bridge").size(13.0).color(palette.text_primary).strong());
            if discord_connected {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(35, 165, 90), "●");
                    ui.colored_label(
                        Color32::from_rgb(35, 165, 90),
                        "Discord Connected (Vesktop / Official)",
                    );
                });
                ui.label(
                    egui::RichText::new(format!("Status: {}", channel_name))
                        .size(12.0)
                        .color(palette.text_muted),
                );
            } else {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(250, 180, 50), "●");
                    ui.colored_label(
                        Color32::from_rgb(250, 180, 50),
                        "Waiting for Discord client (ws://127.0.0.1:9876)...",
                    );
                });

                ui.add_space(6.0);
                egui::Frame::new()
                    .fill(palette.surface_elevated)
                    .corner_radius(CornerRadius::same(8))
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("💡 Using Standard Discord?")
                                    .strong()
                                    .size(12.5)
                                    .color(palette.text_primary),
                            );
                        });
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(
                                "Stock official Discord does not support third-party overlay bridges directly. To enable the voice overlay, please use:",
                            )
                            .size(11.5)
                            .color(palette.text_muted),
                        );
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Recommended:")
                                    .strong()
                                    .size(12.0)
                                    .color(palette.accent),
                            );
                            ui.hyperlink_to("Install Vesktop (vesktop.dev)", "https://vesktop.dev");
                        });
                        ui.label(
                            egui::RichText::new(
                                "   Native Discord client with full Wayland, screen share & built-in overlay bridge support.",
                            )
                            .size(10.5)
                            .color(palette.text_muted),
                        );

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Alternative:")
                                    .strong()
                                    .size(12.0)
                                    .color(palette.text_primary),
                            );
                            ui.hyperlink_to("Vencord Installer Script", "https://vencord.dev/download");
                        });
                        ui.label(
                            egui::RichText::new(
                                "   Or patch standard Discord using the official Vencord installer script:",
                            )
                            .size(10.5)
                            .color(palette.text_muted),
                        );
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            let cmd = "sh -c \"$(curl -sS https://raw.githubusercontent.com/Vendicated/VencordInstaller/main/install.sh)\"";
                            ui.monospace(
                                egui::RichText::new(cmd)
                                    .size(10.0)
                                    .color(palette.text_primary),
                            );
                            if ui.button("📋 Copy").clicked() {
                                ui.ctx().copy_text(cmd.to_string());
                            }
                        });
                    });
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new(
                    "Tip: Hold ALT at any time to move, resize, or interact with this overlay!",
                )
                .size(11.0)
                .color(palette.text_muted),
            );

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("✓ Settings saved automatically")
                        .size(11.5)
                        .color(palette.success)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close Settings").clicked() {
                        config.save();
                        *show_settings = false;
                    }
                });
            });
        });

    if !open {
        *show_settings = false;
    }
}
