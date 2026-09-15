use egui::{Color32, Stroke};

use crate::{avatar_cache::AvatarCache, theme::ThemePalette};

#[derive(Clone, Debug)]
pub struct VoiceUser {
    pub id: String,
    pub name: String,
    pub avatar_initials: String,
    pub avatar_url: Option<String>,
    pub is_speaking: bool,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub volume_level: f32,
}

pub fn render_avatar(
    ctx: &egui::Context,
    painter: &egui::Painter,
    palette: &ThemePalette,
    user: &VoiceUser,
    center: egui::Pos2,
    radius: f32,
    avatar_cache: &AvatarCache,
) {
    let mut rendered_image = false;

    if let Some(color_image) = avatar_cache.get(&user.id) {
        let texture = ctx.load_texture(
            format!("avatar_{}", user.id),
            (*color_image).clone(),
            egui::TextureOptions::LINEAR,
        );

        let segments = 32;
        let mut mesh = egui::Mesh::with_texture(texture.id());
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center,
            uv: egui::pos2(0.5, 0.5),
            color: Color32::WHITE,
        });
        for i in 0..=segments {
            let angle = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let (sin, cos) = angle.sin_cos();
            let pos = center + egui::vec2(cos * radius, sin * radius);
            let uv = egui::pos2(0.5 + cos * 0.5, 0.5 + sin * 0.5);
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv,
                color: Color32::WHITE,
            });
        }
        for i in 1..=segments {
            mesh.add_triangle(0, i as u32, (i + 1) as u32);
        }
        painter.add(mesh);
        rendered_image = true;
    }

    if !rendered_image {
        painter.circle_filled(center, radius, palette.surface_elevated);
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            &user.avatar_initials,
            egui::FontId::proportional(radius * 0.75),
            if user.is_speaking {
                Color32::WHITE
            } else {
                palette.text_primary
            },
        );

        if let Some(url) = &user.avatar_url {
            avatar_cache.queue_download(user.id.clone(), url.clone());
        }
    }

    painter.circle_stroke(
        center,
        radius,
        Stroke::new(
            1.2,
            if user.is_speaking {
                Color32::from_rgb(35, 165, 90)
            } else {
                palette.border
            },
        ),
    );
}
