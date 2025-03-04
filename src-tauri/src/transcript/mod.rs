use std::{
    env,
    path::Path,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tauri::{AppHandle, Emitter, Manager};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::Mutex,
};

use crate::{
    utils::logging::{log_command, CommandLogger},
    AppState, ProcessError, Task,
};

#[cfg(target_os = "macos")]
use crate::MACOS_PATH;

pub async fn run(
    app: AppHandle,
    child: Arc<Mutex<Option<Child>>>,
    is_running: Arc<AtomicBool>,
    mut cmd_logger: CommandLogger,
    source: &Path,
    task: &Task,
) -> Result<(), ProcessError> {
    let app_clone = app.clone();
    let state = app.state::<AppState>().to_owned();
    let running_clone = is_running.clone();
    let mut transcript_cmd = state.config.lock().await.transcript_cmd.clone();
    let lang = task.transcript.as_ref().map_or("auto", |v| v);
    let file_name = source.file_name().unwrap();
    let temp_out = env::temp_dir().join(&file_name).with_extension("vtt");
    let output_path = match &task.target {
        Some(p) => Path::new(p).join(file_name).with_extension("vtt"),
        None => source.with_extension("vtt"),
    };

    #[cfg(target_os = "windows")]
    let mut source = source;

    #[cfg(target_os = "windows")]
    {
        transcript_cmd = transcript_cmd.replace("\\", "\\\\");
        source = source.replace("\\", "\\\\");
    }

    if transcript_cmd.contains("%mount%") {
        if let Some(parent) = Path::new(&task.path).parent() {
            transcript_cmd = transcript_cmd.replace("%mount%", &format!("{parent:?}"))
        };
    }

    transcript_cmd = transcript_cmd.replace("%lang%", lang);

    transcript_cmd = transcript_cmd.replace("%file%", &format!("{source:?}"));

    if transcript_cmd.contains("%output%") {
        transcript_cmd = transcript_cmd.replace("%output%", &format!("{:?}", env::temp_dir()));
    }

    let mut args = shlex::split(&transcript_cmd).ok_or("No transcript command to split")?;

    log_command("Transcript", None, args.clone());

    let program = args.remove(0);

    let mut cmd = Command::new(program);
    cmd.args(args).stderr(Stdio::piped()).stdout(Stdio::piped());

    #[cfg(target_os = "macos")]
    cmd.env("PATH", MACOS_PATH);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);

    app_clone
        .emit("transcript-start", 0)
        .expect("Emit progress");

    let mut proc = cmd.spawn()?;

    let stderr = proc.stderr.take().ok_or("Failed to capture stderr")?;
    let stdout = proc.stdout.take().ok_or("Failed to capture stdout")?;

    *child.lock().await = Some(proc);

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut buffer = Vec::new();

        while let Ok(bytes_read) = reader.read_until(b'\n', &mut buffer).await {
            if bytes_read == 0 {
                break; // EOF
            }

            if !is_running.load(Ordering::SeqCst) {
                break;
            }

            cmd_logger.log(Some("[transcript]"), &String::from_utf8_lossy(&buffer));
            buffer.clear();
        }
    });

    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut buffer = Vec::new();

        while let Ok(bytes_read) = reader.read_until(b'\n', &mut buffer).await {
            if bytes_read == 0 {
                break; // EOF
            }

            if !running_clone.load(Ordering::SeqCst) {
                break;
            }

            app_clone
                .emit("transcript-progress", &String::from_utf8_lossy(&buffer))
                .expect("Emit progress");
            buffer.clear();
        }
    });

    stderr_task.await?;
    stdout_task.await?;

    if let Some(proc) = child.lock().await.as_mut() {
        proc.wait().await?;
    }

    *child.lock().await = None;

    if temp_out.is_file() {
        fs::copy(&temp_out, output_path).await?;
        fs::remove_file(temp_out).await?;
    }

    app.emit("transcript-finish", lang).expect("Emit progress");

    Ok(())
}
