pub mod dotfiles;
pub mod git;
pub mod homebrew;
pub mod ssh;
pub mod system;

use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;

use anyhow::{Context, Result};
use clap::ValueEnum;
use console::style;
use indicatif::ProgressBar;

use crate::{config::Config, sudo::SudoHelper};

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq, Hash)]
pub enum StepKind {
    System,
    Homebrew,
    Dotfiles,
    Ssh,
    Git,
}

impl StepKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            StepKind::System => "System",
            StepKind::Homebrew => "Homebrew",
            StepKind::Dotfiles => "Dotfiles",
            StepKind::Ssh => "SSH",
            StepKind::Git => "Git",
        }
    }
}

pub struct StepContext<'cfg> {
    pub cfg: &'cfg Config,
    pub root: &'cfg Path,
    pub sudo: &'cfg mut SudoHelper,
    pub progress: ProgressBar,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum StepLogLevel {
    Info,
    Warn,
    Error,
}

impl<'cfg> StepContext<'cfg> {
    pub fn status(&self, message: impl Into<String>) {
        self.progress.set_message(message.into());
    }

    pub fn log(&self, level: StepLogLevel, message: impl AsRef<str>) {
        let label = match level {
            StepLogLevel::Info => style("[info]").dim().to_string(),
            StepLogLevel::Warn => style("[warn]").yellow().to_string(),
            StepLogLevel::Error => style("[error]").red().to_string(),
        };

        self.progress
            .println(format!("  {} {}", label, message.as_ref()));
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.log(StepLogLevel::Info, message);
    }

    #[allow(dead_code)]
    pub fn warn(&self, message: impl AsRef<str>) {
        self.log(StepLogLevel::Warn, message);
    }

    #[allow(dead_code)]
    pub fn error(&self, message: impl AsRef<str>) {
        self.log(StepLogLevel::Error, message);
    }

    pub fn stream_command(&self, mut command: Command, label: &str) -> Result<ExitStatus> {
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .with_context(|| format!("failed to spawn {label}"))?;

        let stdout_handle = child.stdout.take().map(|stdout| {
            spawn_stream_logger(self.progress.clone(), label.to_string(), "stdout", stdout)
        });

        let stderr_handle = child.stderr.take().map(|stderr| {
            spawn_stream_logger(self.progress.clone(), label.to_string(), "stderr", stderr)
        });

        let status = child
            .wait()
            .with_context(|| format!("failed to wait for {label}"))?;

        if let Some(handle) = stdout_handle {
            let _ = handle.join();
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        Ok(status)
    }
}

fn spawn_stream_logger<R>(
    progress: ProgressBar,
    label: String,
    stream: &'static str,
    reader: R,
) -> thread::JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let prefix = if stream == "stdout" {
            style(format!("[{label}]")).dim().to_string()
        } else {
            style(format!("[{label}:{stream}]")).dim().to_string()
        };

        let reader = BufReader::new(reader);
        for line in reader.lines() {
            match line {
                Ok(line) => progress.println(format!("  {} {}", prefix, line)),
                Err(err) => {
                    let error_prefix = style(format!("[{label}:{stream}]")).red().to_string();
                    progress.println(format!("  {} stream read error: {err}", error_prefix));
                    break;
                }
            }
        }
    })
}
