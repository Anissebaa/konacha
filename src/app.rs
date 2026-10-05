use std::sync::mpsc::{self, Receiver};

use egui::{Context, Vec2};
use uuid::Uuid;

use crate::ai::ollama;
use crate::config::{paths, prefs::{Preferences, Profile}};
use crate::error::{KonachaError, Result};
use crate::platform;
use crate::storage::models::{Note, NoteMeta};
use crate::storage::vault::Vault;
use crate::theme::palette::{Palette, ThemeId};
use crate::theme::motion::smooth_to;
use crate::ui;

// ═════════════════════════════════════════════════════════════════════════

pub enum Screen {
    /// Splash, shown for ~300 ms while we load prefs.
    Splash { started: f64 },
    /// First-run optimisation menu.
    Setup,
    /// Vault unlock / create.
    Lock,
    /// Main application.
    Main,
}

pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub born: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind { Info, Warn, Error }

/// Result of a background AI job.
pub enum AiJob {
    Summarize(Uuid, Result<String>),
    Tags(Uuid, Result<Vec<String>>),
    Probe(bool),
}

// ═════════════════════════════════════════════════════════════════════════

pub struct KonachaApp {
    // ── lifecycle ────────────────────────────────────────────────────
    pub screen: Screen,
    pub prefs: Preferences,

    // ── theme ────────────────────────────────────────────────────────
    pub palette: Palette,          // currently displayed (lerped)
    pub target:  Palette,          // what we're lerping toward
    pub theme_animating: bool,

    // ── vault ────────────────────────────────────────────────────────
    pub vault: Option<Vault>,
    pub notes: Vec<NoteMeta>,
    pub current: Option<Note>,
    pub dirty: bool,
    pub last_autosave: f64,

    // ── filters ──────────────────────────────────────────────────────
    pub search: String,
    pub active_tag: Option<String>,

    // ── UX state ─────────────────────────────────────────────────────
    pub toast: Option<Toast>,
    pub ai_online: bool,
    pub ai_job: Option<Receiver<AiJob>>,
    pub lock_error: Option<String>,
    pub lock_pass: String,
    pub lock_pass2: String,
    pub lock_creating: bool,

    // ── setup wizard scratch ─────────────────────────────────────────
    pub setup_scratch: Preferences,

    // ── platform ─────────────────────────────────────────────────────
    pub tray_rx: Option<Receiver<platform::TrayEvent>>,
    pub hotkey_rx: Option<Receiver<platform::HotkeyEvent>>,

    // ── time ─────────────────────────────────────────────────────────
    pub last_frame: f64,
}

