use relltee::{Subcommand::*, *};

fn main() {
    let cfg = Config::from_args();

    let cfg = match cfg.subcommand {
        Simulate(x) => run_simulations(&x),
        Format(_) => todo!("Write output conversions"),
    };
}
