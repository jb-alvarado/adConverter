use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use chrono::{Datelike, Local};
use log::*;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::{mpsc::Receiver, Mutex},
};

use super::{analyze::Lufs, filter::filter_chain, probe::MediaProbe, FFmpegProgress};
use crate::{
    publisher,
    utils::{
        logging::{log_command, CommandLogger},
        Sources,
    },
};
use crate::{transcript, vec_strings, AppState, ProcessError, Task};

#[cfg(target_os = "macos")]
use crate::MACOS_PATH;

fn to_vec(value: Value) -> Vec<String> {
    let mut params = Vec::new();

    if let Value::Object(map) = value {
        for (key, val) in map {
            params.push(key);

            match val {
                Value::String(s) => params.push(s),
                _ => params.push(val.to_string()),
            }
        }
    }

    params
}

async fn calc_duration(task: &Task) -> (f64, f64, f64) {
    let mut duration_intro = 0.0;
    let mut duration_outro = 0.0;

    if let Some(template) = &task.template {
        if let Some(intro) = &template.intro {
            if let Ok(probe) = MediaProbe::new(&intro).await {
                duration_intro = probe.format_duration();
            };
        }

        if let Some(outro) = &template.outro {
            if let Ok(probe) = MediaProbe::new(&outro).await {
                duration_outro = probe.format_duration();
            };
        }
    }

    let duration = if task.r#in > 0.0 || task.out > 0.0 {
        task.out - task.r#in
    } else {
        task.probe.format.duration.unwrap_or_default()
    };

    (duration_intro, duration, duration_outro)
}

