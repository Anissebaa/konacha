use egui::{Color32, Context, Frame, Margin, Rounding, ScrollArea, Stroke, Ui, Vec2};

use crate::app::{KonachaApp, Screen, ToastKind};
use crate::config::prefs::Profile;
use crate::theme::palette::{Palette, ThemeId};
use crate::ui::widgets;

pub fn show(app: &mut KonachaApp, ctx: &Context) {
    let p = app.palette;
    let screen = ctx.screen_rect();
    let rect = egui::Rect::from_center_size(screen.center(), Vec2::new(960.0, 700.0).min(screen.size() - Vec2::splat(60.0)));

    egui::Area::new(egui::Id::new("konacha-setup"))
    .fixed_pos(rect.min)
    .show(ctx, |ui| {
        ui.set_min_size(rect.size());
        ui.set_max_size(rect.size());

        Frame::new()
        .fill(p.panel_strong)
        .stroke(Stroke::new(1.0, p.border))
        .rounding(Rounding::same(16.0))
        .inner_margin(Margin::symmetric(26, 22))
        .show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                header(ui, &p, app.prefs.onboarding_done);
                ui.add_space(8.0);
                profiles(ui, &p, app, ctx);
                ui.add_space(18.0);
                themes(ui, &p, app, ctx);
                ui.add_space(18.0);
                motion_section(ui, &p, app, ctx);
                ui.add_space(18.0);
                system_section(ui, &p, app, ctx);
                ui.add_space(18.0);
                footer(ui, &p, app, ctx);
            });
        });
    });
}

fn header(ui: &mut Ui, p: &Palette, done: bool) {
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("◈ Konacha")
        .color(p.accent_1).size(20.0).strong());
        let title = if done { "Preferences" } else { "Welcome — let's tune the vibe" };
        ui.add_space(6.0);
        ui.label(egui::RichText::new(title).color(p.text).size(24.0).strong());
        ui.add_space(4.0);
        let sub = if done { "Optimise performance, appearance and behaviour" }
        else   { "You can change any of this later from the tray menu" };
        ui.label(egui::RichText::new(sub).color(p.text_dim).size(12.5));
    });
}

fn section_label(ui: &mut Ui, p: &Palette, text: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(text.to_uppercase())
        .color(p.text_dim).size(11.0).strong());
        let w = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 1.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 0.0, p.border);
    });
}

fn profiles(ui: &mut Ui, p: &Palette, app: &mut KonachaApp, ctx: &Context) {
    section_label(ui, p, "Performance profile");
    ui.add_space(8.0);

    let variants = [
        (Profile::Performance, "⚡ Performance", "No blur · minimal motion · best on old GPUs"),
        (Profile::Balanced,    "⚖ Balanced",     "Recommended · glass on · 260 ms transitions"),
        (Profile::Beauty,      "✨ Beauty",       "Deep blur · neon glow · full motion"),
    ];

    let avail = ui.available_width();
    let card_w = (avail - 20.0) / 3.0;

    ui.horizontal(|ui| {
        for (prof, title, sub) in variants {
            let active = app.setup_scratch.profile == prof;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, 78.0), egui::Sense::click());
            let hovered = resp.hovered();

            let fill = if active {
                Color32::from_rgba_unmultiplied(p.accent_1.r(), p.accent_1.g(), p.accent_1.b(), 26)
            } else if hovered {
                p.panel_hover
            } else {
                p.bg_inset
            };
            ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
            ui.painter().rect_stroke(rect, Rounding::same(10.0),
                                     Stroke::new(if active { 1.5 } else { 1.0 },
                                                 if active { p.border_hi } else { p.border }));

            let pad = Vec2::new(14.0, 12.0);
            let text_pos = rect.min + pad;
            let color = p.text;
            ui.painter().text(text_pos, egui::Align2::LEFT_TOP, title,
                              egui::FontId::proportional(13.5), color);
            ui.painter().text(text_pos + Vec2::new(0.0, 22.0), egui::Align2::LEFT_TOP, sub,
                              egui::FontId::proportional(11.5), p.text_dim);

            if active {
                ui.painter().text(rect.right_top() + Vec2::new(-14.0, 12.0),
                                  egui::Align2::RIGHT_TOP, "✓",
                                  egui::FontId::proportional(14.0), p.accent_1);
            }

            if resp.clicked() {
                app.setup_scratch.profile = prof;
                ctx.request_repaint();
            }
            ui.add_space(10.0);
        }
    });
}

