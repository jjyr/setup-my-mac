use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn resolve_path(input: &str, base_dir: &Path) -> Result<PathBuf> {
    let trimmed = input.trim();
    if trimmed.starts_with('~') {
        let expanded = shellexpand::tilde(trimmed).into_owned();
        Ok(PathBuf::from(expanded))
    } else if Path::new(trimmed).is_absolute() {
        Ok(PathBuf::from(trimmed))
    } else {
        Ok(base_dir.join(trimmed))
    }
}

pub fn normalize_newlines(input: &str) -> String {
    let mut normalized = input.replace("\r\n", "\n");
    while normalized.ends_with('\n') {
        normalized.pop();
    }
    normalized.push('\n');
    normalized
}

pub fn write_if_changed(path: &Path, contents: &str) -> Result<bool> {
    if path.exists() {
        let existing = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if normalize_newlines(&existing) == normalize_newlines(contents) {
            return Ok(false);
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    std::fs::write(path, contents)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}
