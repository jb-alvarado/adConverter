// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::process::exit;

use log::error;

use adconverter_lib::{cli, init_logging};

#[tokio::main]
async fn main() {
    if std::env::args().any(|a| a == "--cli") {
        let _logger = init_logging(None);

        if let Err(e) = cli::encoder::run().await {
            error!("{e}");

            exit(1);
        };
    } else {
        #[cfg(target_os = "linux")]
        {
            // prevent "Error 71 dispatching to Wayland display." error
            // TODO: Audit that the environment access only happens in single-threaded code.
            unsafe { std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1") };
            // prevent "Could not create GBM EGL display: EGL_SUCCESS."
            // TODO: Audit that the environment access only happens in single-threaded code.
            unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
        }

        // set tauri runtime to tokio
        tauri::async_runtime::set(tokio::runtime::Handle::current());

        adconverter_lib::run().await.expect("Run converter")
    }
}
