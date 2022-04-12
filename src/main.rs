#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use vs_toolbox::Toolbox;
    let toolbox = Toolbox {
        cached: Default::default(),
        filter_options: Default::default(),
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(toolbox), native_options);
}
