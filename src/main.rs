use egui::{CornerRadius, Margin, Stroke, Vec2};

use waycord::{ThemeKind, ThemePalette, apply_theme};


#[derive(PartialEq)]
enum ActiveTab {
    Overview,
    Components,
    PaletteInspector,
    VoiceOverlay,
}

struct UiPreviewApp {
    selected_theme: ThemeKind,
    palette: ThemePalette,
    active_tab: ActiveTab,
    corner_rounding: f32,
    slider_value: f32,
    toggle_1: bool,
    toggle_2: bool,
    text_input: String,
    progress: f32,
    config: waycord::OverlayConfig,
    show_settings: bool,
    alt_held: bool,
    demo_mode: bool,
    avatar_cache: waycord::avatar_cache::AvatarCache,
}

impl UiPreviewApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let initial_theme = ThemeKind::Mocha;
        let palette = ThemePalette::from_kind(initial_theme);
        let corner_rounding = 6.0;

        apply_theme(&cc.egui_ctx, &palette, corner_rounding);

        let config = waycord::OverlayConfig::load();
        let avatar_cache = waycord::avatar_cache::AvatarCache::new();

        Self {
            selected_theme: initial_theme,
            palette,
            active_tab: ActiveTab::Overview,
            corner_rounding,
            slider_value: 42.0,
            toggle_1: true,
            toggle_2: false,
            text_input: "Sample input text".to_string(),
            progress: 0.65,
            config,
            show_settings: false,
            alt_held: false,
            demo_mode: true,
            avatar_cache,
        }
    }
}

impl eframe::App for UiPreviewApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let mut theme_changed = false;

        egui::Panel::left("nav_panel")
            .resizable(false)
            .default_size(210.0)
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui.heading("UI Preview");
                ui.add_space(12.0);

                ui.label(egui::RichText::new("TABS").size(11.0).color(self.palette.text_muted));
                ui.add_space(4.0);

                let tabs = [
                    (ActiveTab::Overview, "Dashboard"),
                    (ActiveTab::Components, "Widgets"),
                    (ActiveTab::PaletteInspector, "Palette"),
                    (ActiveTab::VoiceOverlay, "Voice Overlay 🎙"),
                ];

                for (tab, label) in tabs {
                    let is_active = self.active_tab == tab;
                    if ui.selectable_label(is_active, label).clicked() {
                        self.active_tab = tab;
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(egui::RichText::new("THEME SELECTION").size(11.0).color(self.palette.text_muted));
                ui.add_space(4.0);

                let themes = [
                    (ThemeKind::Mocha, "Catppuccin Mocha"),
                    (ThemeKind::TokyoNight, "Tokyo Night"),
                    (ThemeKind::Nord, "Nord"),
                    (ThemeKind::Gruvbox, "Gruvbox Dark"),
                    (ThemeKind::RosePine, "Rosé Pine"),
                ];

                for (theme, name) in themes {
                    let is_selected = self.selected_theme == theme;
                    if ui.selectable_label(is_selected, name).clicked() && !is_selected {
                        self.selected_theme = theme;
                        self.palette = ThemePalette::from_kind(theme);
                        theme_changed = true;
                    }
                }

                ui.add_space(16.0);
                ui.label(egui::RichText::new("CORNER ROUNDING").size(11.0).color(self.palette.text_muted));
                if ui.add(egui::Slider::new(&mut self.corner_rounding, 0.0..=16.0).text("px")).changed() {
                    theme_changed = true;
                }
            });

        if theme_changed {
            apply_theme(&ctx, &self.palette, self.corner_rounding);
        }

        // Central Content Panel
        egui::CentralPanel::default().show(ui, |ui| {
            match self.active_tab {
                ActiveTab::Overview => self.render_overview(ui),
                ActiveTab::Components => self.render_components(ui),
                ActiveTab::PaletteInspector => self.render_palette_inspector(ui),
                ActiveTab::VoiceOverlay => self.render_voice_tab(ui),
            }
        });
    }
}

