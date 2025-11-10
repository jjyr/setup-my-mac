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

## Config
`setup-my-mac --example-config` prints the same annotated example stored in `src/examples.rs`, so you can copy, trim, or extend it as needed. Every section is optional unless otherwise noted, and you can delete blocks you do not care about.

```toml
# Example configuration for setup-my-mac.
# Save as config.toml and customize each section for your machine.

[system]
# Core system identity and timezone tweaks.
home_directory = "/Users/your-user"
primary_user = "your-user"
timezone = "America/Los_Angeles"
touch_id_sudo = true

[system.trackpad]
# Optional trackpad prefs; set to true/false or remove entirely.
clicking = true
three_finger_drag = true

[homebrew]
# Enable Homebrew automation and list formulas/casks to install.
enable = true
brews = [
  "git",
  "ripgrep",
  "fzf",
]
casks = [
  "iterm2",
  "visual-studio-code",
]

[user.ssh]
# Raw SSH config content written to ~/.ssh/config.
config = """
Host github.com
    HostName github.com
    User git
    AddKeysToAgent yes
    IdentityFile ~/.ssh/id_ed25519
"""

# Dotfile entries describe source -> target sync pairs.
[user.dotfiles.zshrc]
source = "dotfiles/zshrc"
target = "~/.zshrc"

[user.dotfiles.nvim]
source = "dotfiles/nvim"
target = "~/.config/nvim"

[user.git]
# Enable Git preferences and supply common options.
enable = true
user_name = "Your Name"
user_email = "you@example.com"
credential_helper = "osxkeychain"
ignores = [
  "target/",
  ".DS_Store",
]

[user.git.init]
default_branch = "main"

[user.git.merge]
conflictstyle = "zdiff3"

[user.git.pull]
rebase = true

[user.git.push]
auto_setup_remote = true
```
