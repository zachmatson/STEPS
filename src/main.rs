use relltee::{
    cfg::{Config, Subcommand::*},
    cli,
};

fn main() {
    let cfg = Config::from_args();

    match cfg.subcommand {
        Simulate(x) => cli::run_simulations(&x),
        Reproduce(x) => cli::reproduce_simulations(&x),
    }
}
