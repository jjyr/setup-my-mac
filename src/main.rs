mod config;
mod examples;
mod runner;
mod steps;
mod sudo;
mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use steps::StepKind;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about = "Opinionated Mac bootstrapper", long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Comma separated list of steps to execute (defaults to everything)
    #[arg(long, value_delimiter = ',', value_enum)]
    steps: Option<Vec<StepKind>>,

    /// Print an example configuration to stdout and exit
    #[arg(long)]
    example_config: bool,
}

fn main() -> Result<()> {
    color_eyre::install().expect("failed to initialize color-eyre");
    init_tracing();

    let Cli {
        config: cfg_path,
        steps,
        example_config,
    } = Cli::parse();

    if example_config {
        print!("{}", examples::example_config());
        return Ok(());
    }

    println!("Using configuration file: {}", cfg_path.display());
    let bundle = config::load_config(&cfg_path)?;

    let mut runner = runner::Runner::new(bundle);
    runner.run(steps)?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
