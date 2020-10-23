use indicatif::{ProgressBar, ProgressStyle};

use crate::{cfg::*, io::*, sim::*};

pub fn run_simulations(cfg: &SimulationsCLIConfig) {
    run_simulations_inner(&cfg.output_cfg, &cfg.sim_cfg);
}

fn run_simulations_inner(output_cfg: &OuputConfig, sim_cfg: &SimConfig) {
    let replicate_bar = styled_bar(sim_cfg.replicates as u64, "Replicate:");
    let mut population_handler = PopulationHandler::new(&sim_cfg);
    let mut output_handler = OutputHandler::new(&output_cfg, &sim_cfg);

    for r in 1..=sim_cfg.replicates {
        let transfer_bar = styled_bar(sim_cfg.transfers as u64, "Transfer:");

        population_handler.start_replicate();
        output_handler.start_replicate();
        output_handler.handle_lineages(r, 0, population_handler.lineages());

        for t in 1..=sim_cfg.transfers {
            population_handler.transfer();
            output_handler.handle_lineages(r, t, population_handler.lineages());

            if t % 4096 == 0 {
                transfer_bar.set_position(t as u64);
            }
        }

        output_handler.finish_replicate();

        transfer_bar.finish_and_clear();
        replicate_bar.inc(1);
    }
}

pub fn reproduce_simulations(cfg: &ReproduceConfig) {
    let sim_cfg = extract_sim_config(&cfg.input_path);

    if sim_cfg.seed.is_none() {
        println!("Note: The simulations were previously run without a seed. \
               Simulations will be run with the same settings but results will not be identical.");
    }

    run_simulations_inner(&cfg.output_cfg, &sim_cfg);
}

fn styled_bar(len: u64, prefix: &str) -> ProgressBar {
    let bar = ProgressBar::new(len)
        .with_style(ProgressStyle::default_bar().template("{prefix} {wide_bar} [{pos}/{len}]"));
    bar.set_prefix(prefix);

    bar
}
