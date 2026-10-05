use std::path::PathBuf;

/// Platform vault root:
///   Linux   → ~/.config/konacha  (or $XDG_CONFIG_HOME/konacha)
///   macOS   → ~/Library/Application Support/konacha
///   Windows → %APPDATA%\konacha
pub fn vault_root() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("konacha"))
        .unwrap_or_else(|| PathBuf::from(".konacha"))
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|p| p.join("Library").join("Application Support").join("konacha"))
        .unwrap_or_else(|| PathBuf::from(".konacha"))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .map(|p| p.join("konacha"))
        .or_else(|| {
            std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|p| p.join(".config").join("konacha"))
        })
        .unwrap_or_else(|| PathBuf::from(".konacha"))
    }
}

pub fn prefs_file() -> PathBuf { vault_root().join("prefs.json") }
