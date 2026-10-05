use std::sync::mpsc::Sender;
use crate::platform::TrayEvent;

/// Spawns the system tray on a dedicated thread.
///
/// NOTE: on macOS the tray icon *must* be created on the main thread; the
/// current build uses eframe on the main thread and creates the tray on a
/// worker thread, which works on Windows and on Linux with `libappindicator`,
/// but on macOS the icon may not appear. For a production macOS build, move
/// this call into the eframe creator callback (which runs on the main thread).
pub fn spawn(tx: Sender<TrayEvent>) -> Result<(), String> {
    std::thread::Builder::new()
    .name("konacha-tray".into())
    .spawn(move || {
        use tray_icon::{
            menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
            TrayIconBuilder,
        };

        let show    = MenuItem::new("Show Konacha", true, None);
        let capture = MenuItem::new("Quick Capture", true, None);
        let prefs   = MenuItem::new("Preferences…", true, None);
        let lock    = MenuItem::new("Lock Vault", true, None);
        let quit    = MenuItem::new("Quit", true, None);
        let sep     = PredefinedMenuItem::separator();

        let menu = match Menu::new() {
            Ok(m) => m,
           Err(_) => return,
        };
        let _ = menu.append_items(&[
            &show, &capture, &sep, &prefs, &lock, &quit,
        ]);

        // Build a 1×1 transparent icon if no asset is bundled.
        let icon = match load_icon() {
            Some(i) => i,
           None => return,
        };

        let _tray = match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Konacha")
        .with_icon(icon)
        .build()
        {
            Ok(t) => t,
           Err(e) => {
               log::warn!("tray init failed: {e}");
               return;
           }
        };

        // Keep the thread alive and route menu events.
        let show_id    = show.id().clone();
        let capture_id = capture.id().clone();
        let prefs_id   = prefs.id().clone();
        let lock_id    = lock.id().clone();
        let quit_id    = quit.id().clone();

        loop {
            while let Ok(ev) = MenuEvent::receiver().try_recv() {
                let mapped = if ev.id == show_id { Some(TrayEvent::Show) }
                else if ev.id == capture_id { Some(TrayEvent::QuickCapture) }
                else if ev.id == prefs_id   { Some(TrayEvent::Prefs) }
                else if ev.id == lock_id    { Some(TrayEvent::Lock) }
                else if ev.id == quit_id    { Some(TrayEvent::Quit) }
                else { None };
                if let Some(e) = mapped {
                    if tx.send(e).is_err() { return; }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(60));
        }
    })
    .map(|_| ())
    .map_err(|e| e.to_string())
}

fn load_icon() -> Option<tray_icon::Icon> {
    // 1×1 fully transparent RGBA — a valid placeholder until a real asset
    // is dropped into `assets/icons/konacha.png`.
    let rgba: Vec<u8> = vec![0, 0, 0, 0];
    tray_icon::Icon::from_rgba(rgba, 1, 1).ok()
}
