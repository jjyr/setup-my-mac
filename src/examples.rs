pub fn example_config() -> &'static str {
    EXAMPLE_CONFIG
}

const EXAMPLE_CONFIG: &str = r#"# Example configuration for setup-my-mac.
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
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn example_config_parses_successfully() {
        let cfg: Config = toml::from_str(example_config()).expect("example config parses");
        assert!(cfg.homebrew.enable);
        assert_eq!(cfg.user.dotfiles.len(), 2);
    }
}
