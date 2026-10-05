use egui::{Context, Rect, Rounding, Stroke, Ui, Vec2};

use crate::app::KonachaApp;
use crate::ui::{markdown, widgets};

pub fn show(app: &mut KonachaApp, ui: &mut Ui, ctx: &Context, rect: Rect) {
    let p = app.palette;

    ui.painter().rect_filled(rect, Rounding::same(14.0), p.panel);
    ui.painter().rect_stroke(rect, Rounding::same(14.0), Stroke::new(1.0, p.border));

    let inner = rect.shrink2(Vec2::new(14.0, 12.0));

    ui.scope_builder(
        egui::UiBuilder::new()
        .max_rect(inner)
        .layout(egui::Layout::top_down(egui::Align::Min)),
                     |ui| {
                         // ── toolbar ────────────────────────────────────────────────
                         toolbar(app, ui, ctx);
                         ui.add_space(6.0);
                         widgets::divider(ui, &p);
                         ui.add_space(6.0);

                         // ── split pane ─────────────────────────────────────────────
                         let split_rect = ui.available_rect_before_wrap();
                         let half = (split_rect.width() - 10.0) / 2.0;
                         let left_rect = Rect::from_min_size(split_rect.min, Vec2::new(half, split_rect.height()));
                         let right_rect = Rect::from_min_size(
                             egui::pos2(split_rect.left() + half + 10.0, split_rect.top()),
                                                              Vec2::new(half, split_rect.height()),
                         );

                         // Left: raw editor
                         let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
                         editor_pane(app, &mut left_ui, ctx, left_rect);

                         // Divider
                         let div_x = split_rect.left() + half + 5.0;
                         ui.painter().rect_filled(
                             Rect::from_min_size(egui::pos2(div_x, split_rect.top()),
                                                 Vec2::new(1.0, split_rect.height())),
                                                  0.0, p.border,
                         );

                         // Right: preview
                         let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
                         preview_pane(app, &mut right_ui, right_rect);
                     });

    // ── statusbar ─────────────────────────────────────────────────────
    let status_rect = Rect::from_min_size(
        egui::pos2(rect.left() + 14.0, rect.bottom() - 22.0),
                                          Vec2::new(rect.width() - 28.0, 18.0),
    );
    let words = app.current.as_ref()
    .map(|c| c.content.split_whitespace().count())
    .unwrap_or(0);
    let chars = app.current.as_ref().map(|c| c.content.chars().count()).unwrap_or(0);
    let left_txt = format!("{words} words · {chars} chars{}",
                           if app.dirty { " · unsaved" } else { "" });
    ui.painter().text(status_rect.left_center(),
                      egui::Align2::LEFT_CENTER, left_txt,
                      egui::FontId::proportional(10.5), p.text_mute);
    if let Some(c) = &app.current {
        ui.painter().text(status_rect.right_center(),
                          egui::Align2::RIGHT_CENTER, format!("id {}", &c.id.to_string()[..8]),
                          egui::FontId::proportional(10.5), p.text_mute);
    }
}

