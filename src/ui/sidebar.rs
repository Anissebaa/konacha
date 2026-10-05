use egui::{Color32, Context, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::{KonachaApp, Screen};
use crate::ui::widgets;

pub fn show(app: &mut KonachaApp, ui: &mut Ui, ctx: &Context, rect: Rect) {
    let p = app.palette;

    // Glass background
    ui.painter().rect_filled(rect, Rounding::same(14.0), p.panel);
    ui.painter().rect_stroke(rect, Rounding::same(14.0), Stroke::new(1.0, p.border));

    ui.scope_builder(
        egui::UiBuilder::new()
        .max_rect(rect.shrink2(Vec2::new(12.0, 12.0)))
        .layout(egui::Layout::top_down(egui::Align::Min)),
                     |ui| {
                         // ── header ─────────────────────────────────────────────────
                         ui.horizontal(|ui| {
                             ui.label(egui::RichText::new("◈ Konacha")
                             .color(p.accent_1).size(15.0).strong());
                             ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                 if widgets::icon_button(ui, &p, "＋", "New note  (⌘/Ctrl+N)").clicked() {
                                     app.new_note();
                                     ctx.request_repaint();
                                 }
                             });
                         });

                         ui.add_space(8.0);

                         // ── search row ─────────────────────────────────────────────
                         ui.horizontal(|ui| {
                             let search_w = ui.available_width() - 54.0;
                             let resp = ui.add_sized(
                                 Vec2::new(search_w, 28.0),
                                                     egui::TextEdit::singleline(&mut app.search)
                                                     .hint_text("Search notes…")
                                                     .desired_width(search_w),
                             );
                             if resp.changed() { app.refresh_notes(); }

                             ui.add_space(6.0);
                             let online = app.ai_online;
                             widgets::pill(ui, &p, if online { "AI ●" } else { "AI ○" }, online);
                         });

                         ui.add_space(6.0);

                         // ── tag chips ──────────────────────────────────────────────
                         let tags = app.tags();
                         if !tags.is_empty() {
                             egui::ScrollArea::horizontal()
                             .id_salt("tagbar")
                             .max_height(28.0)
                             .show(ui, |ui| {
                                 ui.horizontal(|ui| {
                                     for t in &tags {
                                         let active = app.active_tag.as_deref() == Some(t.as_str());
                                         let label = format!("#{t}");
                                         let col = if active { p.text } else { p.accent_1 };
                                         let bg = if active {
                                             Color32::from_rgba_unmultiplied(
                                                 p.accent_1.r(), p.accent_1.g(), p.accent_1.b(), 60)
                                         } else {
                                             Color32::from_rgba_unmultiplied(
                                                 p.accent_1.r(), p.accent_1.g(), p.accent_1.b(), 22)
                                         };
                                         let galley = ui.painter().layout_no_wrap(
                                             label, egui::FontId::proportional(11.0), col);
                                         let size = galley.size() + Vec2::new(14.0, 6.0);
                                         let (r, resp) = ui.allocate_exact_size(size, Sense::click());
                                         ui.painter().rect_filled(r, Rounding::same(999.0), bg);
                                         ui.painter().rect_stroke(r, Rounding::same(999.0),
                                                                  Stroke::new(1.0, p.border_hi));
                                         let pos = r.center() - galley.size() / 2.0;
                                         ui.painter().galley(pos, galley, col);
                                         if resp.clicked() {
                                             app.active_tag = if active { None } else { Some(t.clone()) };
                                             app.refresh_notes();
                                         }
                                         ui.add_space(4.0);
                                     }
                                 });
                             });
                             ui.add_space(4.0);
                         }

                         widgets::divider(ui, &p);
                         ui.add_space(4.0);

                         // ── note list ──────────────────────────────────────────────
                         let item_h = if app.prefs.compact { 40.0 } else { 62.0 };
                         let current_id = app.current.as_ref().map(|c| c.id);

                         egui::ScrollArea::vertical()
                         .id_salt("note-list")
                         .auto_shrink([false, false])
                         .show(ui, |ui| {
                             let notes: Vec<_> = app.notes.iter().cloned().collect();
                             if notes.is_empty() {
                                 ui.add_space(30.0);
                                 ui.vertical_centered(|ui| {
                                     ui.label(egui::RichText::new("No notes yet")
                                     .color(p.text_mute).size(12.0));
                                 });
                                 return;
                             }

                             for meta in notes {
                                 draw_note_row(app, ui, &p, &meta, current_id, item_h, ctx);
                                 ui.add_space(4.0);
                             }
                         });

                         // ── footer ─────────────────────────────────────────────────
                         ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                             ui.horizontal(|ui| {
                                 let mut vim = app.prefs.vim_mode;
                                 if widgets::toggle(ui, &p, &mut vim, "Vim mode", ctx) {
                                     app.prefs.vim_mode = vim;
                                     if let Some(v) = app.vault.as_mut() { v.vim_mode = vim; }
                                     let _ = app.prefs.save();
                                 }
                                 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                     if widgets::ghost_button(ui, &p, "Lock").clicked() {
                                         app.commit_pending();
                                         app.lock_vault();
                                         app.screen = Screen::Lock;
                                         crate::ui::lock::refresh_mode(app);
                                         ctx.request_repaint();
                                     }
                                     if widgets::ghost_button(ui, &p, "⚙").clicked() {
                                         app.setup_scratch = app.prefs.clone();
                                         app.screen = Screen::Setup;
                                         ctx.request_repaint();
                                     }
                                 });
                             });
                             ui.add_space(6.0);
                         });
                     });
}

