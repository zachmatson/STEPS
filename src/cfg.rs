//! Structs with configuration settings and parameters for simulations, outputs
//! and subcommands
//!
//! Also contains all code for handling the receiving of command line input

// Many biological parameters like "N", "W", or "U" will be expressed here with capitalization
// that does not match the normal Rust snake-case guidelines
#![allow(non_snake_case)]

use std::path::PathBuf;

use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;
use serde::{Deserialize, Serialize};
use structopt::{clap, StructOpt};

use crate::sim::MutationType;

/// Configuration options for STEPS command line app  
#[derive(StructOpt)]
#[structopt(about = "Serially Transferred Evolving Population Simulator")]
pub struct Config {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

impl Config {
    /// Generate `Config` from command line arguments  
    /// Exits the program if there is a failure  
    pub fn from_args() -> Self {
        // Underlying StructOpt/clap implementation performs the actual building of `Config`
        let mut cfg = <Config as StructOpt>::from_args();
        // Additional initialization where needed
        match &mut cfg.subcommand {
            Subcommand::Simulate(sim_cfg) => sim_cfg.finish_initialization(),
            Subcommand::Reproduce(_) => (),
        };

        cfg
    }
}

/// Subcommand definitions
#[derive(StructOpt)]
#[structopt(about = "Serially Transferred Evolving Population Simulator", setting = clap::AppSettings::DeriveDisplayOrder)]
pub enum Subcommand {
    /// Run simulations
    Simulate(SimulationsCLIConfig),
    /// Reproduce results from a previous simulation run  
    Reproduce(ReproduceConfig),
}

/// Run the STEPS simulation
#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub struct SimulationsCLIConfig {
    #[structopt(flatten)]
    pub output_cfg: CLIOutputConfig,

    #[structopt(flatten)]
    pub sim_cfg: SimConfig,
}

impl SimulationsCLIConfig {
    /// Finish the initialization work that cannot be handled by StructOpt/Clap
    fn finish_initialization(&mut self) {
        self.sim_cfg.finish_initialization();
    }
}

/// Reproduce results of a previous run of the STEPS simulation
#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub struct ReproduceConfig {
    /// Path of the input file, which came from a previous run  
    /// and contains the information needed to reproduce the results
    pub input_path: PathBuf,

    #[structopt(flatten)]
    pub output_cfg: CLIOutputConfig,
}

/// Command line inputs needed to output results
#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub struct CLIOutputConfig {
    /// The rate at which populations should be sampled
    #[structopt(short = "f", long, default_value = "1")]
    pub sampling_frequency: u32,

    /// Path to output the summarized simulation results (as CSV),
    /// which contains the fitness and marker ratio (if applicable) over time
    #[structopt(short = "o", long = "summary-output")]
    pub summary_output_path: Option<PathBuf>,

    /// Path to output the full raw simulation results (as ndjson),
    /// which includes data for all mutations at each sampled interval
    #[structopt(short = "j", long = "raw-output")]
    pub raw_output_path: Option<PathBuf>,

    /// Path to output information about all mutations that occur (as ndjson),
    /// which includes change in fitness and IDs for all mutations over time
    #[structopt(short, long = "sequencing-output")]
    pub sequencing_output_path: Option<PathBuf>,

    #[structopt(flatten)]
    pub summary_cfg: SummaryOutputConfig,
}

impl CLIOutputConfig {
    /// Should sequencing information be output?
    pub fn is_sequencing_enabled(&self) -> bool {
        self.sequencing_output_path.is_some()
    }
}

/// Options for summary output statistics
#[derive(Clone, StructOpt)]
pub struct SummaryOutputConfig {
    /// Output the ratio of marker 1 to other markers
    #[structopt(long)]
    pub marker_1_ratio: bool,
    /// Output weighted standard deviation of lineage fitnessees
    #[structopt(long)]
    pub stdev_W: bool,
    /// Output maximum lineage fitness
    #[structopt(long)]
    pub max_W: bool,
    /// Output the standard deviation of the number of mutations accumulated since the ancestor
    #[structopt(long)]
    pub stdev_accumulated_muts: bool,
    /// Output the maximum number of mutations accumulated since the ancestor
    #[structopt(long)]
    pub max_accumulated_muts: bool,
    /// Output the number of genotypes present in the population
    #[structopt(long)]
    pub genotype_count: bool,
    /// Output the Shannon diversity of genotypes in the population
    #[structopt(long)]
    pub shannon_diversity: bool,
}

