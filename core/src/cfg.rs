//! Configuration options for the simulations and output, with CLI parsing traits derived

// Many biological parameters like "N", "W", or "U" will be expressed here with capitalization
// that does not match the normal Rust snake-case guidelines
#![allow(non_snake_case)]

use clap::{AppSettings, Parser};
use serde::{Deserialize, Serialize};

/// Options for summary output statistics
#[derive(Clone, Parser)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct SummaryOutputConfig {
    /// Output weighted arithmetic mean of lineage fitnesses
    #[clap(skip = true)]
    pub avg_W: bool,
    /// Output the ratio of marker 1 to other markers
    #[clap(long)]
    pub marker_1_ratio: bool,
    /// Output weighted standard deviation of lineage fitnesses
    #[clap(long)]
    pub stdev_W: bool,
    /// Output maximum lineage fitness
    #[clap(long)]
    pub max_W: bool,
    /// Output the standard deviation of the number of mutations accumulated since the ancestor
    #[clap(long)]
    pub stdev_accumulated_muts: bool,
    /// Output the maximum number of mutations accumulated since the ancestor
    #[clap(long)]
    pub max_accumulated_muts: bool,
    /// Output the mean number of mutations accumulated since the ancestor
    #[clap(skip = true)]
    pub mean_accumulated_muts: bool,
    /// Output the minimum number of mutations accumulated since the ancestor
    #[clap(long)]
    pub min_accumulated_muts: bool,
    /// Output the number of genotypes present in the population
    #[clap(long)]
    pub genotype_count: bool,
    /// Output the Shannon diversity of genotypes in the population
    #[clap(long)]
    pub shannon_diversity: bool,
}

/// Options for STEPS simulations
#[derive(Clone, Parser, Serialize, Deserialize)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct SimConfig {
    /// Number of replicates to perform
    #[clap(short, long, default_value = "12")]
    pub replicates: u32,
    /// Number of transfers to run the experiment for in each replicate
    #[clap(short, long, default_value = "300")]
    pub transfers: u32,
    /// Number of neutral markers to include in the experiment
    #[clap(short, long, default_value = "1")]
    pub markers: u16,
    /// The dilution factor
    #[clap(short = 'D', long, default_value = "100")]
    pub dilution_factor: f64,
    /// Beneficial mutation rate
    #[clap(long = "Ub", default_value = "1.7E-6")]
    pub beneficial_mutation_rate: f64,
    /// Neutral mutation rate
    #[clap(long = "Un", default_value = "0.0")]
    pub neutral_mutation_rate: f64,
    /// Deleterious mutation rate
    #[clap(long = "Ud", default_value = "0.0")]
    pub deleterious_mutation_rate: f64,
    /// Initial mean beneficial mutation size
    #[clap(long = "Sb", default_value = "0.012")]
    pub initial_beneficial_mutation_size: f64,
    /// Fixed deleterious mutation size
    #[clap(long = "Sd")]
    pub fixed_deleterious_mutation_size: Option<f64>,
    /// Diminishing returns epistasis strength
    #[clap(short = 'g', default_value = "6.0")]
    pub diminishing_returns_epistasis_strength: f64,
    /// Seed for the RNG
    #[clap(long)]
    pub seed: Option<u64>,
    /// Maximum population size reached before bottleneck
    #[clap(long = "Nmax", default_value = "5E8")]
    pub max_pop_size: f64,
}
