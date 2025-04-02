use arboard::Clipboard;
use clap::Parser;
use enigo::{Direction::*, Enigo, Key, Keyboard, Settings};
use std::process::Command;
use ui::DClipWindow;

mod cli;
mod config;
mod system;
mod ui;

fn main() {
    // Load or create user config file
    let config = config::Config::load().expect("Failed to load config file.");

    // Initialize enigo for keyboard controls (copy and paste)
    let mut enigo = Enigo::new(&Settings::default()).expect("Failed to initialize enigo.");

    let cli = cli::Cli::parse();
    println!("{}", sdl2::version::version());

    let active_window = Command::new("xdotool")
        .arg("getactivewindow")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Create and launch window
    let mut dclip_window = DClipWindow::new(cli.copy, config);
    let paste_text = dclip_window.launch();

    if let Some(snippet) = paste_text {
        let mut ctx: Clipboard = Clipboard::new().unwrap();
        ctx.set_text(snippet)
            .expect("Failed to set text to clipboard.");
        let _ = ctx.get_text();

        if let Some(win_id) = &active_window {
            let _ = Command::new("xdotool")
                .args(["windowactivate", win_id])
                .status();
        }

        let _ = enigo.key(Key::Control, Press);
        let _ = enigo.key(Key::Shift, Press);
        let _ = enigo.key(Key::Unicode('v'), Click);
        let _ = enigo.key(Key::Control, Release);
        let _ = enigo.key(Key::Shift, Release);
    }
}
