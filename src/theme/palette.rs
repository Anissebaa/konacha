use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeId {
    Nebula, Obsidian, Crimson, Emerald, Solar,
    Arctic, Sakura, Cyberpunk, Parchment,
}

impl ThemeId {
    pub const ALL: [ThemeId; 9] = [
        ThemeId::Nebula, ThemeId::Obsidian, ThemeId::Crimson,
        ThemeId::Emerald, ThemeId::Solar, ThemeId::Arctic,
        ThemeId::Sakura, ThemeId::Cyberpunk, ThemeId::Parchment,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ThemeId::Nebula    => "Nebula",
            ThemeId::Obsidian  => "Obsidian",
            ThemeId::Crimson   => "Crimson",
            ThemeId::Emerald   => "Emerald",
            ThemeId::Solar     => "Solar",
            ThemeId::Arctic    => "Arctic",
            ThemeId::Sakura    => "Sakura",
            ThemeId::Cyberpunk => "Cyberpunk",
            ThemeId::Parchment => "Parchment",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            ThemeId::Nebula    => "Signature purple/cyan glass",
            ThemeId::Obsidian  => "Monochrome, editorial",
            ThemeId::Crimson   => "Ember forge, warm reds",
            ThemeId::Emerald   => "Deep forest glass",
            ThemeId::Solar     => "Amber dusk, warm focus",
            ThemeId::Arctic    => "Frosted ice blues",
            ThemeId::Sakura    => "Soft pink dusk",
            ThemeId::Cyberpunk => "High-contrast neon",
            ThemeId::Parchment => "Warm light writer theme",
        }
    }

    pub fn swatch(self) -> (Color32, Color32, Color32) {
        let p = self.palette();
        (p.bg_0, p.accent_1, p.accent_2)
    }

    pub fn palette(self) -> Palette {
        match self {
            ThemeId::Nebula    => NEBULA,
            ThemeId::Obsidian  => OBSIDIAN,
            ThemeId::Crimson   => CRIMSON,
            ThemeId::Emerald   => EMERALD,
            ThemeId::Solar     => SOLAR,
            ThemeId::Arctic    => ARCTIC,
            ThemeId::Sakura    => SAKURA,
            ThemeId::Cyberpunk => CYBERPUNK,
            ThemeId::Parchment => PARCHMENT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme { Dark, Light }

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub scheme:       Scheme,
    pub bg_0:         Color32,
    pub bg_1:         Color32,
    pub bg_inset:     Color32,
    pub panel:        Color32,
    pub panel_strong: Color32,
    pub panel_hover:  Color32,
    pub border:       Color32,
    pub border_hi:    Color32,
    pub accent_1:     Color32,
    pub accent_2:     Color32,
    pub accent_3:     Color32,
    pub accent_soft:  Color32,
    pub text:         Color32,
    pub text_dim:     Color32,
    pub text_mute:    Color32,
    pub mesh_1:       Color32,
    pub mesh_2:       Color32,
    pub mesh_3:       Color32,
}

impl Palette {
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            scheme:       if t < 0.5 { a.scheme } else { b.scheme },
            bg_0:         lerp_c(a.bg_0,         b.bg_0,         t),
            bg_1:         lerp_c(a.bg_1,         b.bg_1,         t),
            bg_inset:     lerp_c(a.bg_inset,     b.bg_inset,     t),
            panel:        lerp_c(a.panel,        b.panel,        t),
            panel_strong: lerp_c(a.panel_strong, b.panel_strong, t),
            panel_hover:  lerp_c(a.panel_hover,  b.panel_hover,  t),
            border:       lerp_c(a.border,       b.border,       t),
            border_hi:    lerp_c(a.border_hi,    b.border_hi,    t),
            accent_1:     lerp_c(a.accent_1,     b.accent_1,     t),
            accent_2:     lerp_c(a.accent_2,     b.accent_2,     t),
            accent_3:     lerp_c(a.accent_3,     b.accent_3,     t),
            accent_soft:  lerp_c(a.accent_soft,  b.accent_soft,  t),
            text:         lerp_c(a.text,         b.text,         t),
            text_dim:     lerp_c(a.text_dim,     b.text_dim,     t),
            text_mute:    lerp_c(a.text_mute,    b.text_mute,    t),
            mesh_1:       lerp_c(a.mesh_1,       b.mesh_1,       t),
            mesh_2:       lerp_c(a.mesh_2,       b.mesh_2,       t),
            mesh_3:       lerp_c(a.mesh_3,       b.mesh_3,       t),
        }
    }

    /// Convert palette → egui Visuals for widgets that use the built-in style.
    pub fn to_visuals(&self) -> egui::Visuals {
        let mut v = match self.scheme {
            Scheme::Dark  => egui::Visuals::dark(),
            Scheme::Light => egui::Visuals::light(),
        };

        v.panel_fill        = self.panel;
        v.window_fill       = self.panel_strong;
        v.extreme_bg_color  = self.bg_inset;
        v.faint_bg_color    = self.panel_hover;
        v.override_text_color = Some(self.text);
        v.hyperlink_color   = self.accent_2;

        let s = |w: f32, c: Color32| egui::Stroke::new(w, c);

        v.widgets.noninteractive.bg_fill      = self.panel;
        v.widgets.noninteractive.weak_bg_fill = self.panel;
        v.widgets.noninteractive.bg_stroke    = s(1.0, self.border);
        v.widgets.noninteractive.fg_stroke    = s(1.0, self.text_dim);

        v.widgets.inactive.bg_fill       = self.panel_hover;
        v.widgets.inactive.weak_bg_fill  = self.panel;
        v.widgets.inactive.bg_stroke     = s(1.0, self.border);
        v.widgets.inactive.fg_stroke     = s(1.0, self.text);

        v.widgets.hovered.bg_fill        = self.accent_soft;
        v.widgets.hovered.weak_bg_fill   = self.panel_hover;
        v.widgets.hovered.bg_stroke      = s(1.0, self.border_hi);
        v.widgets.hovered.fg_stroke      = s(1.0, self.text);

        v.widgets.active.bg_fill         = self.accent_soft;
        v.widgets.active.weak_bg_fill    = self.panel_hover;
        v.widgets.active.bg_stroke       = s(1.5, self.accent_1);
        v.widgets.active.fg_stroke       = s(1.0, self.text);

        v.selection.bg_fill = self.accent_soft;
        v.selection.stroke  = s(1.0, self.accent_1);

        v
    }
}

