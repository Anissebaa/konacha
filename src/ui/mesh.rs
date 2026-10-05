use egui::{Color32, Context, Mesh, Painter, Pos2, Rect, Shape, Vec2};
use crate::theme::palette::Palette;

/// Paints the ambient gradient mesh that sits behind all glass panels.
pub fn paint(ctx: &Context, rect: Rect, palette: &Palette, animated: bool, time: f64) {
    let painter = ctx.layer_painter(egui::LayerId::background());

    // Solid base
    painter.rect_filled(rect, 0.0, palette.bg_0);

    if !animated { return; }

    // Three soft radial blobs that slowly drift.
    let w = rect.width();
    let h = rect.height();

    let t = time as f32 * 0.06;

    blob(&painter,
         Pos2::new(w * (0.18 + 0.03 * t.sin()), h * (0.08 + 0.02 * t.cos())),
         w.max(h) * 0.62, palette.mesh_1);

    blob(&painter,
         Pos2::new(w * (0.86 + 0.03 * (t * 0.7).cos()), h * (0.14 + 0.03 * (t * 0.9).sin())),
         w.max(h) * 0.55, palette.mesh_2);

    blob(&painter,
         Pos2::new(w * (0.50 + 0.04 * (t * 1.3).sin()), h * (1.05 + 0.02 * (t * 0.8).cos())),
         w.max(h) * 0.68, palette.mesh_3);
}

fn blob(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let mut mesh = Mesh::default();
    let segments: u32 = 56;

    let edge = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0);

    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: Pos2::ZERO,
        color,
    });
    for i in 0..=segments {
        let ang = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let p = center + Vec2::new(ang.cos(), ang.sin()) * radius;
        mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: Pos2::ZERO, color: edge });
    }
    for i in 1..=segments {
        mesh.indices.push(0);
        mesh.indices.push(i);
        mesh.indices.push(if i == segments { 1 } else { i + 1 });
    }
    painter.add(Shape::mesh(mesh));
}
