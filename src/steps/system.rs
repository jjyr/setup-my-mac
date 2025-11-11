use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use console::Emoji;
use tempfile::NamedTempFile;
use tracing::info;

use super::StepContext;
use crate::config::SystemConfig;

const SPARKLES: Emoji<'_, '_> = Emoji("✨", "*");

pub fn run(ctx: &mut StepContext<'_>) -> Result<()> {
    let system = &ctx.cfg.system;

    if !has_system_tasks(system) {
        ctx.status("No system settings configured, skipping");
        return Ok(());
    }

    if let Some(tz) = &system.timezone {
        ctx.info(format!("{SPARKLES} Setting timezone to {tz}"));
        ensure_timezone(ctx, tz)?;
    }

    if system.touch_id_sudo {
        ctx.info(format!("{SPARKLES} Enabling Touch ID for sudo"));
        enable_touch_id(ctx)?;
    }

    if let Some(clicking) = system.trackpad.clicking {
        ctx.info(format!("{SPARKLES} Trackpad clicking -> {}", clicking));
        ensure_trackpad_bool("com.apple.AppleMultitouchTrackpad", "Clicking", clicking)?;
        ensure_trackpad_bool(
            "com.apple.driver.AppleBluetoothMultitouch.trackpad",
            "Clicking",
            clicking,
        )?;
    }

    if let Some(drag) = system.trackpad.three_finger_drag {
        ctx.info(format!("{SPARKLES} Trackpad three finger drag -> {}", drag));
        ensure_trackpad_bool(
            "com.apple.AppleMultitouchTrackpad",
            "TrackpadThreeFingerDrag",
            drag,
        )?;
        ensure_trackpad_bool(
            "com.apple.driver.AppleBluetoothMultitouch.trackpad",
            "TrackpadThreeFingerDrag",
            drag,
        )?;
    }

    Ok(())
}

fn has_system_tasks(system: &SystemConfig) -> bool {
    system.timezone.is_some()
        || system.touch_id_sudo
        || system.trackpad.clicking.is_some()
        || system.trackpad.three_finger_drag.is_some()
}

fn ensure_timezone(ctx: &mut StepContext<'_>, target: &str) -> Result<()> {
    let current = ctx
        .sudo
        .run_with_output("/usr/sbin/systemsetup", &["-gettimezone"])
        .unwrap_or_default();

    if current.trim().ends_with(target) {
        info!("timezone already {target}");
        return Ok(());
    }

    ctx.sudo
        .run("/usr/sbin/systemsetup", &["-settimezone", target])
        .with_context(|| format!("unable to set timezone to {target}"))
}

fn enable_touch_id(ctx: &mut StepContext<'_>) -> Result<()> {
    let pam_path = Path::new("/etc/pam.d/sudo_local");
    let contents = match fs::read_to_string(pam_path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            fs::read_to_string("/etc/pam.d/sudo").with_context(|| "reading /etc/pam.d/sudo")?
        }
        Err(err) => {
            return Err(err).with_context(|| "reading /etc/pam.d/sudo_local");
        }
    };
    if contents.contains("pam_tid.so") {
        return Ok(());
    }

    let mut tmp = NamedTempFile::new().context("allocating temp file")?;
    writeln!(tmp, "auth       sufficient     pam_tid.so")?;
    write!(tmp, "{contents}")?;
    tmp.flush()?;

    let tmp_path = tmp.path().to_str().context("temp path not valid utf8")?;
    let dest = pam_path.to_str().context("pam path not utf8")?;
    ctx.sudo
        .run("/bin/cp", &[tmp_path, dest])
        .context("updating /etc/pam.d/sudo_local")?;
    ctx.sudo
        .run("/bin/chmod", &["644", dest])
        .context("fixing pam perms")?;

    Ok(())
}

fn ensure_trackpad_bool(domain: &str, key: &str, desired: bool) -> Result<()> {
    let current = read_defaults_bool(domain, key);
    if current == Some(desired) {
        return Ok(());
    }

    let flag = if desired { "TRUE" } else { "FALSE" };
    let status = Command::new("/usr/bin/defaults")
        .args(["write", domain, key, "-bool", flag])
        .status()
        .with_context(|| format!("defaults write {domain} {key}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("defaults write {domain} {key} failed"))
    }
}

fn read_defaults_bool(domain: &str, key: &str) -> Option<bool> {
    let output = Command::new("/usr/bin/defaults")
        .args(["read", domain, key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    match raw.trim() {
        "1" | "YES" | "TRUE" | "true" => Some(true),
        "0" | "NO" | "FALSE" | "false" => Some(false),
        _ => None,
    }
}
