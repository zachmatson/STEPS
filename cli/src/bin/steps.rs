use clap::Parser;

mod cli;

fn main() {
    let cfg = cli::CliConfig::parse();
    cli::run_cli_config(cfg);
}
