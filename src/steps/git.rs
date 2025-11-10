use std::process::Command;

use anyhow::{anyhow, Context, Result};
use console::Emoji;
use tracing::info;

use super::StepContext;
use crate::{
    config::GitConfig,
    util::{resolve_path, write_if_changed},
};

const BRANCH: Emoji<'_, '_> = Emoji("🌿", "git");

pub fn run(ctx: &mut StepContext<'_>) -> Result<()> {
    let Some(git_cfg) = ctx.cfg.user.git.as_ref() else {
        ctx.status("No git settings configured, skipping");
        return Ok(());
    };

    if !git_cfg.enable {
        ctx.status("user.git.enable is false, skipping git module");
        return Ok(());
    }

    if !has_git_work(git_cfg) {
        ctx.status("No git preferences provided, skipping");
        return Ok(());
    }

    ctx.status(format!("{BRANCH} applying git config"));

    if let Some(name) = &git_cfg.user_name {
        ensure_git_config("user.name", name)?;
    }
    if let Some(email) = &git_cfg.user_email {
        ensure_git_config("user.email", email)?;
    }
    if let Some(helper) = &git_cfg.credential_helper {
        ensure_git_config("credential.helper", helper)?;
    }
    if let Some(init) = &git_cfg.init {
        if let Some(branch) = &init.default_branch {
            ensure_git_config("init.defaultBranch", branch)?;
        }
    }
    if let Some(merge) = &git_cfg.merge {
        if let Some(style) = &merge.conflictstyle {
            ensure_git_config("merge.conflictStyle", style)?;
        }
    }
    if let Some(pull) = &git_cfg.pull {
        if let Some(rebase) = pull.rebase {
            ensure_git_config("pull.rebase", if rebase { "true" } else { "false" })?;
        }
    }
    if let Some(push) = &git_cfg.push {
        if let Some(auto) = push.auto_setup_remote {
            ensure_git_config("push.autoSetupRemote", if auto { "true" } else { "false" })?;
        }
    }

    if !git_cfg.ignores.is_empty() {
        ensure_global_ignore(ctx, &git_cfg.ignores)?;
    }

    Ok(())
}

fn ensure_git_config(key: &str, value: &str) -> Result<()> {
    if git_get(key)?.as_deref() == Some(value) {
        info!("git {key} already set");
        return Ok(());
    }

    let status = Command::new("git")
        .args(["config", "--global", key, value])
        .status()
        .with_context(|| format!("setting git {key}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("git config for {key} failed"))
    }
}

fn git_get(key: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .with_context(|| format!("reading git {key}"))?;
    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else if output.status.code() == Some(1) {
        Ok(None)
    } else {
        Err(anyhow!(
            "git config --get {key} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn ensure_global_ignore(ctx: &StepContext<'_>, ignores: &[String]) -> Result<()> {
    let path = resolve_path("~/.config/git/ignore", ctx.root)?;
    let mut body = ignores.join("\n");
    body.push('\n');
    let changed = write_if_changed(&path, &body)?;
    if changed {
        info!("updated global gitignore at {}", path.display());
    }

    let path_str = path
        .to_str()
        .context("global ignore path contains invalid UTF-8")?;
    ensure_git_config("core.excludesFile", path_str)
}

fn has_git_work(cfg: &GitConfig) -> bool {
    cfg.user_name.is_some()
        || cfg.user_email.is_some()
        || cfg.credential_helper.is_some()
        || cfg
            .init
            .as_ref()
            .and_then(|init| init.default_branch.as_ref())
            .is_some()
        || cfg
            .merge
            .as_ref()
            .and_then(|merge| merge.conflictstyle.as_ref())
            .is_some()
        || cfg.pull.as_ref().and_then(|pull| pull.rebase).is_some()
        || cfg
            .push
            .as_ref()
            .and_then(|push| push.auto_setup_remote)
            .is_some()
        || !cfg.ignores.is_empty()
}
