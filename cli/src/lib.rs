//! Library for concerns and functions specific to the STEPS CLI, rather than the STEPS library
//!
//! This is kept separate to "dogfood" the STEPS lib interface by making the CLI use it,
//! to prevent overly tight coupling of the CLI and the main lib, and to keep CLI concerns totally
//! out of the public STEPS interface.

use std::time;

use anyhow::{Error, Result};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use itertools::{izip, Itertools};

use steps_core::cfg::SimConfig;
use steps_core::sim::{SimulationHandler, SimulationState};

use cfg::{CliCommand, CliOutputConfig, ReproduceConfig};
use io::{extract_sim_config_from_path, outputter_group_for_cli};

mod cfg;
mod io;

pub use cfg::CliConfig;

/// Run the CLI as specified by some `CliConfig`
pub fn run_cli_config(cfg: CliConfig) {
    match cfg.command {
        CliCommand::Simulate(sim_cli_cfg) => {
            run_simulations(&sim_cli_cfg.output_cfg, sim_cli_cfg.sim_cfg)
        }
        CliCommand::Reproduce(reproduce_cfg) => reproduce_simulations(&reproduce_cfg),
    }
}

/// Run the simulations with command line display and display error results if applicable
fn run_simulations(output_cfg: &CliOutputConfig, sim_cfg: SimConfig) {
    if let Err(e) = run_simulations_inner(output_cfg, sim_cfg) {
        report_error("Error: Failed to properly output results.", e);
    }
}

/// Reproduce simulation results by extracting settings and handing off to the normal `Simulate`
/// subcommand
fn reproduce_simulations(cfg: &ReproduceConfig) {
    match extract_sim_config_from_path(&cfg.input_path) {
        Ok(sim_cfg) => {
            if sim_cfg.seed.is_none() {
                eprintln!(
                    "Note: The simulations were previously run without a seed. \
                       Simulations will be run with the same settings but results will not be identical."
                );
            }

            run_simulations(&cfg.output_cfg, sim_cfg);
        }
        Err(e) => {
            report_error(
                "Error: Failed to read simulation options for reproduction",
                e,
            );
        }
    }
}

/// Run the simulations with command line display and pass error results up
fn run_simulations_inner(output_cfg: &CliOutputConfig, sim_cfg: SimConfig) -> Result<()> {
    // Create the progress bars
    const TARGET_UPDATE_INTERVAL: time::Duration = time::Duration::from_millis(500);
    let mut bar_handler = ProgressBarHandler::new(
        TARGET_UPDATE_INTERVAL,
        [
            styled_bar(sim_cfg.replicates as u64, "Replicate:"),
            styled_bar(sim_cfg.transfers as u64, "Transfer:"),
        ],
    );

    // Objects which manage the underlying simulations and the outputting of results
    let mut output_handler = outputter_group_for_cli(output_cfg, &sim_cfg)?;
    let mut simulation_handler =
        SimulationHandler::new(sim_cfg, output_cfg.should_track_mutations());

    while let Some(state) = simulation_handler.next_state() {
        let SimulationState {
            replicate,
            transfer,
            end_of_replicate,
            lineages,
            mutations,
        } = state;

        output_handler.record_lineages(replicate, transfer, lineages)?;

        if let Some(mutations) = mutations {
            output_handler.record_pruned_mutations(replicate, mutations)?;
            if end_of_replicate {
                output_handler.record_active_mutations(replicate, mutations)?;
            }
        }

        bar_handler.maybe_set_positions([replicate as u64 - 1, transfer as u64]);
    }

    Ok(())
}

/// Report an `error` and a `message` to the user
fn report_error(message: &str, error: Error) {
    eprintln!("{}", message);
    eprintln!("{:#}", error);
    eprintln!("Details:\n{:#?}", error);
}

/// Get `ProgressBar` with style options and a custom prefix set to use for displaying progress
fn styled_bar(len: u64, prefix: &str) -> ProgressBar {
    let bar = ProgressBar::with_draw_target(len, ProgressDrawTarget::stderr_nohz())
        .with_style(ProgressStyle::default_bar().template("{prefix} {wide_bar} [{pos}/{len}]"));
    bar.set_prefix(prefix);

    bar
}

/// Handler for multiple `indicatif::ProgressBar`s
struct ProgressBarHandler<const N: usize> {
    bars: [ProgressBar; N],
    update_interval: time::Duration,
    last_update: time::Instant,
}

impl<const N: usize> ProgressBarHandler<N> {
    /// Create new `ProgressBarHandler` taking ownership of underlying progress bars
    pub fn new(update_interval: time::Duration, bars: [ProgressBar; N]) -> Self {
        // ProgressBars are Arc under the hood, clone is Arc clone
        // Need to do this so bars don't interfere with panic messages
        let handles = bars.clone();
        let old_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            for handle in &handles {
                handle.abandon();
            }

            old_hook(info);
        }));

        let mut result = Self {
            bars,
            update_interval,
            last_update: time::Instant::now(),
        };
        // Make sure bars start cleared out
        result.set_positions([0; N]);
        result
    }

    /// Set positions of the handled bars
    pub fn set_positions(&mut self, positions: [u64; N]) {
        if let Some((first_updatable, _)) = izip!(positions, &self.bars)
            .find_position(|(position, bar)| *position != bar.position())
        {
            // Clear all bars that come after this
            for bar in self.bars.iter_mut().skip(first_updatable + 1).rev() {
                bar.finish_and_clear();
            }
            // Set position of this bar
            self.bars[first_updatable].set_position(positions[first_updatable]);
            // Reset/set positions for remaining bars
            for (position, bar) in izip!(positions, &mut self.bars).skip(first_updatable + 1) {
                bar.reset();
                bar.set_position(position);
            }
        }

        self.last_update = time::Instant::now();
    }

    /// Set positions of the handled bars only if enough time has elapsed
    pub fn maybe_set_positions(&mut self, positions: [u64; N]) {
        if self.last_update.elapsed() >= self.update_interval {
            self.set_positions(positions);
        }
    }
}

impl<const N: usize> Drop for ProgressBarHandler<N> {
    fn drop(&mut self) {
        // Clear all of the progress bars
        for bar in &self.bars {
            bar.finish_and_clear();
        }
    }
}
