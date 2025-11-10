use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use console::Emoji;

use super::StepContext;
use crate::util::resolve_path;

const LINK: Emoji<'_, '_> = Emoji("🔗", "link");

pub fn run(ctx: &mut StepContext<'_>) -> Result<()> {
    if ctx.cfg.user.dotfiles.is_empty() {
        ctx.status("No dotfiles requested, skipping module");
        return Ok(());
    }

    for (name, entry) in ctx.cfg.user.dotfiles.iter() {
        ctx.info(format!("{LINK} syncing dotfile {name}"));
        sync_entry(ctx, entry)?;
    }

    Ok(())
}

fn sync_entry(ctx: &mut StepContext<'_>, entry: &crate::config::DotfileEntry) -> Result<()> {
    let source = resolve_path(&entry.source, ctx.root)
        .with_context(|| format!("resolving {}", entry.source))?;
    let target = resolve_path(&entry.target, ctx.root)
        .with_context(|| format!("resolving {}", entry.target))?;

    if !source.exists() {
        bail!("source {} does not exist", source.display());
    }

    if source.is_dir() {
        if target.exists() {
            backup_existing(&target)?;
        }
        sync_dir(&source, &target)
    } else {
        if target.exists() {
            if files_differ(&source, &target)? {
                backup_existing(&target)?;
            } else {
                return Ok(());
            }
        }
        copy_file(&source, &target)
    }
}

fn sync_dir(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)
        .with_context(|| format!("creating directory {}", target.display()))?;

    visit_dir(source, source, target)
}

fn visit_dir(current: &Path, source_root: &Path, target_root: &Path) -> Result<()> {
    let entries = fs::read_dir(current)
        .with_context(|| format!("reading directory {}", current.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("reading directory entry in {}", current.display()))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(source_root)
            .with_context(|| format!("unable to compute relative path for {}", path.display()))?;
        let dest = target_root.join(rel);
        let file_type = entry
            .file_type()
            .with_context(|| format!("getting file type for {}", path.display()))?;
        if file_type.is_dir() {
            fs::create_dir_all(&dest)
                .with_context(|| format!("creating directory {}", dest.display()))?;
            visit_dir(&path, source_root, target_root)?;
        } else {
            copy_file(&path, &dest)?;
        }
    }
    Ok(())
}

fn copy_file(source: &Path, target: &Path) -> Result<()> {
    if target.exists() && !files_differ(source, target)? {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
    }

    fs::copy(source, target)
        .with_context(|| format!("copying {} -> {}", source.display(), target.display()))?;
    Ok(())
}

fn files_differ(a: &Path, b: &Path) -> Result<bool> {
    if !b.exists() {
        return Ok(true);
    }
    let a_meta = fs::metadata(a)?;
    let b_meta = fs::metadata(b)?;
    if a_meta.len() != b_meta.len() {
        return Ok(true);
    }
    let a_bytes = fs::read(a)?;
    let b_bytes = fs::read(b)?;
    Ok(a_bytes != b_bytes)
}

fn backup_existing(target: &Path) -> Result<()> {
    let backup = next_backup_path(target)?;
    fs::rename(target, &backup)
        .with_context(|| format!("renaming {} -> {}", target.display(), backup.display()))?;
    Ok(())
}

fn next_backup_path(target: &Path) -> Result<PathBuf> {
    let parent = target
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let name = target
        .file_name()
        .with_context(|| format!("{} has no file name", target.display()))?;
    let base = name.to_string_lossy();

    let mut attempt = 0;
    loop {
        let suffix = if attempt == 0 {
            "".to_string()
        } else {
            format!(".{}", attempt)
        };
        let candidate = parent.join(format!("{}.bak{}", base, suffix));
        if !candidate.exists() {
            return Ok(candidate);
        }
        attempt += 1;
    }
}
