//! Functions to run the simulations and output results with command line display
//! after configuration options are retrieved and processed

use std::{error::Error, time};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::{cfg::*, io::*, sim::*};

/// Run the `Simulate` subcommand with command line display
pub fn run_simulations(cfg: &SimulationsCLIConfig) {
    run_simulations_private(&cfg.output_cfg, &cfg.sim_cfg);
}

/// Run the simulations with command line display and display error results if applicable
///
/// Exists as a wrapped function to be reused by run_simulations and reproduce_simulations
fn run_simulations_private(output_cfg: &CLIOutputConfig, sim_cfg: &SimConfig) {
    if let Err(e) = run_simulations_inner(output_cfg, sim_cfg) {
        eprintln!("Error: Failed to properly output results.");
        eprintln!("Details:\n{:#?}", e);
    }
}

/// Run the simulations with command line display and pass error results up
///
/// To display the error results to the user use `run_simulations_outer`
fn run_simulations_inner(
    output_cfg: &CLIOutputConfig,
    sim_cfg: &SimConfig,
) -> Result<(), Box<dyn Error>> {
    let replicate_bar = styled_bar(sim_cfg.replicates as u64, "Replicate:");
    let transfer_bar = styled_bar(sim_cfg.transfers as u64, "Transfer:");
    // ProgressBars are Arc under the hood, clone is Arc clone
    // Need to do this so bars don't interfere with panic messages
    let hook_replicate_bar = replicate_bar.clone();
    let hook_transfer_bar = transfer_bar.clone();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        hook_replicate_bar.abandon();
        hook_transfer_bar.abandon();
        hook(info);
    }));
    // To pick how often to update the transfers bar
    let mut last_update = time::Instant::now();
    const TARGET_UPDATE_INTERVAL: time::Duration = time::Duration::from_millis(500);

    // Objects which manage the underlying simulations and the outputting of results
    let tracking_mutations = output_cfg.should_track_mutations();
    let mut simulation_handler = SimulationHandler::new(sim_cfg.to_owned(), tracking_mutations);
    let mut output_handler = OutputHandler::new(output_cfg, sim_cfg)?;

    for r in 1..=sim_cfg.replicates {
        simulation_handler.start_replicate();
        // All other lineages will be handled after transferring
        // Must handle the output for the initial lineages before any transfers
        output_handler.handle_output_for_transfer(r, 0, simulation_handler.lineages())?;

        // 1 index because t is day *1* after the first transfer
        for t in 1..=sim_cfg.transfers {
            simulation_handler.transfer();

            output_handler.handle_output_for_transfer(r, t, simulation_handler.lineages())?;
            if tracking_mutations {
                // Pruned mutations, no longer being used for sequencing, can be output then
                // cleared so the population_handler no longer has to keep them in memory
                output_handler
                    .output_pruned_mutations(r, simulation_handler.mutations().unwrap())?;
                simulation_handler.clear_pruned_mutations();
            }

            // Update progress bar only periodically to reduce time spent redrawing it
            if last_update.elapsed() >= TARGET_UPDATE_INTERVAL {
                if replicate_bar.position() < r as u64 - 1 {
                    // This weird trickery with removing the transfer bar first is required to make
                    // the display right
                    transfer_bar.finish_and_clear();
                    replicate_bar.set_position(r as u64 - 1);
                    transfer_bar.reset();
                }
                transfer_bar.set_position(t as u64);
                last_update = time::Instant::now();
            }
        }

        // Only *pruned* mutations have been output up until this point
        // Many mutations will not have been pruned by the end of replicate
        if tracking_mutations {
            output_handler
                .finish_replicate_mutations(r, simulation_handler.mutations().unwrap())?;
        }
    }

    transfer_bar.finish_and_clear();
    replicate_bar.finish_and_clear();
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

            run_simulations_private(&cfg.output_cfg, &sim_cfg);
        }
        Err(e) => {
            eprintln!("Error: Failed to read simulation options for reproduction");
            eprintln!("Details:\n{:#?}", e);
        }
    }
}

/// Get `ProgressBar` with style options and a custom prefix set to use for displaying progress
fn styled_bar(len: u64, prefix: &str) -> ProgressBar {
    let bar = ProgressBar::with_draw_target(len, ProgressDrawTarget::stderr_nohz())
        .with_style(ProgressStyle::default_bar().template("{prefix} {wide_bar} [{pos}/{len}]"));
    bar.set_prefix(prefix);

    bar
}