impl KonachaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let prefs = Preferences::load();
        let palette = prefs.theme.palette();
        cc.egui_ctx.set_visuals(palette.to_visuals());

        let now = cc.egui_ctx.input(|i| i.time);
        let screen = if !prefs.onboarding_done {
            Screen::Splash { started: now }
        } else if Vault::exists(&paths::vault_root()) {
            Screen::Splash { started: now }
        } else {
            Screen::Splash { started: now }
        };

        // Spawn tray + hotkeys (best-effort; ignore failures on headless).
        let (tray_tx, tray_rx) = mpsc::channel();
        let _ = platform::tray::spawn(tray_tx);
        let (hk_tx, hk_rx) = mpsc::channel();
        let _ = platform::hotkeys::spawn(hk_tx);

        Self {
            screen,
            prefs: prefs.clone(),
            palette,
            target: palette,
            theme_animating: false,

            vault: None,
            notes: vec![],
            current: None,
            dirty: false,
            last_autosave: now,

            search: String::new(),
            active_tag: None,

            toast: None,
            ai_online: false,
            ai_job: None,
            lock_error: None,
            lock_pass: String::new(),
            lock_pass2: String::new(),
            lock_creating: false,

            setup_scratch: prefs,

            tray_rx: Some(tray_rx),
            hotkey_rx: Some(hk_rx),
            last_frame: now,
        }
    }

    // ── theme transitions ────────────────────────────────────────────

    pub fn set_theme(&mut self, id: ThemeId) {
        self.prefs.theme = id;
        self.target = id.palette();
        self.theme_animating = true;
    }

    fn tick_theme(&mut self, ctx: &Context, dt: f32) {
        if !self.theme_animating && self.palette_approx_eq() { return; }
        let speed = if self.prefs.animations { 8.0 } else { 1000.0 };
        let p = &self.palette;
        let t = &self.target;
        self.palette = Palette {
            scheme:       t.scheme,
            bg_0:         smooth_color(p.bg_0,         t.bg_0,         dt, speed),
            bg_1:         smooth_color(p.bg_1,         t.bg_1,         dt, speed),
            bg_inset:     smooth_color(p.bg_inset,     t.bg_inset,     dt, speed),
            panel:        smooth_color(p.panel,        t.panel,        dt, speed),
            panel_strong: smooth_color(p.panel_strong, t.panel_strong, dt, speed),
            panel_hover:  smooth_color(p.panel_hover,  t.panel_hover,  dt, speed),
            border:       smooth_color(p.border,       t.border,       dt, speed),
            border_hi:    smooth_color(p.border_hi,    t.border_hi,    dt, speed),
            accent_1:     smooth_color(p.accent_1,     t.accent_1,     dt, speed),
            accent_2:     smooth_color(p.accent_2,     t.accent_2,     dt, speed),
            accent_3:     smooth_color(p.accent_3,     t.accent_3,     dt, speed),
            accent_soft:  smooth_color(p.accent_soft,  t.accent_soft,  dt, speed),
            text:         smooth_color(p.text,         t.text,         dt, speed),
            text_dim:     smooth_color(p.text_dim,     t.text_dim,     dt, speed),
            text_mute:    smooth_color(p.text_mute,    t.text_mute,    dt, speed),
            mesh_1:       smooth_color(p.mesh_1,       t.mesh_1,       dt, speed),
            mesh_2:       smooth_color(p.mesh_2,       t.mesh_2,       dt, speed),
            mesh_3:       smooth_color(p.mesh_3,       t.mesh_3,       dt, speed),
        };
        ctx.set_visuals(self.palette.to_visuals());
        ctx.request_repaint();
        if self.palette_approx_eq() { self.theme_animating = false; }
    }

    fn palette_approx_eq(&self) -> bool {
        self.palette.bg_0 == self.target.bg_0
        && self.palette.panel == self.target.panel
        && self.palette.accent_1 == self.target.accent_1
        && self.palette.text == self.target.text
    }

    // ── vault ops (called from UI) ───────────────────────────────────

    pub fn create_vault(&mut self, pass: &str) -> Result<()> {
        let v = Vault::create(&paths::vault_root(), pass)?;
        v.persist_index()?;
        self.vault = Some(v);
        self.notes.clear();
        self.current = None;
        Ok(())
    }

    pub fn unlock_vault(&mut self, pass: &str) -> Result<()> {
        let v = Vault::unlock(&paths::vault_root(), pass)?;
        self.notes = v.index.notes.clone();
        self.vault = Some(v);
        Ok(())
    }

    pub fn lock_vault(&mut self) {
        self.vault = None;
        self.notes.clear();
        self.current = None;
        self.dirty = false;
        self.search.clear();
        self.active_tag = None;
    }

    pub fn refresh_notes(&mut self) {
        let Some(v) = &self.vault else { return; };
        self.notes = v.search(&self.search, self.active_tag.as_deref());
    }

    pub fn tags(&self) -> Vec<String> {
        self.vault.as_ref().map(|v| v.all_tags()).unwrap_or_default()
    }

    pub fn open_note(&mut self, id: Uuid) {
        self.commit_pending();
        let Some(v) = &self.vault else { return; };
        if let Ok(note) = v.load_note(&id) {
            self.current = Some(note);
            self.dirty = false;
        }
    }

    pub fn new_note(&mut self) {
        self.commit_pending();
        let Some(v) = self.vault.as_mut() else { return; };
        let mut n = Note::new();
        if let Ok(meta) = v.save_note(&mut n) {
            self.current = Some(n);
            self.dirty = false;
            self.notes.insert(0, meta);
        }
    }

    pub fn commit_pending(&mut self) {
        if !self.dirty { return; }
        let Some(note) = self.current.clone() else { return; };
        let Some(v) = self.vault.as_mut() else { return; };
        let mut n = note;
        if let Ok(meta) = v.save_note(&mut n) {
            if let Some(slot) = self.notes.iter_mut().find(|m| m.id == n.id) {
                *slot = meta;
            }
            self.current = Some(n);
            self.dirty = false;
        }
    }

    pub fn toggle_pin(&mut self, id: Uuid) {
        self.commit_pending();
        let Some(v) = self.vault.as_mut() else { return; };
        if let Ok(mut n) = v.load_note(&id) {
            n.pinned = !n.pinned;
            let _ = v.save_note(&mut n);
            if self.current.as_ref().map(|c| c.id) == Some(id) {
                self.current = Some(n);
            }
            self.refresh_notes();
        }
    }

    pub fn delete_note(&mut self, id: Uuid) {
        let Some(v) = self.vault.as_mut() else { return; };
        let _ = v.delete_note(&id);
        if self.current.as_ref().map(|c| c.id) == Some(id) {
            self.current = None;
            self.dirty = false;
        }
        self.refresh_notes();
    }

    // ── AI ───────────────────────────────────────────────────────────

    pub fn probe_ai(&mut self, ctx: &Context) {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(AiJob::Probe(ollama::is_online()));
        });
        self.ai_job = Some(rx);
        ctx.request_repaint();
    }

    pub fn request_summary(&mut self, ctx: &Context, id: Uuid) {
        let Some(v) = &self.vault else { return; };
        let Ok(note) = v.load_note(&id) else { return; };
        let model = self.prefs.ai_model.clone();
        let content = note.content.clone();

        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let res = ollama::summarize(&content, Some(&model));
            let _ = tx.send(AiJob::Summarize(id, res));
        });
        self.ai_job = Some(rx);
        ctx.request_repaint();
    }

    fn poll_ai(&mut self, ctx: &Context) {
        let Some(rx) = &self.ai_job else { return; };
        match rx.try_recv() {
            Ok(AiJob::Probe(online)) => {
                self.ai_online = online;
                self.ai_job = None;
            }
            Ok(AiJob::Summarize(id, res)) => {
                self.ai_job = None;
                match res {
                    Ok(text) => {
                        if self.current.as_ref().map(|c| c.id) == Some(id) {
                            if let Some(c) = &mut self.current {
                                c.content.push_str("\n\n---\n\n## ✨ Summary\n\n");
                                c.content.push_str(&text);
                                c.content.push('\n');
                                self.dirty = true;
                            }
                        }
                        self.toast("Summary appended", ToastKind::Info, ctx);
                    }
                    Err(e) => self.toast(format!("AI: {e}"), ToastKind::Error, ctx),
                }
            }
            Ok(AiJob::Tags(id, res)) => {
                self.ai_job = None;
                if let Ok(tags) = res {
                    if self.current.as_ref().map(|c| c.id) == Some(id) {
                        if let Some(c) = &mut self.current {
                            for t in tags {
                                if !c.tags.contains(&t) { c.tags.push(t); }
                            }
                            self.dirty = true;
                        }
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(80));
            }
            Err(mpsc::TryRecvError::Disconnected) => { self.ai_job = None; }
        }
    }

    // ── toasts ───────────────────────────────────────────────────────

    pub fn toast(&mut self, msg: impl Into<String>, kind: ToastKind, ctx: &Context) {
        self.toast = Some(Toast {
            message: msg.into(),
                          kind,
                          born: ctx.input(|i| i.time),
        });
    }

    // ── platform events ──────────────────────────────────────────────

    fn pump_platform(&mut self, ctx: &Context) {
        if let Some(rx) = &self.tray_rx {
            while let Ok(ev) = rx.try_recv() {
                match ev {
                    platform::TrayEvent::Show       => ctx.send_viewport_cmd(egui::ViewportCommand::Focus),
                    platform::TrayEvent::QuickCapture => { self.new_note(); }
                    platform::TrayEvent::Lock       => { self.lock_vault(); self.screen = Screen::Lock; }
                    platform::TrayEvent::Quit       => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    platform::TrayEvent::Prefs      => { self.screen = Screen::Setup; }
                }
            }
        }
        if let Some(rx) = &self.hotkey_rx {
            while let Ok(ev) = rx.try_recv() {
                if matches!(ev, platform::HotkeyEvent::QuickCapture) {
                    if matches!(self.screen, Screen::Main) {
                        self.new_note();
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════

impl eframe::App for KonachaApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        let dt = (now - self.last_frame) as f32;
        self.last_frame = now;

        self.tick_theme(ctx, dt);
        self.poll_ai(ctx);
        self.pump_platform(ctx);

        // ── background mesh (painted behind everything) ─────────────
        let rect = ctx.screen_rect();
        ui::mesh::paint(ctx, rect, &self.palette, self.prefs.ambient_mesh, now);

        // ── autosave ────────────────────────────────────────────────
        if self.dirty && now - self.last_autosave > 0.9 {
            self.commit_pending();
            self.last_autosave = now;
        }

        // ── route to current screen ─────────────────────────────────
        let next = match &self.screen {
            Screen::Splash { started } => {
                let elapsed = now - *started;
                let hold = if self.prefs.animations { 0.35 } else { 0.0 };
                if elapsed >= hold {
                    Some(if !self.prefs.onboarding_done { Screen::Setup } else { Screen::Lock })
                } else {
                    ctx.request_repaint();
                    None
                }
            }
            Screen::Setup => { ui::setup::show(self, ctx); None }
            Screen::Lock  => { ui::lock::show(self, ctx);  None }
            Screen::Main  => { ui::main_view::show(self, ctx); None }
        };
        if let Some(s) = next { self.screen = s; }

        // ── toast overlay ───────────────────────────────────────────
        ui::widgets::toast_overlay(self, ctx);

        // ── motion keep-alive ───────────────────────────────────────
        if self.prefs.animations && self.theme_animating {
            ctx.request_repaint();
        }

        let _ = dt;
        let _ = KonachaError::Locked; // silence unused-import lint in some builds
        let _ = Profile::Balanced;
        let _ = Vec2::ZERO;
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.commit_pending();
        let _ = self.prefs.save();
    }
}

// ── helpers ──────────────────────────────────────────────────────────────

fn smooth_color(a: egui::Color32, b: egui::Color32, dt: f32, speed: f32) -> egui::Color32 {
    let s = |x: u8, y: u8| {
        let v = smooth_to(x as f32, y as f32, dt, speed);
        v.round().clamp(0.0, 255.0) as u8
    };
    egui::Color32::from_rgba_unmultiplied(s(a.r(), b.r()), s(a.g(), b.g()), s(a.b(), b.b()), s(a.a(), b.a()))
}
