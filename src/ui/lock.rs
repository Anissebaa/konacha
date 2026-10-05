use egui::{Color32, Context, Frame, Margin, Rounding, Stroke, Vec2};

use crate::app::{KonachaApp, Screen, ToastKind};
use crate::config::paths;
use crate::storage::vault::Vault;
use crate::ui::widgets;

pub fn show(app: &mut KonachaApp, ctx: &Context) {
    let p = app.palette;
    let screen = ctx.screen_rect();
    let card = egui::Rect::from_center_size(
        screen.center(),
                                            Vec2::new(400.0, if app.lock_creating { 300.0 } else { 240.0 }),
    );

    egui::Area::new(egui::Id::new("konacha-lock"))
    .fixed_pos(card.min)
    .show(ctx, |ui| {
        ui.set_min_size(card.size());
        ui.set_max_size(card.size());

        Frame::new()
        .fill(p.panel_strong)
        .stroke(Stroke::new(1.0, p.border))
        .rounding(Rounding::same(16.0))
        .inner_margin(Margin::symmetric(28, 26))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("◈ Konacha")
                .color(p.accent_1).size(22.0).strong());
                ui.add_space(10.0);

                let creating = app.lock_creating;
                let sub = if creating {
                    "Create a master passphrase (cannot be recovered)"
                } else {
                    "Enter your master passphrase"
                };
                ui.label(egui::RichText::new(sub).color(p.text_dim).size(12.5));
                ui.add_space(14.0);
            });

            ui.add_space(4.0);

            let resp = ui.add_sized(
                Vec2::new(ui.available_width(), 34.0),
                                    egui::TextEdit::singleline(&mut app.lock_pass)
                                    .password(true)
                                    .hint_text("Master passphrase"),
            );
            if resp.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                submit(app, ctx);
            }
            resp.request_focus();

            if app.lock_creating {
                ui.add_space(8.0);
                ui.add_sized(
                    Vec2::new(ui.available_width(), 34.0),
                             egui::TextEdit::singleline(&mut app.lock_pass2)
                             .password(true)
                             .hint_text("Confirm passphrase"),
                );
            }

            if let Some(err) = &app.lock_error {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(err)
                .color(Color32::from_rgb(255, 100, 130)).size(12.0));
            }

            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                let label = if app.lock_creating { "Create Vault" } else { "Unlock" };
                if widgets::primary_button(ui, &p, label).clicked() {
                    submit(app, ctx);
                }
            });
        });
    });
}

fn submit(app: &mut KonachaApp, ctx: &Context) {
    app.lock_error = None;
    if app.lock_creating {
        if app.lock_pass.len() < 8 {
            app.lock_error = Some("Passphrase must be at least 8 characters.".into());
            return;
        }
        if app.lock_pass != app.lock_pass2 {
            app.lock_error = Some("Passphrases do not match.".into());
            return;
        }
        match app.create_vault(&app.lock_pass.clone()) {
            Ok(()) => {
                app.lock_pass.clear();
                app.lock_pass2.clear();
                app.screen = Screen::Main;
                app.refresh_notes();
                app.toast("Vault created", ToastKind::Info, ctx);
            }
            Err(e) => app.lock_error = Some(e.to_string()),
        }
    } else {
        match app.unlock_vault(&app.lock_pass.clone()) {
            Ok(()) => {
                app.lock_pass.clear();
                app.lock_pass2.clear();
                app.screen = Screen::Main;
                app.refresh_notes();
                app.probe_ai(ctx);
            }
            Err(e) => app.lock_error = Some(e.to_string()),
        }
    }
    ctx.request_repaint();
}

// Ensure the vault existence check updates `lock_creating` on entry.
pub fn refresh_mode(app: &mut KonachaApp) {
    app.lock_creating = !Vault::exists(&paths::vault_root());
}
