use clap::Parser;

use steps_cli::{run_cli_config, CliConfig};

/// Entry-point for the main "steps" command-line executable
fn main() {
    let cfg = CliConfig::parse();
    run_cli_config(cfg);
}