fn lerp_c(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        lerp_u8(a.r(), b.r(), t),
                                    lerp_u8(a.g(), b.g(), t),
                                    lerp_u8(a.b(), b.b(), t),
                                    lerp_u8(a.a(), b.a(), t),
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

#[inline]
fn rgb(r: u8, g: u8, b: u8) -> Color32 { Color32::from_rgb(r, g, b) }
#[inline]
fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 { Color32::from_rgba_unmultiplied(r, g, b, a) }

// ── The 9 palettes ────────────────────────────────────────────────────

pub const NEBULA: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(10, 12, 16), bg_1: rgb(14, 17, 24),
    bg_inset:     rgba(0, 0, 0, 82),
    panel:        rgba(20, 24, 34, 140),
    panel_strong: rgba(26, 30, 42, 200),
    panel_hover:  rgba(168, 85, 247, 26),
    border:       rgba(255, 255, 255, 18),
    border_hi:    rgba(168, 85, 247, 90),
    accent_1: rgb(168, 85, 247),
    accent_2: rgb( 34, 211, 238),
    accent_3: rgb(244, 114, 182),
    accent_soft:  rgba(168, 85, 247, 36),
    text:      rgb(230, 233, 239),
    text_dim:  rgb(139, 147, 167),
    text_mute: rgb( 90,  98, 116),
    mesh_1: rgba(168, 85, 247, 56),
    mesh_2: rgba( 34, 211, 238, 40),
    mesh_3: rgba(244, 114, 182, 26),
};

