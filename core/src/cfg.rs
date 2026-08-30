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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[clap(flatten)]
        sim: SimConfig,
    }

    #[test]
    fn test_sim_config_defaults() {
        let cli = TestCli::parse_from(["test"]);
        assert_eq!(cli.sim.replicates, 12);
        assert_eq!(cli.sim.transfers, 300);
        assert_eq!(cli.sim.markers, 1);
        assert_eq!(cli.sim.dilution_factor, 100.0);
        assert_eq!(cli.sim.beneficial_mutation_rate, 1.7e-6);
        assert_eq!(cli.sim.neutral_mutation_rate, 0.0);
        assert_eq!(cli.sim.deleterious_mutation_rate, 0.0);
        assert_eq!(cli.sim.initial_beneficial_mutation_size, 0.012);
        assert_eq!(cli.sim.fixed_deleterious_mutation_size, None);
        assert_eq!(cli.sim.diminishing_returns_epistasis_strength, 6.0);
        assert_eq!(cli.sim.seed, None);
        assert_eq!(cli.sim.max_pop_size, 5e8);
    }

    #[test]
    fn test_sim_config_custom_args() {
        let cli = TestCli::parse_from([
            "test", "-r", "5", "-t", "50", "-m", "3", "-D", "200", "--Ub", "0.001", "--seed", "123",
        ]);
        assert_eq!(cli.sim.replicates, 5);
        assert_eq!(cli.sim.transfers, 50);
        assert_eq!(cli.sim.markers, 3);
        assert_eq!(cli.sim.dilution_factor, 200.0);
        assert_eq!(cli.sim.beneficial_mutation_rate, 0.001);
        assert_eq!(cli.sim.seed, Some(123));
    }

    #[test]
    fn test_sim_config_serialize_deserialize() {
        let cfg = SimConfig {
            replicates: 10,
            transfers: 200,
            markers: 2,
            dilution_factor: 50.0,
            beneficial_mutation_rate: 0.01,
            neutral_mutation_rate: 0.005,
            deleterious_mutation_rate: 0.002,
            initial_beneficial_mutation_size: 0.05,
            fixed_deleterious_mutation_size: Some(0.1),
            diminishing_returns_epistasis_strength: 3.0,
            seed: Some(999),
            max_pop_size: 1e9,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: SimConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.replicates, 10);
        assert_eq!(deserialized.transfers, 200);
        assert_eq!(deserialized.markers, 2);
        assert_eq!(deserialized.dilution_factor, 50.0);
        assert_eq!(deserialized.beneficial_mutation_rate, 0.01);
        assert_eq!(deserialized.neutral_mutation_rate, 0.005);
        assert_eq!(deserialized.deleterious_mutation_rate, 0.002);
        assert_eq!(deserialized.initial_beneficial_mutation_size, 0.05);
        assert_eq!(deserialized.fixed_deleterious_mutation_size, Some(0.1));
        assert_eq!(deserialized.diminishing_returns_epistasis_strength, 3.0);
        assert_eq!(deserialized.seed, Some(999));
        assert_eq!(deserialized.max_pop_size, 1e9);
    }

    #[test]
    fn test_sim_config_no_seed_serialization() {
        let cfg = SimConfig {
            replicates: 1,
            transfers: 1,
            markers: 1,
            dilution_factor: 2.0,
            beneficial_mutation_rate: 0.0,
            neutral_mutation_rate: 0.0,
            deleterious_mutation_rate: 0.0,
            initial_beneficial_mutation_size: 0.01,
            fixed_deleterious_mutation_size: None,
            diminishing_returns_epistasis_strength: 1.0,
            seed: None,
            max_pop_size: 1000.0,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: SimConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.seed, None);
        assert_eq!(deserialized.fixed_deleterious_mutation_size, None);
    }
}
