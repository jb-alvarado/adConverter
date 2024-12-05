use std::{
    path::Path,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tauri::{AppHandle, Emitter, Manager};
use tokio::{
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
    task: &Task,
    source: &str,
) -> Result<(), ProcessError> {
    let app_clone = app.clone();
    let state = app.state::<AppState>().to_owned();
    let running_clone = is_running.clone();
    let mut transcript_cmd = state.config.lock().await.transcript_cmd.clone();

    #[cfg(target_os = "windows")]
    {
        transcript_cmd = transcript_cmd.replace("\\", "\\\\");
    }

    if transcript_cmd.contains("%mount%") {
        if let Some(parent) = Path::new(&task.path).parent() {
            transcript_cmd =
                transcript_cmd.replace("%mount%", &format!("\"{}\"", parent.to_string_lossy()))
        };
    }

    transcript_cmd = transcript_cmd.replace(
        "%lang%",
        task.transcript.as_ref().unwrap_or(&"auto".to_string()),
    );

    transcript_cmd = transcript_cmd.replace("%file%", &format!("\"{source}\""));

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
        let mut reader = BufReader::new(stderr).lines();

        while let Some(line) = reader.next_line().await.expect("Read line") {
            if !is_running.load(Ordering::SeqCst) {
                break;
            }

            cmd_logger.log(None, &line)
        }
    });

    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await.expect("Read line") {
            if !running_clone.load(Ordering::SeqCst) {
                break;
            }

            app_clone
                .emit("transcript-progress", &line)
                .expect("Emit progress");
        }
    });

    stderr_task.await?;
    stdout_task.await?;

    if let Some(proc) = child.lock().await.as_mut() {
        proc.wait().await?;
    }

    *child.lock().await = None;

    Ok(())
}
