use structopt::StructOpt;

mod cli;

fn main() {
    let cfg = cli::CliConfig::from_args();
    cli::run_cli_config(cfg);
}
