//! Ratio between two neutral markers over the course of a run.
//!
//! ```text
//! cargo run --release --example marker_ratio
//! ```

use steps_core::cfg::SimConfig;
use steps_core::sim::{summarize, SimulationHandler, SimulationState};

fn main() {
    let cfg = SimConfig {
        replicates: 6,
        transfers: 300,
        markers: 2,
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

    println!("replicate,transfer,log2_marker_1_ratio,genotype_count");

    let mut handler = SimulationHandler::new(cfg, false);

    while let Some(SimulationState {
        replicate,
        transfer,
        lineages,
        ..
    }) = handler.next_state()
    {
        let log2_ratio = summarize::marker_1_ratio(lineages).log2();

        // The log is undefined once either marked lineage is lost, so stop reporting that
        // replicate rather than emitting an infinity
        if !log2_ratio.is_finite() {
            continue;
        }

        println!(
            "{},{},{:.6},{}",
            replicate,
            transfer,
            log2_ratio,
            summarize::genotype_count(lineages),
        );
    }
}
