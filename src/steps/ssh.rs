use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};
use console::Emoji;
use tracing::info;

use super::StepContext;
use crate::util::{normalize_newlines, resolve_path, write_if_changed};

const KEY: Emoji<'_, '_> = Emoji("🗝", "ssh");

pub fn run(ctx: &mut StepContext<'_>) -> Result<()> {
    let Some(ssh_cfg) = ctx.cfg.user.ssh.as_ref() else {
        ctx.status("No SSH config provided, skipping");
        return Ok(());
    };

    ctx.status(format!("{KEY} syncing ~/.ssh/config"));

    let ssh_dir = resolve_path("~/.ssh", ctx.root)?;
    fs::create_dir_all(&ssh_dir).context("creating ~/.ssh")?;
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&ssh_dir)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&ssh_dir, perms)?;
    }

    let config_path = ssh_dir.join("config");
    let content = normalize_newlines(&ssh_cfg.config);

    ctx.info(format!(
        "updating \n------\n{content}\n------\n{}",
        config_path.display()
    ));
    info!(
        "updating \n------\n{content}\n------\n{}",
        config_path.display()
    );

    let changed = write_if_changed(&config_path, &content)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&config_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&config_path, perms)?;
    }

    if changed {
        ctx.info(format!("updated {}", config_path.display()));
        info!("updated {}", config_path.display());
    } else {
        ctx.info(format!("{} already up to date", config_path.display()));
        info!("{} already up to date", config_path.display());
    }

    Ok(())
}
