use egui::{Color32, Context, Frame, Margin, Response, Rounding, Sense, Stroke, Ui, Vec2};
use crate::app::{KonachaApp, ToastKind};
use crate::theme::palette::Palette;

// ── Glass panel ─────────────────────────────────────────────────────────

pub struct GlassOpts {
    pub rounding: f32,
    pub margin:   [i8; 2],   // [x, y]
    pub fill_over: Option<Color32>,
    pub stroke_hi: bool,
}

impl Default for GlassOpts {
    fn default() -> Self {
        Self { rounding: 14.0, margin: [12, 10], fill_over: None, stroke_hi: false }
    }
}

pub fn glass_frame(p: &Palette, opts: GlassOpts) -> Frame {
    let fill = opts.fill_over.unwrap_or(p.panel);
    let border = if opts.stroke_hi { p.border_hi } else { p.border };
    Frame::new()
    .fill(fill)
    .stroke(Stroke::new(1.0, border))
    .rounding(Rounding::same(opts.rounding))
    .inner_margin(Margin::symmetric(opts.margin[0], opts.margin[1]))
}

/// Adds a hairline highlight on the top edge — the "glass lip".
pub fn glass_lip(ui: &mut Ui, rect: egui::Rect, p: &Palette, rounding: f32) {
    let top = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 1.0));
    let _ = rounding;
    ui.painter().rect_filled(top, 0.0,
                             Color32::from_rgba_unmultiplied(255, 255, 255, if p.scheme == crate::theme::palette::Scheme::Light { 60 } else { 20 }));
}

// ── Buttons ─────────────────────────────────────────────────────────────

pub fn primary_button(ui: &mut Ui, p: &Palette, label: &str) -> Response {
    let text_color = if p.scheme == crate::theme::palette::Scheme::Light {
        Color32::WHITE
    } else {
        Color32::from_rgb(10, 12, 16)
    };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
                                             egui::FontId::proportional(13.0),
                                             text_color,
    );
    let padding = Vec2::new(16.0, 8.0);
    let size = galley.size() + padding * 2.0;

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();

    let fill = if hovered {
        Color32::from_rgba_unmultiplied(
            ((p.accent_1.r() as u16 + 30).min(255)) as u8,
                                        ((p.accent_1.g() as u16 + 30).min(255)) as u8,
                                        ((p.accent_1.b() as u16 + 30).min(255)) as u8,
                                        235,
        )
    } else {
        Color32::from_rgba_unmultiplied(p.accent_1.r(), p.accent_1.g(), p.accent_1.b(), 225)
    };

    ui.painter().rect_filled(rect, Rounding::same(9.0), fill);
    ui.painter().rect_stroke(rect, Rounding::same(9.0), Stroke::new(1.0, p.border_hi));

    let text_pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(text_pos, galley, text_color);

    response
}

pub fn ghost_button(ui: &mut Ui, p: &Palette, label: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
                                             egui::FontId::proportional(12.5),
                                             p.text_dim,
    );
    let padding = Vec2::new(12.0, 7.0);
    let size = galley.size() + padding * 2.0;

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();

    let fill = if hovered { p.panel_hover } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, Rounding::same(9.0), fill);
    ui.painter().rect_stroke(rect, Rounding::same(9.0), Stroke::new(1.0, p.border));

    let text_color = if hovered { p.text } else { p.text_dim };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(), egui::FontId::proportional(12.5), text_color,
    );
    let text_pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(text_pos, galley, text_color);

    response
}

