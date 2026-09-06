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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::fs;
    use tempfile::TempDir;

    /// Body lines of an output file, with the two JSON header lines dropped
    fn body(path: &std::path::Path) -> Vec<String> {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .skip(2)
            .map(str::to_string)
            .collect()
    }

    /// Run the CLI as if the given arguments had been passed on the command line
    fn run(args: &[&str]) {
        run_cli_config(CliConfig::try_parse_from(args).unwrap());
    }

    /// Arguments for a short, seeded run writing a summary to `path`
    fn simulate_args(path: &str) -> Vec<String> {
        ["steps", "simulate", "-r", "1", "-t", "3", "--seed", "42"]
            .iter()
            .map(|s| s.to_string())
            .chain(["--summary-output".to_string(), path.to_string()])
            .collect()
    }

    #[test]
    fn test_simulate_writes_a_row_per_transfer() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("summary.csv");
        let args = simulate_args(path.to_str().unwrap());

        run(&args.iter().map(String::as_str).collect::<Vec<_>>());

        // Header row, then transfers 0 through 3
        assert_eq!(body(&path).len(), 5);
    }

    #[test]
    fn test_simulate_writes_every_requested_output() {
        let dir = TempDir::new().unwrap();
        let paths = ["summary.csv", "raw.ndjson", "seq.ndjson", "muts.csv"]
            .map(|name| dir.path().join(name));

        run(&[
            "steps",
            "simulate",
            "-r",
            "1",
            "-t",
            "2",
            "--seed",
            "42",
            "--summary-output",
            paths[0].to_str().unwrap(),
            "--raw-output",
            paths[1].to_str().unwrap(),
            "--sequencing-output",
            paths[2].to_str().unwrap(),
            "--mutation-summary-output",
            paths[3].to_str().unwrap(),
        ]);

        for path in &paths {
            let contents = fs::read_to_string(path).unwrap();
            assert!(!contents.is_empty(), "{} was empty", path.display());
        }
    }

    #[test]
    fn test_sampling_frequency_reduces_recorded_transfers() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("summary.csv");

        run(&[
            "steps",
            "simulate",
            "-r",
            "1",
            "-t",
            "4",
            "--seed",
            "42",
            "-f",
            "2",
            "--summary-output",
            path.to_str().unwrap(),
        ]);

        // Header row, then only transfers 0, 2 and 4
        assert_eq!(body(&path).len(), 4);
    }

    #[test]
    fn test_reproduce_reruns_a_seeded_simulation_identically() {
        let dir = TempDir::new().unwrap();
        let original = dir.path().join("original.csv");
        let repeated = dir.path().join("repeated.csv");

        let args = simulate_args(original.to_str().unwrap());
        run(&args.iter().map(String::as_str).collect::<Vec<_>>());

        run(&[
            "steps",
            "reproduce",
            original.to_str().unwrap(),
            "--summary-output",
            repeated.to_str().unwrap(),
        ]);

        // The recorded config carries the seed, so the rerun must match exactly
        let rows = body(&original);
        assert_eq!(rows.len(), 5, "the original run should have produced rows");
        assert_eq!(rows, body(&repeated));
    }

    #[test]
    fn test_reproduce_reports_a_bad_input_path_without_panicking() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("nope.csv");
        let output = dir.path().join("out.csv");

        run(&[
            "steps",
            "reproduce",
            missing.to_str().unwrap(),
            "--summary-output",
            output.to_str().unwrap(),
        ]);

        // Errors are reported to stderr rather than propagated, so no output is produced
        assert!(!output.exists());
    }

    #[test]
    fn test_handler_sets_positions_of_all_bars() {
        let mut handler =
            ProgressBarHandler::new(time::Duration::ZERO, [hidden_bar(10), hidden_bar(20)]);

        handler.set_positions([3, 4]);

        assert_eq!(handler.bars[0].position(), 3);
        assert_eq!(handler.bars[1].position(), 4);
    }

    #[test]
    fn test_handler_skips_update_before_the_interval_elapses() {
        let mut handler = ProgressBarHandler::new(
            time::Duration::from_secs(3600),
            [hidden_bar(10), hidden_bar(20)],
        );

        handler.maybe_set_positions([3, 4]);

        assert_eq!(handler.bars[0].position(), 0);
        assert_eq!(handler.bars[1].position(), 0);
    }

    #[test]
    fn test_handler_updates_once_the_interval_has_elapsed() {
        let mut handler =
            ProgressBarHandler::new(time::Duration::ZERO, [hidden_bar(10), hidden_bar(20)]);

        handler.maybe_set_positions([3, 4]);

        assert_eq!(handler.bars[0].position(), 3);
    }

    /// A progress bar which does not draw anything, to keep test output clean
    fn hidden_bar(len: u64) -> ProgressBar {
        ProgressBar::with_draw_target(len, ProgressDrawTarget::hidden())
    }
}
