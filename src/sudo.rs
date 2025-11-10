use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use tracing::debug;

const SUDO_TTL: Duration = Duration::from_secs(4 * 60);

#[derive(Default)]
pub struct SudoHelper {
    valid_until: Option<Instant>,
}

impl SudoHelper {
    pub fn run(&mut self, program: &str, args: &[&str]) -> Result<()> {
        let output = self.exec(program, args)?;
        if output.status.success() {
            if !output.stdout.is_empty() {
                debug!(
                    "sudo {} stdout: {}",
                    program,
                    String::from_utf8_lossy(&output.stdout)
                );
            }
            if !output.stderr.is_empty() {
                debug!(
                    "sudo {} stderr: {}",
                    program,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok(())
        } else {
            Err(anyhow!(
                "command `{}` failed: {}",
                program,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub fn run_with_output(&mut self, program: &str, args: &[&str]) -> Result<String> {
        let output = self.exec(program, args)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            Err(anyhow!(
                "command `{}` failed: {}",
                program,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn exec(&mut self, program: &str, args: &[&str]) -> Result<std::process::Output> {
        self.ensure_ticket()?;

        let mut child = Command::new("sudo");
        child.arg(program);
        child.args(args);
        child.stdin(Stdio::inherit());
        child.stdout(Stdio::piped());
        child.stderr(Stdio::piped());

        let output = child
            .spawn()
            .with_context(|| format!("unable to spawn sudo for {}", program))?
            .wait_with_output()
            .context("failed to wait for sudo command")?;

        if !output.status.success() {
            // Force revalidation next time to avoid stale auth.
            self.valid_until = None;
        }

        Ok(output)
    }

    fn ensure_ticket(&mut self) -> Result<()> {
        if self
            .valid_until
            .map(|until| until > Instant::now())
            .unwrap_or(false)
        {
            return Ok(());
        }

        if Command::new("sudo")
            .args(["-n", "true"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            self.valid_until = Some(Instant::now() + SUDO_TTL);
            return Ok(());
        }

        self.prompt_through_sudo()
    }

    fn prompt_through_sudo(&mut self) -> Result<()> {
        let status = Command::new("sudo")
            .arg("-v")
            .status()
            .context("failed to refresh sudo credentials")?;
        if status.success() {
            self.valid_until = Some(Instant::now() + SUDO_TTL);
            Ok(())
        } else {
            Err(anyhow!("sudo authentication failed"))
        }
    }
}
