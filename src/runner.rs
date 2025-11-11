use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use console::style;
use dialoguer::Confirm;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{
    config::{Config, ConfigBundle},
    steps::{self, StepContext, StepKind},
    sudo::SudoHelper,
};

pub struct Runner {
    config: Config,
    root: PathBuf,
}

impl Runner {
    pub fn new(bundle: ConfigBundle) -> Self {
        Runner {
            config: bundle.config,
            root: bundle.root,
        }
    }

    pub fn run(&mut self, requested: Option<Vec<StepKind>>) -> Result<()> {
        let steps = match requested {
            Some(list) if !list.is_empty() => list,
            _ => self.default_steps(),
        };

        if steps.is_empty() {
            println!("{} No steps to run", style("↷").yellow());
            return Ok(());
        }

        if !self.confirm_steps(&steps)? {
            println!(
                "{} {}",
                style("↷").yellow(),
                style("Stop all steps").yellow()
            );
            return Ok(());
        }

        let mp = MultiProgress::new();
        let spinner_style = ProgressStyle::with_template("{spinner:.green} {msg}")?
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

        let mut sudo = SudoHelper::default();

        for step in steps {
            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style.clone());
            pb.enable_steady_tick(Duration::from_millis(120));
            pb.set_message(format!(
                "{} {}",
                style("›").cyan(),
                style(step.display_name()).bold()
            ));

            sudo.set_prompt_ui(pb.clone());
            let result = self.run_step(step, &mut sudo, pb.clone());
            sudo.clear_prompt_ui();

            match result {
                Ok(_) => pb.finish_with_message(format!(
                    "{} {}",
                    style("✔").green().bold(),
                    style(step.display_name()).bold()
                )),
                Err(err) => {
                    pb.finish_with_message(format!(
                        "{} {}",
                        style("✖").red().bold(),
                        style(step.display_name()).bold()
                    ));
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    fn run_step(&self, kind: StepKind, sudo: &mut SudoHelper, pb: ProgressBar) -> Result<()> {
        let mut ctx = StepContext {
            cfg: &self.config,
            root: &self.root,
            sudo,
            progress: pb,
        };

        match kind {
            StepKind::System => steps::system::run(&mut ctx),
            StepKind::Homebrew => steps::homebrew::run(&mut ctx),
            StepKind::Dotfiles => steps::dotfiles::run(&mut ctx),
            StepKind::Ssh => steps::ssh::run(&mut ctx),
            StepKind::Git => steps::git::run(&mut ctx),
        }
    }

    pub fn default_steps(&self) -> Vec<StepKind> {
        use StepKind::*;
        let mut steps = vec![System];

        if self.config.homebrew.enable {
            steps.push(Homebrew);
        }
        if !self.config.user.dotfiles.is_empty() {
            steps.push(Dotfiles);
        }
        if self.config.user.ssh.is_some() {
            steps.push(Ssh);
        }
        if self
            .config
            .user
            .git
            .as_ref()
            .map(|g| g.enable)
            .unwrap_or(false)
        {
            steps.push(Git);
        }

        steps
    }

    fn confirm_steps(&self, kinds: &[StepKind]) -> Result<bool> {
        if kinds.is_empty() {
            return Ok(true);
        }

        let joined = kinds
            .iter()
            .map(|k| k.display_name())
            .collect::<Vec<_>>()
            .join(", ");
        let prompt = format!("Run all steps ({})?", style(joined).bold());
        Confirm::new()
            .with_prompt(prompt)
            .default(true)
            .interact()
            .map_err(Into::into)
    }
}
