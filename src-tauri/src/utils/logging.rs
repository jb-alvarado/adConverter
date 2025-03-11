use std::io::{self, ErrorKind};

use flexi_logger::{writers::LogWriter, DeferredNow, Level, LogSpecification, Logger};
use log::*;
use regex::Regex;
use tauri::{AppHandle, Emitter};

use crate::utils::VIDEO_EXTENSIONS;

#[macro_export]
macro_rules! plain {
    ($($arg:tt)*) => {{
        log::info!(target: "{plain}", "{}", format_args!($($arg)*));
    }};
}

#[derive(Clone)]
pub struct CommandLogger {
    last_level: Level,
}

impl CommandLogger {
    pub fn new() -> Self {
        Self {
            last_level: Level::Info,
        }
    }

    pub fn clean_log(&self, input: &str) -> String {
        let re = Regex::new(r"(?i)\s*\[(info|warning|error|fatal)\] ?").unwrap();

        re.replace_all(input, "").to_string()
    }

    pub fn log(&mut self, prefix: Option<&str>, line: &str) {
        let prefix = match prefix {
            Some(p) => format!("<span class=\"text-base-content/60\">{p} </span>"),
            None => String::new(),
        };

        if line.to_lowercase().contains("[info]") {
            info!("{prefix}{}", self.clean_log(line));

            self.last_level = Level::Info;
        } else if line.to_lowercase().contains("[warning]") {
            warn!("{prefix}{}", self.clean_log(line));

            self.last_level = Level::Warn;
        } else if line.to_lowercase().contains("[error]") || line.to_lowercase().contains("[fatal]")
        {
            error!("{prefix}{}", self.clean_log(line));

            self.last_level = Level::Error;
        } else {
            match self.last_level {
                Level::Info => info!("{prefix}{line}"),
                Level::Warn => warn!("{prefix}{line}"),
                Level::Error => error!("{prefix}{line}"),
                _ => debug!("{prefix}{line}"),
            }
        }
    }
}

pub struct LogPlain {
    pub app: AppHandle,
}

impl LogPlain {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl LogWriter for LogPlain {
    fn write(&self, _now: &mut DeferredNow, record: &Record<'_>) -> std::io::Result<()> {
        if let Err(e) = self.app.emit("logging", record.args().to_string()) {
            eprint!("{e:?}");
        };

        Ok(())
    }
    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct LogEmitter {
    pub app: AppHandle,
}

impl LogEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl LogWriter for LogEmitter {
    fn write(&self, now: &mut DeferredNow, record: &Record<'_>) -> std::io::Result<()> {
        if let Err(e) = self.app.emit("logging", formatter(now, record)) {
            eprint!("{e:?}");
        };

        Ok(())
    }
    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

fn formatter(now: &mut DeferredNow, record: &Record) -> String {
    let log_line = match record.level() {
        Level::Debug => format!(
            "<span class=\"text-cyan-500\">[DEBUG]</span> {}",
            record.args()
        ),
        Level::Error => format!(
            "<span class=\"text-red-500\">[ERROR]</span> {}",
            record.args()
        ),
        Level::Info => format!(
            "<span class=\"text-lime-500\">[ INFO]</span> {}",
            record.args()
        ),
        Level::Warn => format!(
            "<span class=\"text-yellow-500\">[ WARN]</span> {}",
            record.args()
        ),
        _ => format!(
            "<span class=\"text-base-content/50\">[OTHER]</span> {}",
            record.args()
        ),
    };

    let time = now.now().format("%H:%M:%S%.6f");

    format!("<span class=\"text-base-content/50\">{time}</span> {log_line}")
}

pub fn init_logging(app: AppHandle) {
    // Build the initial log specification
    let mut builder = LogSpecification::builder();
    builder
        .default(LevelFilter::Debug)
        .module("log", LevelFilter::Error)
        .module("rpc", LevelFilter::Error)
        .module("tao", LevelFilter::Error)
        .module("tokio", LevelFilter::Error)
        .module("winit", LevelFilter::Error)
        .module("winit-gtk", LevelFilter::Error)
        .module("reqwest", LevelFilter::Error)
        .module("tauri-plugin-http", LevelFilter::Error)
        .module("tauri-plugin-updater", LevelFilter::Error);

    let _ = Logger::with(builder.build())
        .log_to_writer(Box::new(LogEmitter::new(app.clone())))
        .add_writer("plain", Box::new(LogPlain::new(app)))
        .start()
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()));
}

pub fn log_command(title: &str, prefix: Option<String>, mut cmd: Vec<String>) {
    let max_line_length = 140;
    let mut formatted_cmd = Vec::new();
    let mut quote_next = false;
    let mut current_line = String::new();
    let mut is_first_line = true;
    let mut last_arg = String::new();

    if let Some(pr) = prefix {
        cmd.insert(0, pr);
    }

    debug!("-------------------------------------------------------------------");
    debug!("{title}");

    for (i, arg) in cmd.iter().enumerate() {
        if quote_next
            || (i == cmd.len() - 1)
            || VIDEO_EXTENSIONS.contains(
                &arg.rsplit('.')
                    .next()
                    .unwrap_or_default()
                    .to_lowercase()
                    .as_str(),
            )
        {
            formatted_cmd.push(format!("\"{}\"", arg));
            quote_next = false;
        } else {
            formatted_cmd.push(arg.to_string());
            // TODO: podman use also -i; command after should not be im quotes
            if ["-i", "-filter_complex", "-map", "-metadata"].contains(&arg.as_str()) {
                quote_next = true;
            }
        }
    }

    for arg in formatted_cmd {
        if last_arg == "-filter_complex" {
            if is_first_line {
                debug!("{current_line} \\");
                is_first_line = false;
            } else {
                plain!("{current_line} \\");
            }

            current_line.clear();

            let filter_lines: Vec<&str> = arg.split(';').collect();
            for (i, line) in filter_lines.iter().enumerate() {
                if i == filter_lines.len() - 1 {
                    plain!("{line} \\");
                } else {
                    plain!("{line}; \\");
                }
            }

            last_arg.clear();
        } else if current_line.len() + arg.len() + 1 < max_line_length {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(&arg);
        } else {
            if is_first_line {
                debug!("{current_line} \\");
                is_first_line = false;
            } else {
                plain!("{current_line} \\");
            }
            current_line = arg.clone();
        }

        last_arg = arg.clone();
    }

    if !current_line.is_empty() {
        if is_first_line {
            debug!("{current_line}");
        } else {
            plain!("{current_line}");
        }
    }
}
