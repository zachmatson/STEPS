use std::path::PathBuf;

use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;
use serde::{Deserialize, Serialize};
use structopt::{
    clap,
    StructOpt,
};

use crate::sim::MutationType;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

impl Config {
    /// Generate Config from command line arguments
    pub fn from_args() -> Self {
        let mut cfg = <Config as StructOpt>::from_args();
        match &mut cfg.subcommand {
            Subcommand::Simulate(sim_cfg) => sim_cfg.finish_initialization(),
            Subcommand::Format(_) => (),
            Subcommand::Reproduce(_) => (),
        };

        cfg
    }
}

#[derive(StructOpt)]
pub enum Subcommand {
    /// Run simulations
    Simulate(SimulationsCLIConfig),
    /// Convert the simulation output to various formats
    Format(FormatConfig),
    /// Reproduce results from a previous simulation run
    Reproduce(ReproduceConfig),
}

#[derive(StructOpt)]
pub struct SimulationsCLIConfig {
    #[structopt(flatten)]
    pub output_cfg: OuputConfig,

    #[structopt(flatten)]
    pub sim_cfg: SimConfig,

    #[structopt(long = "Nmax", default_value = "5E8")]
    /// Maximum population size reached before transfer
    input_max_pop_size: f64,
}

impl SimulationsCLIConfig {
    /// Finish the initialization of the SimConfig struct with fields that cannot be handled by StructOpt/Clap
    fn finish_initialization(&mut self) {
        // Might have rounding issues if the input gets this large
        // and population size should never be this large
        // so handle with a panic
        // In the future this might need to be replaced with something more robust,
        // but for now it is best to prevent any subtle errors
        if self.input_max_pop_size >= (1u64 << 53u64) as f64 {
            clap::Error::with_description(
                "Max pop size exceeds maximum 2^53-1. Input may be rounded incorrectly with this population size.",
                clap::ErrorKind::InvalidValue
            ).exit();
        }
        self.sim_cfg.max_pop_size = self.input_max_pop_size.round() as u64;
        self.sim_cfg.finish_initialization();
    }
}

#[derive(StructOpt)]
pub struct FormatConfig {/* TODO: Format options */}

#[derive(StructOpt)]
pub struct ReproduceConfig {
    /// Path of the input file, which came from a previous run  
    /// and contains the information needed to reproduce the results
    pub input_path: PathBuf,

    #[structopt(flatten)]
    pub output_cfg: OuputConfig,
}

#[derive(StructOpt)]
pub struct OuputConfig {
    #[structopt(short = "j", long = "raw-output")]
    /// Path to output the full raw simulation results (as ndjson),
    /// which includes data for all mutations at each sampled interval
    pub raw_output_path: Option<PathBuf>,

    #[structopt(short = "o", long = "summary-output")]
    /// Path to output the summarized simulation results (as CSV),
    /// which contains the fitness and marker ratio over time
    pub summary_output_path: Option<PathBuf>,
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct SimConfig {
    #[structopt(short, long, default_value = "1")]
    /// Number of replicates to perform
    pub replicates: u32,

    #[structopt(short, long, default_value = "1000")]
    /// How many transfers to run the experiment for
    pub transfers: u32,

    #[structopt(short = "f", long, default_value = "1")]
    /// The rate at which populations should be sampled
    pub sampling_frequency: u32,

    #[structopt(short, long, default_value = "2")]
    /// Number of neutral markers to include in the experiment
    pub markers: u16,

    #[structopt(short = "D", long, default_value = "100")]
    /// The dilution factor
    pub dilution_factor: f64,

    #[structopt(long = "Ub", default_value = "0.0")]
    /// Beneficial mutation rate
    pub beneficial_mutation_rate: f64,

    #[structopt(long = "Un", default_value = "0.0")]
    /// Neutral mutation rate
    pub neutral_mutation_rate: f64,

    #[structopt(long = "Ud", default_value = "0.0")]
    /// Deleterious mutation rate
    pub deleterious_mutation_rate: f64,

    #[structopt(long = "Um", default_value = "0.0")]
    /// The mutation rate of the mutation rate
    pub mutation_rate_mutation_rate: f64,

    #[structopt(long = "Sb", default_value = "0.011")]
    /// Initial mean beneficial mutation size
    pub initial_beneficial_mutation_size: f64,

    #[structopt(long = "Sd", default_value = "0.0")]
    /// Factor describing the deleterious mutation rate size
    pub deleterious_mutation_size_factor: f64,

    #[structopt(long = "Sm", default_value = "0.0")]
    /// Factor describing mutation rate mutation size
    pub mutation_rate_mutation_size_factor: f64,

    #[structopt(short = "g", default_value = "1.0")]
    /// Diminishing returns epistasis strength
    pub diminishing_returns_epistasis_strength: f64,

    #[structopt(long)]
    /// Seed for the RNG
    pub seed: Option<u64>,

    // Calculated fields
    #[structopt(skip)]
    /// Maximum population size reached before transfer
    pub max_pop_size: u64,
    #[structopt(skip)]
    #[serde(skip_deserializing)]
    /// Total mutation rate
    pub total_mutation_rate: f64,

    // Private fields
    #[structopt(skip)]
    #[serde(skip)]
    /// Distribution from which to pick mutation types
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
        // Validate that unimplemented parameters aren't in use
        if self.mutation_rate_mutation_rate != 0.0 {
            mutation_rate_todo();
        }

        self.total_mutation_rate = self.beneficial_mutation_rate
            + self.deleterious_mutation_rate
            + self.neutral_mutation_rate
            + self.mutation_rate_mutation_rate;

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

    /// Randomly pick a mutation type weighted by the mutation rates
    /// selected  
    /// Will return None if all mutation rates are 0
    pub fn sample_mutation_type<R: Rng>(&self, rng: &mut R) -> Option<MutationType> {
        self.mutation_type_index_distribution
            .as_ref()
            .map(|dist| Self::MUTATION_TYPES[dist.sample(rng)])
    }
}

pub fn deleterious_todo() -> ! {
    todo!("Deleterious mutations not yet supported")
}

pub fn mutation_rate_todo() -> ! {
    todo!("Mutation rate mutations not yet supported")
}