pub const OBSIDIAN: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(5, 6, 8), bg_1: rgb(10, 11, 14),
    bg_inset:     rgba(0, 0, 0, 100),
    panel:        rgba(18, 20, 24, 158),
    panel_strong: rgba(22, 24, 28, 218),
    panel_hover:  rgba(255, 255, 255, 16),
    border:       rgba(255, 255, 255, 16),
    border_hi:    rgba(255, 255, 255, 60),
    accent_1: rgb(229, 231, 235),
    accent_2: rgb(156, 163, 175),
    accent_3: rgb(107, 114, 128),
    accent_soft:  rgba(229, 231, 235, 26),
    text:      rgb(243, 244, 246),
    text_dim:  rgb(156, 163, 175),
    text_mute: rgb( 75,  85,  99),
    mesh_1: rgba(255, 255, 255, 16),
    mesh_2: rgba(255, 255, 255, 10),
    mesh_3: rgba(255, 255, 255,  6),
};

pub const CRIMSON: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(16, 6, 7), bg_1: rgb(24, 9, 11),
    bg_inset:     rgba(0, 0, 0, 98),
    panel:        rgba(40, 14, 18, 140),
    panel_strong: rgba(48, 18, 22, 204),
    panel_hover:  rgba(239, 68, 68, 30),
    border:       rgba(255, 200, 200, 22),
    border_hi:    rgba(239, 68, 68, 115),
    accent_1: rgb(239, 68, 68),
    accent_2: rgb(249, 115, 22),
    accent_3: rgb(251, 191, 36),
    accent_soft:  rgba(239, 68, 68, 40),
    text:      rgb(253, 232, 232),
    text_dim:  rgb(201, 155, 155),
    text_mute: rgb(122,  82,  82),
    mesh_1: rgba(239, 68, 68, 56),
    mesh_2: rgba(249, 115, 22, 40),
    mesh_3: rgba(251, 191, 36, 26),
};

pub const EMERALD: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(4, 16, 11), bg_1: rgb(7, 24, 16),
    bg_inset:     rgba(0, 0, 0, 98),
    panel:        rgba(12, 34, 24, 140),
    panel_strong: rgba(16, 44, 32, 204),
    panel_hover:  rgba(16, 185, 129, 30),
    border:       rgba(200, 255, 230, 20),
    border_hi:    rgba(16, 185, 129, 108),
    accent_1: rgb( 16, 185, 129),
    accent_2: rgb( 34, 211, 238),
    accent_3: rgb(163, 230,  53),
    accent_soft:  rgba(16, 185, 129, 36),
    text:      rgb(217, 245, 232),
    text_dim:  rgb(127, 182, 155),
    text_mute: rgb( 78, 107,  92),
    mesh_1: rgba(16, 185, 129, 56),
    mesh_2: rgba( 34, 211, 238, 36),
    mesh_3: rgba(163, 230,  53, 20),
};

pub const SOLAR: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(18, 11, 4), bg_1: rgb(26, 16, 7),
    bg_inset:     rgba(0, 0, 0, 98),
    panel:        rgba(44, 28, 12, 140),
    panel_strong: rgba(52, 34, 14, 204),
    panel_hover:  rgba(245, 158, 11, 30),
    border:       rgba(255, 230, 180, 22),
    border_hi:    rgba(245, 158, 11, 112),
    accent_1: rgb(245, 158, 11),
    accent_2: rgb(239, 68, 68),
    accent_3: rgb(252, 211, 77),
    accent_soft:  rgba(245, 158, 11, 36),
    text:      rgb(251, 233, 201),
    text_dim:  rgb(194, 160, 108),
    text_mute: rgb(122,  96,  57),
    mesh_1: rgba(245, 158, 11, 56),
    mesh_2: rgba(239, 68, 68, 40),
    mesh_3: rgba(252, 211, 77, 26),
};

