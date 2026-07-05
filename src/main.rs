// tidewm — minimal tiling window manager for Windows
// Consumes ~0% CPU at idle. Only wakes on window events.

#![windows_subsystem = "windows"] // no console window

mod config;
mod layout;
mod animator;
mod manager;
mod hotkeys;

use manager::WindowManager;

fn main() {
    let config = config::Config::load();
    let mut wm = WindowManager::new(config);
    wm.run(); // blocks on the Windows message loop
}
