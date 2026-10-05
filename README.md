# Konacha

Offline-first, encrypted, native desktop note-taking. Pure Rust + egui.
No webview. No HTML. No JS. One binary.

## Build

    # Linux system deps (for tray + hotkeys)
    sudo apt install libgtk-3-dev libxdo-dev libx11-dev

    cargo build --release

## Run

    ./target/release/konacha

Data lives in:
- Linux:   ~/.config/konacha
- macOS:   ~/Library/Application Support/konacha
- Windows: %APPDATA%\konacha
