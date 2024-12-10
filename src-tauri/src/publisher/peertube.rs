use tauri::{AppHandle, Manager};
// use tauri_plugin_http::reqwest;

use crate::{AppState, ProcessError, Task};

pub async fn publish(app: AppHandle, _task: &Task) -> Result<(), ProcessError> {
    let state = app.state::<AppState>().to_owned();
    let config = state.config.lock().await.clone();
    let _publisher = config.publisher.ok_or("Get publisher")?;

    Ok(())
}
