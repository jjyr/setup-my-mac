# setup-my-mac

Single-file macOS management.

> The author is too stupid to learn nixos, so he asks AI to write this tool.

## What it does
- System tweaks: timezone, Touch ID for sudo, and trackpad prefs
- Homebrew packages installation
- SSH, Git config
- Dotfile sync

## Install
```bash
cargo install setup-my-mac
```

## Requirements
- macOS with administrator access (system tweaks require sudo)
- Rust toolchain for `cargo install`
- Homebrew available on `PATH` if you enable that step

## Quick start
```bash
# 1. Generate a fully documented starter config
setup-my-mac --example-config > config.toml

# 2. Edit the config
$EDITOR config.toml

# 3. Run
setup-my-mac
```
