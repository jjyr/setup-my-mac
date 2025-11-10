use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub system: SystemConfig,
    #[serde(default)]
    pub homebrew: HomebrewConfig,
    pub user: UserConfig,
}

#[derive(Debug)]
pub struct ConfigBundle {
    pub config: Config,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub root: PathBuf,
}

pub fn load_config(path: &Path) -> Result<ConfigBundle> {
    let data =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let config: Config =
        toml::from_str(&data).with_context(|| format!("Invalid TOML in {}", path.display()))?;
    let root = path
        .parent()
        .map(|p| p.to_owned())
        .unwrap_or_else(|| Path::new(".").to_owned());

    Ok(ConfigBundle {
        config,
        path: path.to_owned(),
        root,
    })
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    #[allow(dead_code)]
    pub home_directory: PathBuf,
    #[allow(dead_code)]
    pub primary_user: String,
    pub timezone: Option<String>,
    #[serde(default)]
    pub touch_id_sudo: bool,
    #[serde(default)]
    pub trackpad: TrackpadConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct TrackpadConfig {
    pub clicking: Option<bool>,
    #[serde(rename = "three_finger_drag")]
    pub three_finger_drag: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct HomebrewConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub brews: Vec<String>,
    #[serde(default)]
    pub casks: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub ssh: Option<SshConfig>,
    #[serde(default)]
    pub dotfiles: HashMap<String, DotfileEntry>,
    pub git: Option<GitConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SshConfig {
    pub config: String,
}

#[derive(Debug, Deserialize)]
pub struct DotfileEntry {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Deserialize)]
pub struct GitConfig {
    #[serde(default)]
    pub enable: bool,
    pub user_email: Option<String>,
    pub user_name: Option<String>,
    pub credential_helper: Option<String>,
    #[serde(default)]
    pub ignores: Vec<String>,
    pub init: Option<GitInit>,
    pub merge: Option<GitMerge>,
    pub pull: Option<GitPull>,
    pub push: Option<GitPush>,
}

#[derive(Debug, Deserialize)]
pub struct GitInit {
    pub default_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitMerge {
    pub conflictstyle: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitPull {
    pub rebase: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GitPush {
    pub auto_setup_remote: Option<bool>,
}
