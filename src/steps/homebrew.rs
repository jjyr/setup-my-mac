use std::io::Write;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use console::Emoji;
use tempfile::NamedTempFile;
use tracing::info;

use super::StepContext;
use crate::config::HomebrewConfig;

const PACKAGE: Emoji<'_, '_> = Emoji("📦", "[pkg]");

pub fn run(ctx: &mut StepContext<'_>) -> Result<()> {
    let hb = &ctx.cfg.homebrew;
    if !hb.enable {
        ctx.status("Homebrew disabled in config, skipping");
        return Ok(());
    }

    ensure_brew_available()?;

    if hb.brews.is_empty() && hb.casks.is_empty() {
        ctx.status("No Homebrew packages configured, skipping");
        return Ok(());
    }

    ctx.status(format!("{PACKAGE} brew bundle"));
    ensure_bundle(ctx, hb)
}

fn ensure_brew_available() -> Result<()> {
    let status = Command::new("brew")
        .arg("--version")
        .status()
        .context("failed to invoke brew")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("brew is not available"))
    }
}

fn ensure_bundle(ctx: &StepContext<'_>, cfg: &HomebrewConfig) -> Result<()> {
    info!(
        "Preparing Brewfile with {} brews and {} casks",
        cfg.brews.len(),
        cfg.casks.len()
    );

    let brewfile_contents = render_brewfile(cfg);
    let mut tmp = NamedTempFile::new().context("failed to create temporary Brewfile")?;
    tmp.write_all(brewfile_contents.as_bytes())
        .context("failed to write temporary Brewfile contents")?;

    let mut command = Command::new("brew");
    command.arg("bundle");
    command.arg("--file");
    command.arg(tmp.path());

    let status = ctx.stream_command(command, "brew bundle")?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("brew bundle failed"))
    }
}

fn render_brewfile(cfg: &HomebrewConfig) -> String {
    let mut contents = String::new();
    for formula in &cfg.brews {
        contents.push_str(&format!("brew \"{}\"\n", formula));
    }
    for cask in &cfg.casks {
        contents.push_str(&format!("cask \"{}\"\n", cask));
    }
    contents
}
