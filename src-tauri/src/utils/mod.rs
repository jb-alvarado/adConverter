use std::{
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use log::*;
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;
use tokio::fs::{self};
use tokio_stream::StreamExt;

pub mod errors;
pub mod logging;
pub mod presets;
pub mod template;

use crate::ProcessError;

pub const AUDIO_EXTENSIONS: [&str; 9] = [
    "wav", "aif", "aiff", "aac", "mp3", "flac", "mp2", "m4a", "opus",
];
pub const IMAGE_EXTENSIONS: [&str; 6] = ["exr", "png", "tga", "tif", "tiff", "gif"];
pub const VIDEO_EXTENSIONS: [&str; 15] = [
    "avi", "mov", "webm", "mp4", "mpv", "m4v", "h264", "mkv", "vob", "wmv", "yuv", "m2v", "mpg",
    "mpeg", "mxf",
];

#[derive(Clone, Default, Debug)]
pub struct Sources {
    pub audio: Option<String>,
    pub video: Option<String>,
    pub template: Option<PathBuf>,
}

impl Sources {
    pub async fn new(src: &str) -> Result<Self, ProcessError> {
        let path = PathBuf::from(src);
        let folder = path
            .parent()
            .ok_or(ProcessError::IO(format!("No parent folder from: {src}")))?;
        let mut entries = fs::read_dir(folder).await?;
        let mut source = Sources::default();

        if VIDEO_EXTENSIONS.contains(
            &path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .as_str(),
        ) {
            source.video = Some(src.to_string());
        };

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().file_stem() == path.file_stem() {
                let extension = entry
                    .path()
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();

                if AUDIO_EXTENSIONS.contains(&extension.as_str()) {
                    source.audio = Some(entry.path().to_string_lossy().to_string());
                } else if source.video.is_none() && VIDEO_EXTENSIONS.contains(&extension.as_str()) {
                    source.video = Some(entry.path().to_string_lossy().to_string());
                } else if extension.as_str() == "json" {
                    source.template = Some(entry.path().to_path_buf());
                }
            }
        }

        Ok(source)
    }
}

pub async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    debug!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    debug!("download finished");
                },
            )
            .await?;

        debug!("update installed");
        app.restart();
    }

    Ok(())
}

pub fn time_to_sec(time_str: &str) -> f64 {
    let mut t = time_str
        .split([':', '.'])
        .filter_map(|n| f64::from_str(n).ok());

    let hours = t.next().unwrap_or(0.0);
    let minutes = t.next().unwrap_or(0.0);
    let seconds = t.next().unwrap_or(0.0);
    let milliseconds = t.next().unwrap_or(0.0);

    hours * 3600.0 + minutes * 60.0 + seconds + milliseconds / 1000.0
}

pub async fn delete_files(path: &Path) -> Result<(), ProcessError> {
    let temp_file = env::temp_dir().join(path.file_name().unwrap());
    let folder = path
        .parent()
        .ok_or(ProcessError::IO(format!("No parent folder from: {path:?}")))?;
    let mut entries = fs::read_dir(folder).await?;

    if temp_file.is_file() {
        fs::remove_file(temp_file).await?;
    }

    while let Some(entry) = entries.next_entry().await? {
        if entry.path().file_stem() == path.file_stem() {
            fs::remove_file(entry.path()).await?;
        }
    }

    Ok(())
}

pub fn is_close<T: num_traits::Signed + std::cmp::PartialOrd>(a: T, b: T, to: T) -> bool {
    (a - b).abs() < to
}

pub async fn copy_assets(app: &AppHandle) -> Result<(), ProcessError> {
    let path_source = app.path().resolve("assets", BaseDirectory::Resource)?;
    let target_source = app.path().app_data_dir()?;

    let mut entries = async_walkdir::WalkDir::new(&path_source);

    loop {
        match entries.next().await {
            Some(Ok(entry)) => {
                let source_path = entry.path();
                let relative_path = source_path.strip_prefix(&path_source)?;
                let target_path = target_source.join(relative_path);

                if source_path.is_dir() && !target_path.is_dir() {
                    fs::create_dir_all(&target_path).await?;
                } else if source_path.is_file() && !target_path.is_file() {
                    fs::copy(&source_path, &target_path).await?;
                }
            }
            Some(Err(e)) => {
                error!("error: {}", e);
                break;
            }
            None => break,
        }
    }

    Ok(())
}
