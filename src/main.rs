use relltee::{
    cfg::{Config, Subcommand::*},
    sim,
};

fn main() {
    let cfg = Config::from_args();

    match cfg.subcommand {
        Simulate(x) => sim::run_simulations(&x),
        Format(_) => todo!("Create formatting mode"),
    }
}
