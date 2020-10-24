//! Functions to run the simulations and output results with command line display
//! after configuration options are retrieved and processed

use std::error::Error;

use indicatif::{ProgressBar, ProgressStyle};

use crate::{cfg::*, io::*, sim::*};

/// Run the `Simulate` subcommand with command line display
pub fn run_simulations(cfg: &SimulationsCLIConfig) {
    run_simulations_outer(&cfg.output_cfg, &cfg.sim_cfg);
}

/// Run the simulations with command line display and display error results if applicable
fn run_simulations_outer(output_cfg: &OuputConfig, sim_cfg: &SimConfig) {
    if let Err(e) = run_simulations_inner(output_cfg, sim_cfg) {
        eprintln!("Error: Failed to properly output results.");
        eprintln!("Details:\n{:#?}", e);
    }
}

/// Run the simulations with command line display and pass error results up
///
/// To display the error results to the user use `run_simulations_outer`
fn run_simulations_inner(
    output_cfg: &OuputConfig,
    sim_cfg: &SimConfig,
) -> Result<(), Box<dyn Error>> {
    let replicate_bar = styled_bar(sim_cfg.replicates as u64, "Replicate:");
    // Objects which manage the underlying simulations and the outputting of results
    let mut population_handler = SimulationHandler::new(&sim_cfg);
    let mut output_handler = OutputHandler::new(&output_cfg, &sim_cfg)?;

    for r in 1..=sim_cfg.replicates {
        let transfer_bar = styled_bar(sim_cfg.transfers as u64, "Transfer:");

        population_handler.start_replicate();
        output_handler.start_replicate()?;
        // All other lineages will be handled after transferring
        // Must handle the output for the initial lineages before any transfers
        output_handler.handle_lineages(r, 0, population_handler.lineages())?;

        // 1 index because t is day *1* after the first transfer
        for t in 1..=sim_cfg.transfers {
            population_handler.transfer();
            output_handler.handle_lineages(r, t, population_handler.lineages())?;

            // Update progress bar only periodically to reduce time spent redrawing it
            if t % 4096 == 0 {
                transfer_bar.set_position(t as u64);
            }
        }

        output_handler.finish_replicate()?;

        // Must reset the transfer bar this way to make the display work for the replicate bar
        // when it gets incremented
        transfer_bar.finish_and_clear();
        replicate_bar.inc(1);
    }

    Ok(())
}

/// Reproduce simulation results by extracting settings and handing off to the normal
/// `Simulate` subcommand
pub fn reproduce_simulations(cfg: &ReproduceConfig) {
    match extract_sim_config(&cfg.input_path) {
        Ok(sim_cfg) => {
            if sim_cfg.seed.is_none() {
                eprintln!(
                    "Note: The simulations were previously run without a seed. \
                       Simulations will be run with the same settings but results will not be identical."
                );
            }

            run_simulations_outer(&cfg.output_cfg, &sim_cfg);
        }
        Err(e) => {
            eprintln!("Error: Failed to read simulation options for reproduction");
            eprintln!("Details:\n{:#?}", e);
        }
    }
}

/// Get `ProgressBar` with style options and a custom prefix set to use for displaying progress
fn styled_bar(len: u64, prefix: &str) -> ProgressBar {
    let bar = ProgressBar::new(len)
        .with_style(ProgressStyle::default_bar().template("{prefix} {wide_bar} [{pos}/{len}]"));
    bar.set_prefix(prefix);

    bar
}