/// Options for STEPS simulations
#[derive(StructOpt, Serialize, Deserialize, Clone)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub struct SimConfig {
    /// Number of replicates to perform
    #[structopt(short, long, default_value = "1")]
    pub replicates: u32,

    /// How many transfers to run the experiment for in each replicate
    #[structopt(short, long, default_value = "1000")]
    pub transfers: u32,

    /// Number of neutral markers to include in the experiment
    #[structopt(short, long, default_value = "2")]
    pub markers: u16,

    /// The dilution factor
    #[structopt(short = "D", long, default_value = "100")]
    pub dilution_factor: f64,

    /// Beneficial mutation rate
    #[structopt(long = "Ub", default_value = "0.0")]
    pub beneficial_mutation_rate: f64,

    /// Neutral mutation rate
    #[structopt(long = "Un", default_value = "0.0")]
    pub neutral_mutation_rate: f64,

    /// Deleterious mutation rate
    #[structopt(long = "Ud", default_value = "0.0")]
    pub deleterious_mutation_rate: f64,

    /// The mutation rate of the mutation rate
    #[structopt(long = "Um", default_value = "0.0")]
    pub mutation_rate_mutation_rate: f64,

    /// Initial mean beneficial mutation size
    #[structopt(long = "Sb", default_value = "0.015873")]
    pub initial_beneficial_mutation_size: f64,

    /// Deleterious mutation size as multiple of beneficial mutation size
    #[structopt(long = "Sd", default_value = "0.0")]
    pub deleterious_mutation_size_factor: f64,

    /// Mutation rate mutation size as multiple of beneficial mutation size
    #[structopt(long = "Sm", default_value = "0.0")]
    pub mutation_rate_mutation_size_factor: f64,

    /// Diminishing returns epistasis strength
    #[structopt(short = "g", default_value = "1.0")]
    pub diminishing_returns_epistasis_strength: f64,

    /// Seed for the RNG
    #[structopt(long)]
    pub seed: Option<u64>,

    /// Maximum population size reached before transfer
    #[structopt(long = "Nmax", default_value = "5E8")]
    pub max_pop_size: f64,

    //
    // Must be calculated after other fields are known
    //
    /// Total mutation rate
    #[structopt(skip)]
    #[serde(skip)]
    pub total_mutation_rate: f64,
    /// Reciprocal of dilution factor
    #[structopt(skip)]
    #[serde(skip)]
    pub dilution_coefficient: f64,

    //
    // Private fields
    //
    /// Distribution from which to pick mutation types
    #[structopt(skip)]
    #[serde(skip)]
    mutation_type_index_distribution: Option<WeightedIndex<f64>>,
}

impl SimConfig {
    /// Available mutation types
    const MUTATION_TYPES: [MutationType; 4] = [
        MutationType::Beneficial,
        MutationType::Neutral,
        MutationType::Deleterious,
        MutationType::MutationRate,
    ];

    /// Finish initialization for fields that require additional steps
    pub fn finish_initialization(&mut self) {
        self.total_mutation_rate = self.beneficial_mutation_rate
            + self.deleterious_mutation_rate
            + self.neutral_mutation_rate
            + self.mutation_rate_mutation_rate;

        self.dilution_coefficient = self.dilution_factor.recip();

        // Weights for the elements of Self::MUTATION_TYPES
        self.mutation_type_index_distribution = if self.total_mutation_rate > 0.0 {
            Some(
                WeightedIndex::new(vec![
                    self.beneficial_mutation_rate,
                    self.neutral_mutation_rate,
                    self.deleterious_mutation_rate,
                    self.mutation_rate_mutation_rate,
                ])
                .unwrap(),
            )
        } else {
            None
        }
    }

    /// Randomly pick a mutation type weighted by the mutation rates selected  
    /// Will return None iff all mutation rates are 0
    pub fn sample_mutation_type<R: Rng>(&self, rng: &mut R) -> Option<MutationType> {
        self.mutation_type_index_distribution
            .as_ref()
            .map(|dist| Self::MUTATION_TYPES[dist.sample(rng)])
    }
}

/// Call where code to support deleterious mutations is activated  
/// these code paths should not be accessed in normal use until
/// support for deleterious mutations is added
pub fn deleterious_todo() -> ! {
    todo!("Deleterious mutations not yet supported")
}

/// Call where code to support mutation rate mutations is activated  
/// these code paths should not be accessed in normal use until
/// support for mutation rate mutations is added
pub fn mutation_rate_todo() -> ! {
    todo!("Mutation rate mutations not yet supported")
}
