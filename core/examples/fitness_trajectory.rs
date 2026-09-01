//! Average fitness of each replicate population after every transfer.
//!
//! ```text
//! cargo run --release --example fitness_trajectory
//! ```

use steps_core::cfg::SimConfig;
use steps_core::sim::{summarize, SimulationHandler, SimulationState};

fn main() {
    let cfg = SimConfig {
        replicates: 12,
        transfers: 300,
        markers: 1,
        dilution_factor: 100.0,
        beneficial_mutation_rate: 1.7e-6,
        neutral_mutation_rate: 0.0,
        deleterious_mutation_rate: 0.0,
        initial_beneficial_mutation_size: 0.012,
        fixed_deleterious_mutation_size: None,
        diminishing_returns_epistasis_strength: 6.0,
        seed: Some(606),
        max_pop_size: 5e8,
    };

    let generations_per_transfer = cfg.dilution_factor.log2();

    println!("replicate,transfer,generation,avg_W");

    let mut handler = SimulationHandler::new(cfg, false);

    while let Some(SimulationState {
        replicate,
        transfer,
        lineages,
        ..
    }) = handler.next_state()
    {
        println!(
            "{},{},{:.1},{:.6}",
            replicate,
            transfer,
            transfer as f64 * generations_per_transfer,
            summarize::avg_W(lineages),
        );
    }
}
