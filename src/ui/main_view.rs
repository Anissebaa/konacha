use egui::{Context, Frame, Margin, Rounding, Stroke};

use crate::app::KonachaApp;
use crate::ui::{editor, sidebar};

pub fn show(app: &mut KonachaApp, ctx: &Context) {
    let p = app.palette;
    let screen = ctx.screen_rect();
    let pad = 12.0;
    let sidebar_w = 300.0;
    let gap = 12.0;

    egui::CentralPanel::default()
    .frame(Frame::new().fill(egui::Color32::TRANSPARENT).inner_margin(Margin::same(0)))
    .show(ctx, |ui| {
        ui.set_min_size(screen.size());

        let sidebar_rect = egui::Rect::from_min_size(
            screen.min + egui::vec2(pad, pad),
                                                     egui::vec2(sidebar_w, screen.height() - 2.0 * pad),
        );
        let main_rect = egui::Rect::from_min_size(
            egui::pos2(sidebar_rect.right() + gap, sidebar_rect.top()),
                                                  egui::vec2(screen.right() - sidebar_rect.right() - gap - pad,
                                                             sidebar_rect.height()),
        );

        // ── SIDEBAR ────────────────────────────────────────────────
        let mut sb_ui = ui.new_child(
            egui::UiBuilder::new()
            .max_rect(sidebar_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        sidebar::show(app, &mut sb_ui, ctx, sidebar_rect);

        // ── MAIN ───────────────────────────────────────────────────
        let mut main_ui = ui.new_child(
            egui::UiBuilder::new()
            .max_rect(main_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        editor::show(app, &mut main_ui, ctx, main_rect);
    });

    let _ = p;
    let _ = Rounding::same(0.0);
    let _ = Stroke::NONE;
}