fn draw_note_row(
    app: &mut KonachaApp,
    ui: &mut Ui,
    p: &crate::theme::palette::Palette,
    meta: &crate::storage::models::NoteMeta,
    current_id: Option<Uuid>,
    h: f32,
    ctx: &Context,
) {
    let is_active = current_id == Some(meta.id);
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), h),
                                              Sense::click(),
    );

    let hovered = resp.hovered();
    let fill = if is_active {
        Color32::from_rgba_unmultiplied(p.accent_1.r(), p.accent_1.g(), p.accent_1.b(), 42)
    } else if hovered {
        Color32::from_rgba_unmultiplied(255, 255, 255, if p.scheme == crate::theme::palette::Scheme::Light { 20 } else { 8 })
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, Rounding::same(9.0), fill);
    if is_active {
        ui.painter().rect_stroke(rect, Rounding::same(9.0),
                                 Stroke::new(1.0, p.border_hi));
    }

    let text_rect = rect.shrink2(Vec2::new(10.0, 8.0));
    let mut x = text_rect.left();
    let mut title_y = text_rect.top();

    if meta.pinned {
        ui.painter().text(egui::pos2(x, title_y),
                          egui::Align2::LEFT_TOP, "📌",
                          egui::FontId::proportional(11.0), p.accent_3);
        x += 18.0;
    }

    let title_color = if is_active { p.text } else { p.text };
    ui.painter().text(egui::pos2(x, title_y),
                      egui::Align2::LEFT_TOP,
                      truncate(&meta.title, 34),
                      egui::FontId::proportional(13.0),
                      title_color);

    title_y += 17.0;

    if !app.prefs.compact {
        ui.painter().text(egui::pos2(text_rect.left(), title_y),
                          egui::Align2::LEFT_TOP,
                          truncate(&meta.excerpt, 46),
                          egui::FontId::proportional(11.0),
                          p.text_dim);
        title_y += 15.0;
        let ts = fmt_time(&meta.updated_at);
        ui.painter().text(egui::pos2(text_rect.left(), title_y),
                          egui::Align2::LEFT_TOP,
                          ts,
                          egui::FontId::proportional(10.0),
                          p.text_mute);
    }

    if resp.clicked() {
        app.open_note(meta.id);
        ctx.request_repaint();
    }

    // Right-click → delete
    resp.context_menu(|ui| {
        if ui.button("Delete note").clicked() {
            app.delete_note(meta.id);
            ui.close_menu();
            ctx.request_repaint();
        }
        if ui.button(if meta.pinned { "Unpin" } else { "Pin" }).clicked() {
            app.toggle_pin(meta.id);
            ui.close_menu();
            ctx.request_repaint();
        }
    });
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ");
    if s.chars().count() <= max { s }
    else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}…")
    }
}

fn fmt_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(*dt);
    let secs = diff.num_seconds();
    if secs < 60 { "just now".into() }
    else if secs < 3600 { format!("{}m ago", secs / 60) }
    else if secs < 86400 { format!("{}h ago", secs / 3600) }
    else if secs < 604_800 { format!("{}d ago", secs / 86400) }
    else { dt.format("%Y-%m-%d").to_string() }
}
