use std::{
    cmp::min,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use log::*;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_http::reqwest;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::{AppState, ProcessError, Task};

pub async fn publish(
    app: AppHandle,
    task: &Task,
    is_running: Arc<AtomicBool>,
) -> Result<(), ProcessError> {
    let state = app.state::<AppState>().to_owned();
    let config = state.config.lock().await.clone();
    let publisher = config.publisher.clone().ok_or("Get publisher")?;
    let token_peertube = publisher
        .get("peertube")
        .and_then(|p| p.get("access_token"))
        .ok_or("Peertube token")?;
    let url_peertube = publisher
        .get("peertube")
        .and_then(|p| p.get("url"))
        .ok_or("Peertube URL")?;
    let id_peertube = publisher
        .get("peertube")
        .and_then(|p| p.get("channel_id"))
        .ok_or("Peertube URL")?;
    let mut source = PathBuf::from(&task.path);

    // TODO: add webvtt file

    let publish_name = task.publish.as_ref().unwrap().name.clone();
    let publish_description = task.publish.as_ref().unwrap().description.clone();
    let publish_tags = task.publish.as_ref().unwrap().tags.clone();

    for preset in &task.presets {
        if let Some(publish_preset) = config.publish_preset.clone() {
            if preset.name == publish_preset {
                if let Some(output_path) = &preset.output_path {
                    source = output_path.to_path_buf();
                }
            }
        }
    }

    let file = tokio::fs::File::open(&source).await?;
    let total_size = file.metadata().await.unwrap().len();
    let mut reader_stream = ReaderStream::new(file);
    let mut uploaded = 0;

    app.emit("upload-start", "Publish Peertube")
        .expect("Emit Upload");

    let async_stream = async_stream::stream! {
        while let Some(chunk) = reader_stream.next().await {
            if let Ok(chunk) = &chunk {
                let new = min(uploaded + (chunk.len() as u64), total_size);
                uploaded = new;

                if is_running.load(Ordering::SeqCst) {
                    warn!("Cancel upload!");
                    break;
                }

                println!("{new:?}");

                app.emit("upload-progress", &new).expect("Emit progress");

                if uploaded >= total_size {
                    app.emit("upload-finish", "Publish Peertube").expect("Emit progress");
                }
            }
            yield chunk;
        }
    };

    let _ = reqwest::Client::new()
        .post(format!("{url_peertube}/api/v1/videos/upload"))
        .timeout(std::time::Duration::from_secs(600))
        .header("Authorization", &format!("Bearer {token_peertube}"))
        .multipart(
            reqwest::multipart::Form::new()
                .part(
                    "videofile",
                    reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(async_stream)),
                )
                .text("channelId", id_peertube.to_string())
                .text("name", publish_name)
                .text("description", publish_description)
                .text("name", publish_tags),
        )
        .send()
        .await?;

    Ok(())
}
