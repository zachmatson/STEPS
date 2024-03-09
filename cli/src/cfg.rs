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
