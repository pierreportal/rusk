#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Minimal command invoked from the frontend, proving the Rust backend works on
/// the reference hardware.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! The Rust kiosk backend is running.")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running the Rusk example application");
}
