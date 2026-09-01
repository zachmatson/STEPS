//! Configuration options specifically for the CLI portion of STEPS
use std::path::PathBuf;

use clap::{AppSettings, Parser, Subcommand};

use steps_core::cfg::{SimConfig, SummaryOutputConfig};

/// Configuration options for STEPS command line app subcommands
#[derive(Parser)]
#[clap(version, about = "Serially Transferred Evolving Population Simulator")]
pub struct CliConfig {
    /// Subcommands of STEPS
    #[clap(subcommand)]
    pub command: CliCommand,
}

/// Subcommand definitions
#[derive(Subcommand)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub enum CliCommand {
    /// Run simulations
    Simulate(SimulateConfig),
    /// Reproduce results from a previous simulation run  
    Reproduce(ReproduceConfig),
}

/// Run the STEPS simulation
#[derive(Parser)]
#[clap(version, setting = AppSettings::DeriveDisplayOrder)]
pub struct SimulateConfig {
    /// Output options for the CLI
    #[clap(flatten)]
    pub output_cfg: CliOutputConfig,

    /// Simulation options
    #[clap(flatten)]
    pub sim_cfg: SimConfig,
}

/// Reproduce results of a previous run of the STEPS simulation
#[derive(Parser)]
#[clap(version, setting = AppSettings::DeriveDisplayOrder)]
pub struct ReproduceConfig {
    /// Path of the input file, which came from a previous run and contains the information needed
    /// to reproduce the results
    pub input_path: PathBuf,

    /// Output options for the CLI
    #[clap(flatten)]
    pub output_cfg: CliOutputConfig,
}

/// Command line inputs needed to output results
#[derive(Parser)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct CliOutputConfig {
    /// The rate at which populations should be sampled
    #[clap(short = 'f', long, default_value = "1")]
    pub sampling_frequency: u32,

    /// Path to output the summarized simulation results (as CSV), which contains the fitness and
    /// other enabled stats over time
    #[clap(short = 'o', long = "summary-output")]
    pub summary_output_path: Option<PathBuf>,

    /// Path to output the full raw simulation results (as ndjson), which includes full data for all
    /// lineages at each sampled interval
    #[clap(short = 'j', long = "raw-output")]
    pub raw_output_path: Option<PathBuf>,

    /// Path to output information about all mutations that occur (as ndjson), which includes
    /// change in fitness and IDs for all mutations over time
    #[clap(short, long = "sequencing-output")]
    pub sequencing_output_path: Option<PathBuf>,

    /// Path to output summary information about mutations (as CSV)
    #[clap(long = "mutation-summary-output")]
    pub mutation_summary_output_path: Option<PathBuf>,

    /// Options for the summary output
    #[clap(flatten)]
    pub summary_cfg: SummaryOutputConfig,
}

impl CliOutputConfig {
    /// Should mutations be tracked?
    pub fn should_track_mutations(&self) -> bool {
        self.sequencing_output_path.is_some() || self.mutation_summary_output_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a `simulate` command line and return its output config
    fn output_cfg_from(args: &[&str]) -> CliOutputConfig {
        let cfg = CliConfig::try_parse_from(args).unwrap();
        match cfg.command {
            CliCommand::Simulate(simulate) => simulate.output_cfg,
            CliCommand::Reproduce(_) => panic!("expected a simulate command"),
        }
    }

    #[test]
    fn test_output_config_defaults() {
        let output_cfg = output_cfg_from(&["steps", "simulate"]);
        assert_eq!(output_cfg.sampling_frequency, 1);
        assert_eq!(output_cfg.summary_output_path, None);
        assert_eq!(output_cfg.raw_output_path, None);
        assert_eq!(output_cfg.sequencing_output_path, None);
        assert_eq!(output_cfg.mutation_summary_output_path, None);
    }

    #[test]
    fn test_output_config_parses_all_paths() {
        let output_cfg = output_cfg_from(&[
            "steps",
            "simulate",
            "-f",
            "5",
            "--summary-output",
            "summary.csv",
            "--raw-output",
            "raw.ndjson",
            "--sequencing-output",
            "seq.ndjson",
            "--mutation-summary-output",
            "muts.csv",
        ]);
        assert_eq!(output_cfg.sampling_frequency, 5);
        assert_eq!(
            output_cfg.summary_output_path,
            Some(PathBuf::from("summary.csv"))
        );
        assert_eq!(
            output_cfg.raw_output_path,
            Some(PathBuf::from("raw.ndjson"))
        );
        assert_eq!(
            output_cfg.sequencing_output_path,
            Some(PathBuf::from("seq.ndjson"))
        );
        assert_eq!(
            output_cfg.mutation_summary_output_path,
            Some(PathBuf::from("muts.csv"))
        );
    }

    #[test]
    fn test_output_config_passes_through_sim_options() {
        let cfg =
            CliConfig::try_parse_from(["steps", "simulate", "-r", "3", "--seed", "7"]).unwrap();
        match cfg.command {
            CliCommand::Simulate(simulate) => {
                assert_eq!(simulate.sim_cfg.replicates, 3);
                assert_eq!(simulate.sim_cfg.seed, Some(7));
            }
            CliCommand::Reproduce(_) => panic!("expected a simulate command"),
        }
    }

    #[test]
    fn test_reproduce_parses_input_path() {
        let cfg = CliConfig::try_parse_from(["steps", "reproduce", "previous.csv"]).unwrap();
        match cfg.command {
            CliCommand::Reproduce(reproduce) => {
                assert_eq!(reproduce.input_path, PathBuf::from("previous.csv"));
            }
            CliCommand::Simulate(_) => panic!("expected a reproduce command"),
        }
    }

    #[test]
    fn test_should_not_track_mutations_without_mutation_outputs() {
        let output_cfg = output_cfg_from(&["steps", "simulate", "--summary-output", "summary.csv"]);
        assert!(!output_cfg.should_track_mutations());
    }

    #[test]
    fn test_should_track_mutations_for_sequencing_output() {
        let output_cfg =
            output_cfg_from(&["steps", "simulate", "--sequencing-output", "seq.ndjson"]);
        assert!(output_cfg.should_track_mutations());
    }

    #[test]
    fn test_should_track_mutations_for_mutation_summary_output() {
        let output_cfg =
            output_cfg_from(&["steps", "simulate", "--mutation-summary-output", "muts.csv"]);
        assert!(output_cfg.should_track_mutations());
    }

    #[test]
    fn test_subcommand_is_required() {
        assert!(CliConfig::try_parse_from(["steps"]).is_err());
    }
}