pub const ARCTIC: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(6, 16, 26), bg_1: rgb(10, 24, 38),
    bg_inset:     rgba(0, 0, 0, 98),
    panel:        rgba(18, 34, 52, 140),
    panel_strong: rgba(24, 44, 66, 204),
    panel_hover:  rgba(56, 189, 248, 30),
    border:       rgba(200, 235, 255, 26),
    border_hi:    rgba(56, 189, 248, 115),
    accent_1: rgb( 56, 189, 248),
    accent_2: rgb(165, 243, 252),
    accent_3: rgb(129, 140, 248),
    accent_soft:  rgba(56, 189, 248, 36),
    text:      rgb(220, 239, 251),
    text_dim:  rgb(134, 168, 196),
    text_mute: rgb( 77, 107, 133),
    mesh_1: rgba(56, 189, 248, 56),
    mesh_2: rgba(165, 243, 252, 36),
    mesh_3: rgba(129, 140, 248, 26),
};

pub const SAKURA: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(18, 6, 14), bg_1: rgb(26, 8, 20),
    bg_inset:     rgba(0, 0, 0, 98),
    panel:        rgba(44, 16, 32, 140),
    panel_strong: rgba(52, 20, 38, 204),
    panel_hover:  rgba(236, 72, 153, 30),
    border:       rgba(255, 210, 235, 26),
    border_hi:    rgba(236, 72, 153, 108),
    accent_1: rgb(236, 72, 153),
    accent_2: rgb(168, 85, 247),
    accent_3: rgb(249, 168, 212),
    accent_soft:  rgba(236, 72, 153, 38),
    text:      rgb(252, 228, 240),
    text_dim:  rgb(196, 146, 171),
    text_mute: rgb(122,  85, 104),
    mesh_1: rgba(236, 72, 153, 56),
    mesh_2: rgba(168, 85, 247, 40),
    mesh_3: rgba(249, 168, 212, 26),
};

pub const CYBERPUNK: Palette = Palette {
    scheme: Scheme::Dark,
    bg_0: rgb(6, 6, 14), bg_1: rgb(12, 12, 26),
    bg_inset:     rgba(0, 0, 0, 108),
    panel:        rgba(20, 20, 46, 148),
    panel_strong: rgba(28, 28, 60, 210),
    panel_hover:  rgba(250, 204, 21, 36),
    border:       rgba(250, 204, 21, 36),
    border_hi:    rgba(250, 204, 21, 140),
    accent_1: rgb(250, 204, 21),
    accent_2: rgb(236, 72, 153),
    accent_3: rgb( 34, 211, 238),
    accent_soft:  rgba(250, 204, 21, 38),
    text:      rgb(245, 245, 255),
    text_dim:  rgb(163, 163, 194),
    text_mute: rgb( 90,  90, 128),
    mesh_1: rgba(250, 204, 21, 52),
    mesh_2: rgba(236, 72, 153, 56),
    mesh_3: rgba( 34, 211, 238, 40),
};

pub const PARCHMENT: Palette = Palette {
    scheme: Scheme::Light,
    bg_0: rgb(243, 234, 216), bg_1: rgb(237, 226, 203),
    bg_inset:     rgba(255, 255, 255, 140),
    panel:        rgba(255, 250, 240, 166),
    panel_strong: rgba(255, 252, 245, 218),
    panel_hover:  rgba(120, 60, 20, 20),
    border:       rgba(80, 50, 20, 36),
    border_hi:    rgba(180, 100, 30, 140),
    accent_1: rgb(180, 83, 9),
    accent_2: rgb(146, 64, 14),
    accent_3: rgb(217, 119, 6),
    accent_soft:  rgba(180, 83, 9, 30),
    text:      rgb( 43,  28,  13),
    text_dim:  rgb(107,  84,  58),
    text_mute: rgb(154, 130, 102),
    mesh_1: rgba(217, 119, 6, 46),
    mesh_2: rgba(146, 64, 14, 30),
    mesh_3: rgba(120, 60, 20, 16),
};
