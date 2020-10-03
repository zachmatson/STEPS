use std::cell::RefCell;
use std::path::PathBuf;

use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;
use structopt::{clap::AppSettings, StructOpt};

use crate::sim::MutationType;

#[derive(Debug, StructOpt)]
#[structopt(no_version, setting(AppSettings::DisableVersion))]
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
        };

        cfg
    }
}

#[derive(Debug, StructOpt)]
#[structopt(no_version, setting(AppSettings::DisableVersion))]
pub enum Subcommand {
    /// Run simulations
    Simulate(SimConfig),
    /// Convert the simulation output to various formats
    Format(FormatConfig),
}

#[derive(Debug, StructOpt)]
#[structopt(version = " ", setting(AppSettings::DisableVersion))]
pub struct SimConfig {
    #[structopt(short, long = "output", parse(from_os_str))]
    /// File to output results
    pub output_file: PathBuf,

    #[structopt(short, long, default_value = "1")]
    /// Number of replicates to perform
    pub replicates: u32,

    #[structopt(short, long, default_value = "1000")]
    /// How many transfers to run the experiment for
    pub transfers: u32,

    #[structopt(short, long, default_value = "2")]
    /// Number of neutral markers to include in the experiment
    pub markers: u16,

    #[structopt(short = "D", long, default_value = "100")]
    /// The dilution factor
    pub dilution_factor: f64,

    #[structopt(long = "Nmax", default_value = "5E8")]
    /// Maximum population size reached before transfer
    input_max_pop_size: f64,

    #[structopt(short = "f", long, default_value = "1")]
    /// The rate at which populations should be sampled
    pub sampling_frequency: u64,

    #[structopt(short = "l", long)]
    /// The minimum fraction of the population that a mutation must makeup before it will be saved
    pub threshold: Option<f64>,

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
    /// Total mutation rate
    pub total_mutation_rate: f64,

    // Private fields
    #[structopt(skip)]
    /// Distribution from which to pick mutation types
    mutation_type_index_distribution: Option<WeightedIndex<f64>>,
    #[structopt(skip)]
    /// Counter to keep unique identifiers
    unique_id_counter: RefCell<u64>,
}

impl SimConfig {
    /// Available mutation types
    const MUTATION_TYPES: [MutationType; 4] = [
        MutationType::Beneficial,
        MutationType::Neutral,
        MutationType::Deleterious,
        MutationType::MutationRate,
    ];

    /// Finish the initialization of the SimConfig struct with fields that cannot be handled by StructOpt/Clap
    fn finish_initialization(&mut self) {
        // Validate that unimplemented parameters aren't in use
        if self.deleterious_mutation_rate != 0.0 {
            deleterious_todo();
        } else if self.mutation_rate_mutation_rate != 0.0 {
            mutation_rate_todo();
        }

        self.max_pop_size = self.input_max_pop_size.round() as u64;
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

    pub fn get_unique_id(&self) -> u64 {
        self.unique_id_counter.replace_with(|x| *x + 1)
    }
}

#[derive(StructOpt, Debug)]
#[structopt(version = " ", setting(AppSettings::DisableVersion))]
pub struct FormatConfig {/* TODO: Format options */}

pub fn deleterious_todo() -> ! {
    todo!("Deleterious mutations not yet supported")
}

pub fn mutation_rate_todo() -> ! {
    todo!("Mutation rate mutations not yet supported")
}
