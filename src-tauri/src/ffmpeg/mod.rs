use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod analyze;
pub mod filter;
pub mod probe;
pub mod transcript;
pub mod worker;

#[derive(Clone, Debug, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
pub struct FFmpegProgress {
    pub title: String,
    pub fps: f32,
    pub bitrate: String,
    pub total_size: u64,
    pub elapsed_sec: f64,
    pub elapsed_pct: i64,
    pub speed: f32,
    pub progress: String,
}

impl FFmpegProgress {
    fn new(map: &HashMap<String, String>, duration: f64) -> Self {
        let seconds = map
            .get("out_time_ms")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or_default()
            / 1000000.0;

        Self {
            title: map.get("title").unwrap_or(&"".to_string()).to_string(),
            fps: map
                .get("fps")
                .and_then(|v| v.parse().ok())
                .unwrap_or_default(),
            bitrate: map.get("bitrate").unwrap_or(&"".to_string()).to_string(),
            total_size: map
                .get("total_size")
                .and_then(|v| v.parse().ok())
                .unwrap_or_default(),
            elapsed_sec: seconds,
            elapsed_pct: (seconds * 100.0 / duration).round() as i64,
            speed: map
                .get("speed")
                .and_then(|v| v.trim_end_matches('x').parse().ok())
                .unwrap_or_default(),
            progress: map.get("progress").unwrap_or(&"".to_string()).to_string(),
        }
    }
}

pub fn prepare_path(path: String) -> String {
    // on windows path for move/amove filter has to format

    if cfg!(windows) {
        return path.replace("\\", "/").replace(":", "\\\\:");
        // for UNC: .replace('//', '\\\\\\\\\\\\\\\\'
    }

    path
}
