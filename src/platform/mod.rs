pub mod hotkeys;
pub mod tray;

#[derive(Debug)]
pub enum TrayEvent {
    Show,
    QuickCapture,
    Lock,
    Prefs,
    Quit,
}

#[derive(Debug)]
pub enum HotkeyEvent {
    QuickCapture,
}