impl UiPreviewApp {
    fn render_overview(&mut self, ui: &mut egui::Ui) {
        ui.heading(format!("Active Theme: {}", self.palette.name));
        ui.label(egui::RichText::new("Interactive preview of styling, colors, and layout metrics.").color(self.palette.text_muted));
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            card(ui, &self.palette, |ui| {
                ui.label(egui::RichText::new("SYSTEM STATUS").size(11.0).color(self.palette.text_muted));
                ui.heading(egui::RichText::new("Connected").color(self.palette.success));
                ui.label("Running at 60 FPS");
            });

            card(ui, &self.palette, |ui| {
                ui.label(egui::RichText::new("MEMORY USAGE").size(11.0).color(self.palette.text_muted));
                ui.heading(egui::RichText::new("42.8 MB").color(self.palette.accent));
                ui.add(egui::ProgressBar::new(0.42).show_percentage());
            });

            card(ui, &self.palette, |ui| {
                ui.label(egui::RichText::new("ACTIVE PRESET").size(11.0).color(self.palette.text_muted));
                ui.heading("Default");
                ui.label("Saved 2 mins ago");
            });
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(12.0);

        ui.heading("Quick Controls");
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.toggle_1, "Enable Hardware Acceleration");
            ui.checkbox(&mut self.toggle_2, "Verbose Logging");
        });

        ui.add_space(8.0);
        ui.add(egui::Slider::new(&mut self.progress, 0.0..=1.0).text("Level Progress"));
    }

    fn render_components(&mut self, ui: &mut egui::Ui) {
        ui.heading("Widget Component Showcase");
        ui.add_space(12.0);

        card(ui, &self.palette, |ui| {
            ui.label(egui::RichText::new("BUTTON STATES").size(12.0).color(self.palette.accent));
            ui.horizontal(|ui| {
                if ui.button("Default Button").clicked() {}
                let _ = ui.button(egui::RichText::new("Accent Button").color(self.palette.accent));
                let _ = ui.button(egui::RichText::new("Danger Button").color(self.palette.danger));
            });
        });

        ui.add_space(10.0);

        card(ui, &self.palette, |ui| {
            ui.label(egui::RichText::new("INPUT CONTROLS").size(12.0).color(self.palette.accent));
            ui.horizontal(|ui| {
                ui.label("Text input:");
                ui.text_edit_singleline(&mut self.text_input);
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Slider value:");
                ui.add(egui::Slider::new(&mut self.slider_value, 0.0..=100.0));
            });
        });
    }

    fn render_palette_inspector(&self, ui: &mut egui::Ui) {
        ui.heading("Color Palette Roles");
        ui.label(egui::RichText::new("Functional color mappings used across this theme.").color(self.palette.text_muted));
        ui.add_space(16.0);

        let colors = [
            ("Background", self.palette.background),
            ("Surface", self.palette.surface),
            ("Surface Elevated", self.palette.surface_elevated),
            ("Accent", self.palette.accent),
            ("Accent Hovered", self.palette.accent_hovered),
            ("Accent Active", self.palette.accent_active),
            ("Primary Text", self.palette.text_primary),
            ("Muted Text", self.palette.text_muted),
            ("Border", self.palette.border),
            ("Success", self.palette.success),
            ("Danger", self.palette.danger),
        ];

        for (label, color) in colors {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(28.0, 18.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, CornerRadius::same(4), color);
                ui.painter().rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.0, self.palette.border), egui::StrokeKind::Inside);
                ui.label(label);
                ui.label(egui::RichText::new(format!("rgb({}, {}, {})", color.r(), color.g(), color.b())).color(self.palette.text_muted).size(11.0));
            });
            ui.add_space(4.0);
        }
    }

    fn render_voice_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Discord Voice Overlay Preview");
        ui.label(
            egui::RichText::new("Preview how the floating voice HUD looks over games & desktop.")
                .color(self.palette.text_muted),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui.button("Launch Transparent Overlay (Click-Through)").clicked() {
                let _ = std::process::Command::new("cargo")
                    .args(["run", "--bin", "overlay"])
                    .spawn();
            }

            if ui.button("Launch Interactive Overlay (For Testing)").clicked() {
                let _ = std::process::Command::new("cargo")
                    .args(["run", "--bin", "overlay", "--", "--interactive"])
                    .spawn();
            }
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(12.0);

        ui.label(egui::RichText::new("Simulated In-Game HUD:").strong());
        ui.add_space(8.0);

        let time = ui.input(|i| i.time);
        let pulse_user_1 = (time * 8.0).sin().abs() as f32;
        let pulse_user_2 = (time * 6.0).cos().abs() as f32;
        let users = vec![
            waycord::VoiceUser {
                id: "demo_user_1".to_string(),
                name: "PoDiax".to_string(),
                avatar_initials: "PD".to_string(),
                avatar_url: Some("https://cdn.discordapp.com/embed/avatars/1.png".to_string()),
                is_speaking: (time % 5.0) < 3.0,
                is_muted: false,
                is_deafened: false,
                volume_level: pulse_user_1 * 0.7 + 0.3,
            },
            waycord::VoiceUser {
                id: "demo_user_2".to_string(),
                name: "Ghost".to_string(),
                avatar_initials: "GH".to_string(),
                avatar_url: Some("https://cdn.discordapp.com/embed/avatars/2.png".to_string()),
                is_speaking: (time % 5.0) >= 3.0,
                is_muted: false,
                is_deafened: false,
                volume_level: pulse_user_2 * 0.8 + 0.2,
            },
            waycord::VoiceUser {
                id: "demo_user_3".to_string(),
                name: "Viper".to_string(),
                avatar_initials: "VP".to_string(),
                avatar_url: Some("https://cdn.discordapp.com/embed/avatars/3.png".to_string()),
                is_speaking: false,
                is_muted: true,
                is_deafened: false,
                volume_level: 0.0,
            },
            waycord::VoiceUser {
                id: "demo_user_4".to_string(),
                name: "Echo".to_string(),
                avatar_initials: "EC".to_string(),
                avatar_url: Some("https://cdn.discordapp.com/embed/avatars/4.png".to_string()),
                is_speaking: false,
                is_muted: false,
                is_deafened: true,
                volume_level: 0.0,
            },
        ];

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.alt_held, "Hold ALT simulation (Interact/Move/Resize)");
            ui.checkbox(&mut self.config.only_speaking, "Only speaking");
            if ui
                .selectable_label(
                    self.config.style == waycord::OverlayStyle::Transparent,
                    "Transparent Mode",
                )
                .clicked()
            {
                self.config.style = waycord::OverlayStyle::Transparent;
            }
            if ui
                .selectable_label(
                    self.config.style == waycord::OverlayStyle::Bubble,
                    "Bubble",
                )
                .clicked()
            {
                self.config.style = waycord::OverlayStyle::Bubble;
            }
            if ui
                .selectable_label(
                    self.config.style == waycord::OverlayStyle::Card,
                    "Card",
                )
                .clicked()
            {
                self.config.style = waycord::OverlayStyle::Card;
            }
            if ui.button("Settings").clicked() {
                self.show_settings = !self.show_settings;
            }
        });
        ui.add_space(8.0);

        let screen_rect = ui.clip_rect();
        let monitors = [waycord::presets::MonitorBounds::new(
            0.0,
            0.0,
            screen_rect.width(),
            screen_rect.height(),
        )];
        waycord::render_voice_overlay(
            ui.ctx(),
            &self.palette,
            &users,
            "General / Gaming Voice",
            time,
            &mut self.config,
            self.alt_held,
            &mut self.show_settings,
            screen_rect.width(),
            screen_rect.height(),
            true,
            &mut self.demo_mode,
            &self.avatar_cache,
            &monitors,
        );

        ui.ctx().request_repaint();
    }
}

fn card(ui: &mut egui::Ui, palette: &ThemePalette, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(palette.surface_elevated)
        .stroke(Stroke::new(1.0, palette.border))
        .inner_margin(Margin::same(12))
        .show(ui, add_contents);
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("UI & Theme Preview"),
        ..Default::default()
    };

    eframe::run_native(
        "UI Preview",
        native_options,
        Box::new(|cc| Ok(Box::new(UiPreviewApp::new(cc)))),
    )
}
