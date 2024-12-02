// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    // prevent "Error 71 dispatching to Wayland display." error
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    // set tauri runtime to tokio
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    adconverter_lib::run().await.expect("Run converter")
}