fn themes(ui: &mut Ui, p: &Palette, app: &mut KonachaApp, ctx: &Context) {
    ui.horizontal(|ui| {
        section_label(ui, p, "Theme");
        ui.add_space(8.0);
        ui.label(egui::RichText::new(format!("{}", ThemeId::ALL.len()))
        .color(p.accent_1).size(10.0));
    });
    ui.add_space(8.0);

    let avail = ui.available_width();
    let card_w = (avail - 20.0) / 3.0;
    let card_h = 90.0;

    egui::Grid::new("theme-grid")
    .num_columns(3)
    .spacing([10.0, 10.0])
    .show(ui, |ui| {
        for (i, id) in ThemeId::ALL.iter().enumerate() {
            let id = *id;
            let active = app.setup_scratch.theme == id;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, card_h), egui::Sense::click());
            let hovered = resp.hovered();

            let fill = if active {
                Color32::from_rgba_unmultiplied(p.accent_1.r(), p.accent_1.g(), p.accent_1.b(), 26)
            } else if hovered {
                p.panel_hover
            } else {
                p.bg_inset
            };
            ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
            ui.painter().rect_stroke(rect, Rounding::same(10.0),
                                     Stroke::new(if active { 1.5 } else { 1.0 },
                                                 if active { p.border_hi } else { p.border }));

            // Swatch preview
            let sw = id.swatch();
            let sw_rect = egui::Rect::from_min_size(rect.min + Vec2::splat(10.0),
                                                    Vec2::new(rect.width() - 20.0, 40.0));
            let inner_p = id.palette();
            ui.painter().rect_filled(sw_rect, Rounding::same(7.0), inner_p.bg_0);

            // Two accent dots
            ui.painter().circle_filled(
                sw_rect.left_center() + Vec2::new(16.0, 0.0), 8.0, inner_p.accent_1);
            ui.painter().circle_filled(
                sw_rect.left_center() + Vec2::new(34.0, 0.0), 8.0, inner_p.accent_2);

            let _ = sw;
            let name_y = sw_rect.bottom() + 8.0;
            ui.painter().text(egui::pos2(rect.left() + 12.0, name_y),
                              egui::Align2::LEFT_TOP, id.name(),
                              egui::FontId::proportional(12.5), p.text);
            ui.painter().text(egui::pos2(rect.left() + 12.0, name_y + 15.0),
                              egui::Align2::LEFT_TOP, id.blurb(),
                              egui::FontId::proportional(10.5), p.text_dim);

            if active {
                ui.painter().text(rect.right_top() + Vec2::new(-12.0, 12.0),
                                  egui::Align2::RIGHT_TOP, "✓",
                                  egui::FontId::proportional(13.0), p.accent_1);
            }

            if resp.clicked() {
                app.setup_scratch.theme = id;
                app.set_target_only(id);   // preview instantly, no prefs write
                ctx.request_repaint();
            }
            if resp.hovered() && !active {
                app.set_target_only(id);
            }
            if resp.hovered() == false && !active {
                app.set_target_only(app.setup_scratch.theme);
            }

            if (i + 1) % 3 == 0 { ui.end_row(); }
        }
    });
}

fn motion_section(ui: &mut Ui, p: &Palette, app: &mut KonachaApp, ctx: &Context) {
    section_label(ui, p, "Motion");
    ui.add_space(8.0);

    let mut animations = app.setup_scratch.animations;
    let mut ambient    = app.setup_scratch.ambient_mesh;
    let mut speed      = app.setup_scratch.animation_speed;

    ui.horizontal(|ui| {
        if widgets::toggle(ui, p, &mut animations, "Enable animations", ctx) {
            app.setup_scratch.animations = animations;
        }
        ui.add_space(24.0);
        if widgets::toggle(ui, p, &mut ambient, "Ambient gradient mesh", ctx) {
            app.setup_scratch.ambient_mesh = ambient;
        }
    });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Animation speed").color(p.text_dim).size(12.5));
        let resp = ui.add(egui::Slider::new(&mut speed, 0.5..=1.5).step_by(0.05)
        .show_value(true)
        .trailing_fill(true));
        if resp.changed() {
            app.setup_scratch.animation_speed = speed;
        }
    });
}

fn system_section(ui: &mut Ui, p: &Palette, app: &mut KonachaApp, ctx: &Context) {
    section_label(ui, p, "System");
    ui.add_space(8.0);

    let mut compact = app.setup_scratch.compact;
    let mut tray    = app.setup_scratch.close_to_tray;
    let mut vim     = app.setup_scratch.vim_mode;

    ui.horizontal(|ui| {
        if widgets::toggle(ui, p, &mut compact, "Compact sidebar rows", ctx) {
            app.setup_scratch.compact = compact;
        }
        ui.add_space(24.0);
        if widgets::toggle(ui, p, &mut tray, "Minimise to tray on close", ctx) {
            app.setup_scratch.close_to_tray = tray;
        }
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if widgets::toggle(ui, p, &mut vim, "Vim mode by default", ctx) {
            app.setup_scratch.vim_mode = vim;
        }
    });
}

fn footer(ui: &mut Ui, p: &Palette, app: &mut KonachaApp, ctx: &Context) {
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, p.border);
    ui.add_space(12.0);

    ui.horizontal(|ui| {
        if widgets::ghost_button(ui, p, "Reset defaults").clicked() {
            app.setup_scratch = crate::config::prefs::Preferences::default();
            app.set_target_only(app.setup_scratch.theme);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let done = app.prefs.onboarding_done;
            let next_label = if done { "Save" } else { "Create my vault →" };
            let skip_label = if done { "Cancel" } else { "Skip" };

            if widgets::primary_button(ui, p, next_label).clicked() {
                // commit
                app.prefs = app.setup_scratch.clone();
                app.prefs.onboarding_done = true;
                let _ = app.prefs.save();
                app.set_target_only(app.prefs.theme);
                app.screen = if done { Screen::Main } else { Screen::Lock };
                ctx.request_repaint();
            }
            ui.add_space(8.0);
            if widgets::ghost_button(ui, p, skip_label).clicked() {
                if done {
                    // cancel — revert
                    app.setup_scratch = app.prefs.clone();
                    app.set_target_only(app.prefs.theme);
                    app.screen = Screen::Main;
                } else {
                    app.prefs.onboarding_done = true;
                    let _ = app.prefs.save();
                    app.set_target_only(app.prefs.theme);
                    app.screen = Screen::Lock;
                    app.toast("Setup skipped", ToastKind::Info, ctx);
                }
                ctx.request_repaint();
            }
        });
    });
}