pub fn icon_button(ui: &mut Ui, p: &Palette, glyph: &str, tooltip: &str) -> Response {
    let size = Vec2::splat(30.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();

    let fill = if hovered { p.panel_hover } else { p.accent_soft };
    ui.painter().rect_filled(rect, Rounding::same(9.0), fill);
    ui.painter().rect_stroke(rect, Rounding::same(9.0), Stroke::new(1.0, p.border_hi));

    let galley = ui.painter().layout_no_wrap(
        glyph.to_owned(),
                                             egui::FontId::proportional(15.0),
                                             p.accent_1,
    );
    let text_pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(text_pos, galley, p.accent_1);

    response.on_hover_text(tooltip)
}

pub fn pill(ui: &mut Ui, p: &Palette, label: &str, online: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(46.0, 20.0), Sense::hover());
    let fill = if online {
        Color32::from_rgba_unmultiplied(p.accent_2.r(), p.accent_2.g(), p.accent_2.b(), 30)
    } else {
        p.panel_hover
    };
    ui.painter().rect_filled(rect, Rounding::same(999.0), fill);
    ui.painter().rect_stroke(rect, Rounding::same(999.0), Stroke::new(1.0, p.border));

    let color = if online { p.accent_2 } else { p.text_mute };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(), egui::FontId::proportional(10.0), color,
    );
    let pos = rect.center() - galley.size() / 2.0;
    ui.painter().galley(pos, galley, color);
    response
}

pub fn divider(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, p.border);
}

// ── Toggle ──────────────────────────────────────────────────────────────

pub fn toggle(ui: &mut Ui, p: &Palette, on: &mut bool, label: &str, ctx: &Context) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(38.0, 22.0), Sense::click());
        if resp.clicked() {
            *on = !*on;
            changed = true;
            if changed { ctx.request_repaint(); }
        }
        let bg = if *on {
            Color32::from_rgba_unmultiplied(p.accent_1.r(), p.accent_1.g(), p.accent_1.b(), 220)
        } else {
            p.panel_hover
        };
        ui.painter().rect_filled(rect, Rounding::same(999.0), bg);
        ui.painter().rect_stroke(rect, Rounding::same(999.0), Stroke::new(1.0, p.border));

        let knob_x = if *on { rect.right() - 11.0 } else { rect.left() + 11.0 };
        let knob_c = egui::pos2(knob_x, rect.center().y);
        let knob_color = if *on { Color32::WHITE } else { p.text_dim };
        ui.painter().circle_filled(knob_c, 8.0, knob_color);

        ui.add_space(10.0);
        ui.label(egui::RichText::new(label).color(p.text).size(13.0));
    });
    changed
}

// ── Toast overlay ───────────────────────────────────────────────────────

pub fn toast_overlay(app: &mut KonachaApp, ctx: &Context) {
    let Some(toast) = &app.toast else { return; };
    let now = ctx.input(|i| i.time);
    let age = now - toast.born;
    let ttl = 3.2;
    if age > ttl {
        app.toast = None;
        return;
    }
    ctx.request_repaint();

    let alpha = if age < 0.18 {
        (age / 0.18).clamp(0.0, 1.0)
    } else if age > ttl - 0.35 {
        ((ttl - age) / 0.35).clamp(0.0, 1.0)
    } else { 1.0 };

    let (fill, border) = match toast.kind {
        ToastKind::Info  => (app.palette.panel_strong, app.palette.border_hi),
        ToastKind::Warn  => (app.palette.panel_strong, app.palette.accent_3),
        ToastKind::Error => (app.palette.panel_strong, egui::Color32::from_rgb(255, 100, 130)),
    };

    let fill = with_alpha(fill, (fill.a() as f32 * alpha) as u8);
    let border = with_alpha(border, (border.a() as f32 * alpha) as u8);

    egui::Area::new(egui::Id::new("konacha-toast"))
    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -28.0))
    .interactable(false)
    .show(ctx, |ui| {
        Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::symmetric(14, 8))
        .show(ui, |ui| {
            let col = with_alpha(app.palette.text, (app.palette.text.a() as f32 * alpha) as u8);
            ui.label(egui::RichText::new(&toast.message).color(col).size(12.5));
        });
    });
}

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

// Keeps egui from warning about unused imports when trimming features.
#[allow(dead_code)]
fn _unused(_: Rounding, _: Margin) {}
