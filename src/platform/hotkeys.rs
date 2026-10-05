use std::sync::mpsc::Sender;
use crate::platform::HotkeyEvent;

/// Registers the global Super/Cmd+Shift+N hotkey and forwards presses.
pub fn spawn(tx: Sender<HotkeyEvent>) -> Result<(), String> {
    use global_hotkey::{
        hotkey::{Code, HotKey, Modifiers},
        GlobalHotKeyEvent, GlobalHotKeyManager,
    };

    // The manager must live for the lifetime of the program.
    let manager = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;

    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyN);
    manager.register(hotkey).map_err(|e| e.to_string())?;

    // Leak the manager so it lives for the whole process (this is deliberate —
    // it owns OS-level registrations that must not be dropped).
    Box::leak(Box::new(manager));

    std::thread::Builder::new()
    .name("konacha-hotkeys".into())
    .spawn(move || {
        loop {
            if let Ok(ev) = GlobalHotKeyEvent::receiver().recv() {
                if ev.state == global_hotkey::HotKeyState::Pressed {
                    if tx.send(HotkeyEvent::QuickCapture).is_err() { return; }
                }
            }
        }
    })
    .map(|_| ())
    .map_err(|e| e.to_string())
}
