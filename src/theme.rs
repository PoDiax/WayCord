use std::sync::Arc;
use egui::{Color32, CornerRadius, Margin, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ThemeKind {
    Mocha,
    TokyoNight,
    Nord,
    Gruvbox,
    RosePine,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThemePalette {
    pub name: &'static str,
    pub background: Color32,
    pub surface: Color32,
    pub surface_elevated: Color32,
    pub accent: Color32,
    pub accent_hovered: Color32,
    pub accent_active: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub danger: Color32,
    pub success: Color32,
}

impl ThemePalette {
    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Mocha => Self {
                name: "Catppuccin Mocha",
                background: Color32::from_rgb(30, 30, 46),
                surface: Color32::from_rgb(24, 24, 37),
                surface_elevated: Color32::from_rgb(49, 50, 68),
                accent: Color32::from_rgb(137, 180, 250),
                accent_hovered: Color32::from_rgb(180, 190, 254),
                accent_active: Color32::from_rgb(116, 199, 236),
                text_primary: Color32::from_rgb(205, 214, 244),
                text_muted: Color32::from_rgb(186, 194, 222),
                border: Color32::from_rgb(69, 71, 90),
                border_subtle: Color32::from_rgb(49, 50, 68),
                danger: Color32::from_rgb(243, 139, 168),
                success: Color32::from_rgb(166, 227, 161),
            },
            ThemeKind::TokyoNight => Self {
                name: "Tokyo Night",
                background: Color32::from_rgb(26, 27, 38),
                surface: Color32::from_rgb(22, 22, 30),
                surface_elevated: Color32::from_rgb(41, 46, 66),
                accent: Color32::from_rgb(122, 162, 247),
                accent_hovered: Color32::from_rgb(125, 207, 255),
                accent_active: Color32::from_rgb(187, 154, 247),
                text_primary: Color32::from_rgb(212, 220, 255),
                text_muted: Color32::from_rgb(169, 177, 214),
                border: Color32::from_rgb(59, 66, 97),
                border_subtle: Color32::from_rgb(41, 46, 66),
                danger: Color32::from_rgb(247, 118, 142),
                success: Color32::from_rgb(158, 206, 106),
            },
            ThemeKind::Nord => Self {
                name: "Nord",
                background: Color32::from_rgb(46, 52, 64),
                surface: Color32::from_rgb(59, 66, 82),
                surface_elevated: Color32::from_rgb(67, 76, 94),
                accent: Color32::from_rgb(136, 192, 208),
                accent_hovered: Color32::from_rgb(143, 188, 187),
                accent_active: Color32::from_rgb(129, 161, 193),
                text_primary: Color32::from_rgb(236, 239, 244),
                text_muted: Color32::from_rgb(216, 222, 233),
                border: Color32::from_rgb(76, 86, 106),
                border_subtle: Color32::from_rgb(67, 76, 94),
                danger: Color32::from_rgb(191, 97, 106),
                success: Color32::from_rgb(163, 190, 140),
            },
            ThemeKind::Gruvbox => Self {
                name: "Gruvbox Dark",
                background: Color32::from_rgb(40, 40, 40),
                surface: Color32::from_rgb(29, 32, 33),
                surface_elevated: Color32::from_rgb(60, 56, 54),
                accent: Color32::from_rgb(254, 128, 25),
                accent_hovered: Color32::from_rgb(250, 189, 47),
                accent_active: Color32::from_rgb(214, 93, 14),
                text_primary: Color32::from_rgb(235, 219, 178),
                text_muted: Color32::from_rgb(198, 182, 155),
                border: Color32::from_rgb(80, 73, 69),
                border_subtle: Color32::from_rgb(60, 56, 54),
                danger: Color32::from_rgb(204, 36, 29),
                success: Color32::from_rgb(184, 187, 38),
            },
            ThemeKind::RosePine => Self {
                name: "Rosé Pine",
                background: Color32::from_rgb(25, 23, 36),
                surface: Color32::from_rgb(31, 29, 46),
                surface_elevated: Color32::from_rgb(38, 35, 58),
                accent: Color32::from_rgb(235, 188, 186),
                accent_hovered: Color32::from_rgb(246, 193, 119),
                accent_active: Color32::from_rgb(196, 167, 231),
                text_primary: Color32::from_rgb(224, 222, 244),
                text_muted: Color32::from_rgb(196, 192, 218),
                border: Color32::from_rgb(68, 65, 90),
                border_subtle: Color32::from_rgb(38, 35, 58),
                danger: Color32::from_rgb(235, 111, 146),
                success: Color32::from_rgb(156, 207, 216),
            },
        }
    }
}

pub fn contrast_text_for(bg: Color32) -> Color32 {
    let r = bg.r() as f32 / 255.0;
    let g = bg.g() as f32 / 255.0;
    let b = bg.b() as f32 / 255.0;
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if lum > 0.42 {
        Color32::from_rgb(18, 18, 24)
    } else {
        Color32::from_rgb(245, 245, 250)
    }
}

pub fn selectable_pill(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    selected: bool,
    text: &str,
) -> egui::Response {
    let (bg_color, fg_color, stroke) = if selected {
        (
            palette.accent,
            contrast_text_for(palette.accent),
            Stroke::new(1.0, palette.accent_hovered),
        )
    } else {
        (
            palette.surface_elevated,
            palette.text_primary,
            Stroke::new(1.0, palette.border),
        )
    };

    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(fg_color)
            .size(12.5)
            .strong(),
    )
    .fill(bg_color)
    .stroke(stroke)
    .corner_radius(CornerRadius::same(6));

    ui.add(button)
}

pub fn init_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidate_fonts = [
        "/usr/share/fonts/TTF/JetBrainsMonoNLNerdFont-SemiBold.ttf",
        "/usr/share/fonts/TTF/JetBrainsMonoNLNerdFont-Regular.ttf",
        "/usr/share/fonts/TTF/MesloLGLNerdFont-Italic.ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];

    for path in candidate_fonts {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "nerd_icons".to_owned(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.push("nerd_icons".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push("nerd_icons".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &egui::Context, palette: &ThemePalette, corner_rounding: f32) {
    init_fonts(ctx);

    let mut visuals = egui::Visuals::dark();

    visuals.window_fill = palette.surface;
    visuals.panel_fill = palette.background;
    visuals.extreme_bg_color = palette.surface_elevated;

    visuals.window_stroke = Stroke::new(1.0, palette.border);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.border_subtle);

    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text_primary.gamma_multiply(0.92));

    visuals.widgets.inactive.bg_fill = palette.surface_elevated;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text_primary);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.border);

    visuals.widgets.hovered.bg_fill = palette.surface_elevated.gamma_multiply(1.25);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.accent_hovered);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.2, palette.accent);

    visuals.widgets.active.bg_fill = palette.accent_active;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.background);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent_active);

    visuals.selection.bg_fill = palette.accent;
    visuals.selection.stroke = Stroke::new(1.0, contrast_text_for(palette.accent));

    let radius = CornerRadius::same(corner_rounding.round() as u8);
    visuals.widgets.noninteractive.corner_radius = radius;
    visuals.widgets.inactive.corner_radius = radius;
    visuals.widgets.hovered.corner_radius = radius;
    visuals.widgets.active.corner_radius = radius;
    visuals.window_corner_radius = CornerRadius::same((corner_rounding + 2.0).round() as u8);

    ctx.set_visuals_of(egui::Theme::Dark, visuals);

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.window_margin = Margin::same(14);
        style.spacing.button_padding = Vec2::new(12.0, 7.0);
        style.spacing.slider_width = 180.0;
        style.animation_time = 0.12;
    });
}