fn toolbar(app: &mut KonachaApp, ui: &mut Ui, ctx: &Context) {
    let p = app.palette;

    let Some(note) = app.current.as_mut() else {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Select or create a note →")
            .color(p.text_mute).size(13.0));
        });
        return;
    };

    ui.horizontal(|ui| {
        // Title
        let title_w = (ui.available_width() - 340.0).max(160.0);
        let resp = ui.add_sized(
            Vec2::new(title_w, 28.0),
                                egui::TextEdit::singleline(&mut note.title)
                                .hint_text("Untitled")
                                .font(egui::FontId::proportional(17.0))
                                .desired_width(title_w),
        );
        if resp.changed() { app.dirty = true; }

        ui.add_space(6.0);

        // Tags
        let mut tags_str = note.tags.join(", ");
        let resp = ui.add_sized(
            Vec2::new(160.0, 28.0),
                                egui::TextEdit::singleline(&mut tags_str)
                                .hint_text("tags, comma, separated")
                                .desired_width(160.0)
                                .font(egui::FontId::proportional(12.0)),
        );
        if resp.changed() {
            note.tags = tags_str
            .split(',')
            .map(|s| s.trim().trim_start_matches('#').to_string())
            .filter(|s| !s.is_empty())
            .collect();
            app.dirty = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::primary_button(ui, &p, "Save").clicked() {
                app.commit_pending();
                app.refresh_notes();
                ctx.request_repaint();
            }
            if widgets::icon_button(ui, &p, "↧", "Export .md").clicked() {
                export_md(app, ctx);
            }
            if widgets::icon_button(ui, &p, "✨", "Summarize with local AI").clicked() {
                if let Some(c) = &app.current {
                    let id = c.id;
                    app.request_summary(ctx, id);
                }
            }
            if widgets::icon_button(ui, &p, "📌", "Pin").clicked() {
                if let Some(c) = &app.current {
                    let id = c.id;
                    app.toggle_pin(id);
                }
            }
        });
    });
}

fn editor_pane(app: &mut KonachaApp, ui: &mut Ui, ctx: &Context, rect: Rect) {
    let p = app.palette;

    ui.painter().rect_filled(rect, Rounding::same(10.0), p.bg_inset);
    ui.painter().rect_stroke(rect, Rounding::same(10.0), Stroke::new(1.0, p.border));

    let inner = rect.shrink(10.0);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
        .max_rect(inner)
        .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::ScrollArea::vertical()
    .id_salt("editor-scroll")
    .auto_shrink([false, false])
    .show(&mut child, |ui| {
        if let Some(note) = app.current.as_mut() {
            let font = egui::FontId::monospace(app.prefs.editor_font_size);
            let resp = ui.add(
                egui::TextEdit::multiline(&mut note.content)
                .font(font)
                .text_color(p.text)
                .desired_width(f32::INFINITY)
                .desired_rows(20)
                .lock_focus(true)
                .frame(false)
                .hint_text("# Start writing…\n\nMarkdown is fully supported."),
            );
            if resp.changed() { app.dirty = true; }

            // Vim escape
            if app.prefs.vim_mode
                && resp.has_focus()
                && ctx.input(|i| i.key_pressed(egui::Key::Escape))
                {
                    resp.surrender_focus();
                }
        } else {
            ui.label(egui::RichText::new("No note open")
            .color(p.text_mute).size(12.0));
        }
    });
}

fn preview_pane(app: &mut KonachaApp, ui: &mut Ui, rect: Rect) {
    let p = app.palette;

    ui.painter().rect_filled(rect, Rounding::same(10.0), p.bg_inset);
    ui.painter().rect_stroke(rect, Rounding::same(10.0), Stroke::new(1.0, p.border));

    let inner = rect.shrink(14.0);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
        .max_rect(inner)
        .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::ScrollArea::vertical()
    .id_salt("preview-scroll")
    .auto_shrink([false, false])
    .show(&mut child, |ui| {
        let content = app.current.as_ref().map(|c| c.content.clone()).unwrap_or_default();
        markdown::render(ui, &p, &content);
    });
}

fn export_md(app: &mut KonachaApp, ctx: &Context) {
    let Some(note) = &app.current else { return; };
    let default_name = format!("{}.md",
                               note.title.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
                               .collect::<String>().trim().replace(' ', "_"));

    let path = rfd::FileDialog::new()
    .set_file_name(&default_name)
    .add_filter("Markdown", &["md"])
    .save_file();

    if let Some(p) = path {
        let body = format!("# {}\n\n{}\n", note.title, note.content);
        match crate::storage::atomic::write_atomic(&p, body.as_bytes()) {
            Ok(()) => app.toast(format!("Exported → {}", p.display()),
                                crate::app::ToastKind::Info, ctx),
                                Err(e) => app.toast(format!("Export failed: {e}"),
                                                    crate::app::ToastKind::Error, ctx),
        }
    }
}