async fn work(
    app: AppHandle,
    child: Arc<Mutex<Option<Child>>>,
    is_running: Arc<AtomicBool>,
    task: Task,
) -> Result<(), ProcessError> {
    let state = app.state::<AppState>().to_owned();
    let config = state.config.lock().await.clone();
    let mut task_clone = task.clone();
    // let mut presets = mem::take(&mut task_clone.presets);
    let sources = Sources::new(&task.path).await;
    let path = Path::new(&task.path);
    let (i_dur, m_dur, o_dur) = calc_duration(&task).await;
    let duration = i_dur + m_dur + o_dur;
    let year = Local::now().year();
    let cmd_logger = CommandLogger::new();
    let mut audio_pos = -1;
    let mut has_audio = !task.probe.audio.is_empty();
    let mut has_video = false;

    let mut task_args = vec_strings![
        "-hide_banner",
        "-progress",
        "pipe:1",
        "-stats_period",
        "1",
        "-nostats",
        "-v",
        "level+info",
        "-y"
    ];

    let seek = if task.r#in > 0.0 {
        vec_strings!["-ss", task.r#in]
    } else {
        vec![]
    };

    let length = if task.out > 0.0 {
        vec_strings!["-t", task.out - task.r#in]
    } else {
        vec![]
    };

    let mut audio_path = String::new();

    if let Ok(src) = &sources {
        if let Some(video) = &src.video {
            audio_path = video.clone();
            audio_pos += 1;
            task_args.extend(seek.clone());
            task_args.extend(vec_strings!["-i", video]);
            task_args.extend(length.clone());
        }

        if let Some(audio) = &src.audio {
            audio_path = audio.clone();
            audio_pos += 1;
            has_audio = true;
            task_args.extend(seek.clone());
            task_args.extend(vec_strings!["-i", audio]);
            task_args.extend(length.clone());
        }
    }

    if let Some(lang) = &task.transcript {
        if lang.to_lowercase() != "none" {
            transcript::run(
                app.clone(),
                child.clone(),
                is_running.clone(),
                cmd_logger.clone(),
                &task,
                &audio_path,
            )
            .await?;
        }
    }

    let lufs = if task.lufs {
        let mut src_cmd = seek.clone();
        src_cmd.extend(vec_strings!["-i", audio_path]);
        src_cmd.extend(length);

        Lufs::new(
            app.clone(),
            m_dur,
            is_running.clone(),
            child.clone(),
            src_cmd,
            config.lufs,
            cmd_logger.clone(),
        )
        .await?
    } else {
        Lufs::default()
    };

    if !is_running.load(Ordering::SeqCst) {
        return Ok(());
    }

    task_args.extend(vec_strings![
        "-map_chapters",
        "-1",
        "-map_metadata",
        "-1",
        "-metadata",
        format!("year={year}")
    ]);

    if !config.copyright.is_empty() {
        task_args.extend(vec_strings![
            "-metadata",
            format!("copyright={}", config.copyright)
        ]);
    }

    for i in 0..task_clone.presets.len() {
        let preset = task_clone.presets[i].clone();
        let mut args = task_args.clone();
        let running = is_running.clone();
        let running_clone = is_running.clone();
        let app_clone1 = app.clone();
        let title = preset.title.clone();
        let finished = preset.finished.clone();
        let mut cmd_logger = cmd_logger.clone();

        let parent_path = path.parent().expect("Path should have a parent");
        let file_stem = path
            .file_stem()
            .ok_or("Path should have a valid file stem")?
            .to_string_lossy();
        let extension = preset
            .container_video
            .clone()
            .or(preset.container_audio.clone())
            .unwrap_or_default();

        let file_name = format!("{} # {}.{}", file_stem, preset.title, extension);

        let output = match task.target.as_ref() {
            Some(target) => {
                let mut op = PathBuf::from(target);

                if task.target_subfolder {
                    let sub = parent_path
                        .file_name()
                        .expect("Parent path should have a file name");
                    op = op.join(sub);

                    fs::create_dir_all(&op).await?;
                }

                op.join(&file_name)
            }
            None => parent_path.join(&file_name),
        };

        let temp_out = env::temp_dir().join(&file_name);
        task_clone.presets[i].output_path = Some(output.clone());

        if sources.as_ref().map(|s| s.video.clone()).is_ok() {
            if let Value::Object(map) = &preset.video {
                if !map.is_empty() {
                    has_video = true;
                }
            }
        }

        let mut filter = filter_chain(&task, &preset, &lufs, has_audio, has_video, audio_pos).await;
        args.extend(filter.cmd());

        if let Some(video_ext) = &preset.container_video {
            if has_video {
                args.extend(filter.map_video());
                args.extend(to_vec(preset.video.clone()));
            }

            if video_ext.eq_ignore_ascii_case("mp4") {
                args.extend(vec_strings!["-movflags", "+faststart"]);
            }
        }

        if let Some(audio_ext) = &preset.container_audio {
            if has_video {
                args.push(temp_out.to_string_lossy().to_string());
            }

            if has_audio {
                args.extend(filter.map_audio());
                args.extend(to_vec(preset.audio.clone()));
            }

            args.push(
                temp_out
                    .with_extension(audio_ext)
                    .to_string_lossy()
                    .to_string(),
            );
        } else {
            if has_audio {
                args.extend(filter.map_audio());
                args.extend(to_vec(preset.audio.clone()));
            }

            if has_video {
                args.push(temp_out.to_string_lossy().to_string());
            }
        }

        log_command(
            &format!("Preset: {}", preset.title),
            Some("ffmpeg".to_string()),
            args.clone(),
        );

        app_clone1
            .emit("preset-start", &preset)
            .expect("Emit Preset");

        let mut cmd = Command::new("ffmpeg");

        cmd.args(args).stderr(Stdio::piped()).stdout(Stdio::piped());

        #[cfg(target_os = "macos")]
        cmd.env("PATH", MACOS_PATH);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let mut proc = cmd.spawn()?;

        let stderr = proc.stderr.take().expect("Failed to capture stderr");
        let stdout = proc.stdout.take().expect("Failed to capture stdout");

        *child.lock().await = Some(proc);

        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();

            while let Some(line) = reader.next_line().await.expect("Read line") {
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                cmd_logger.log(Some("[ffmpeg]"), &line)
            }
        });

        let mut stat_map = HashMap::new();
        stat_map.insert("title".to_string(), title.clone());

        let stdout_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();

            while let Some(line) = reader.next_line().await.expect("Read line") {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }

                let mut process = String::new();

                if let Some((key, value)) = line.split_once('=') {
                    process = value.trim().to_string();

                    stat_map.insert(key.trim().to_string(), process.clone());
                }

                if line.starts_with("progress") {
                    let progress = FFmpegProgress::new(&stat_map, duration);

                    stat_map.clear();
                    stat_map.insert("title".to_string(), title.clone());

                    if &process == "end" {
                        finished.store(true, Ordering::SeqCst);

                        app_clone1
                            .emit("preset-finish", &preset)
                            .expect("Emit progress");
                    } else {
                        app_clone1
                            .emit("preset-progress", &progress)
                            .expect("Emit progress");
                    }
                }
            }
        });

        stderr_task.await?;
        stdout_task.await?;

        if let Some(proc) = child.lock().await.as_mut() {
            proc.wait().await?;
        }

        fs::copy(&temp_out, output).await?;
        fs::remove_file(temp_out).await?;
    }

    *child.lock().await = None;

    if task.publish.is_some() && is_running.load(Ordering::SeqCst) {
        publisher::peertube::publish(app, &task_clone, is_running).await?;
    }

    Ok(())
}

pub async fn run(
    app: AppHandle,
    state: State<'_, AppState>,
    mut rx: Receiver<Task>,
) -> Result<(), ProcessError> {
    while let Some(task) = rx.recv().await {
        task.active.store(true, Ordering::SeqCst);

        if !task.presets.is_empty()
            || (task.transcript.is_some() && task.transcript != Some("none".to_string()))
        {
            app.emit("task-active", &task)?;

            work(
                app.clone(),
                state.encoder.clone(),
                state.run.clone(),
                task.clone(),
            )
            .await?;
            task.active.store(false, Ordering::SeqCst);
            task.finished.store(true, Ordering::SeqCst);

            app.emit("task-finish", &task)?;
        } else {
            warn!("Task {:?} doesn't contain any job to process!", task.path);
        }
    }

    warn!("Task worker done...");

    Ok(())
}
