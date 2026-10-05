use serde::{Deserialize, Serialize};
use crate::config::paths;
use crate::error::Result;
use crate::storage::atomic;
use crate::theme::palette::ThemeId;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Performance,
    Balanced,
    Beauty,
}

impl Default for Profile {
    fn default() -> Self { Profile::Balanced }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Preferences {
    pub theme: ThemeId,
    pub profile: Profile,
    pub animations: bool,
    pub animation_speed: f32,
    pub ambient_mesh: bool,
    pub compact: bool,
    pub close_to_tray: bool,
    pub onboarding_done: bool,
    pub vim_mode: bool,
    pub editor_font_size: f32,
    pub ai_model: String,
    pub window_size: [f32; 2],
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: ThemeId::Nebula,
            profile: Profile::Balanced,
            animations: true,
            animation_speed: 1.0,
            ambient_mesh: true,
            compact: false,
            close_to_tray: true,
            onboarding_done: false,
            vim_mode: false,
            editor_font_size: 14.0,
            ai_model: "llama3.2:3b".into(),
            window_size: [1280.0, 820.0],
        }
    }
}

impl Preferences {
    pub fn load() -> Self {
        let p = paths::prefs_file();
        if !p.exists() { return Self::default(); }
        std::fs::read(&p)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        atomic::write_atomic(&paths::prefs_file(), &bytes)?;
        Ok(())
    }

    /// Multiplier applied to every animation duration.
    pub fn motion_scale(&self) -> f32 {
        if !self.animations { return 0.0; }
        let profile = match self.profile {
            Profile::Performance => 0.55,
            Profile::Balanced    => 1.00,
            Profile::Beauty      => 1.15,
        };
        (self.animation_speed * profile).clamp(0.0, 2.0)
    }
}
