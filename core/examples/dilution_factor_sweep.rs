//! Fitness reached under different dilution factors, holding generations roughly constant.
//!
//! ```text
//! cargo run --release --example dilution_factor_sweep
//! ```

use steps_core::cfg::SimConfig;
use steps_core::sim::{summarize, SimulationHandler};

const DILUTION_FACTORS: [f64; 4] = [2.0, 8.0, 100.0, 1000.0];

const TARGET_GENERATIONS: f64 = 2000.0;

fn main() {
    println!("dilution_factor,transfers,mean_final_fitness");

    for dilution_factor in DILUTION_FACTORS {
        let transfers = (TARGET_GENERATIONS / dilution_factor.log2()).round() as u32;

        let cfg = SimConfig {
            replicates: 12,
            transfers,
            markers: 1,
            dilution_factor,
            beneficial_mutation_rate: 1.7e-6,
            neutral_mutation_rate: 0.0,
            deleterious_mutation_rate: 0.0,
            initial_beneficial_mutation_size: 0.012,
            fixed_deleterious_mutation_size: None,
            diminishing_returns_epistasis_strength: 6.0,
            seed: Some(606),
            max_pop_size: 5e8,
        };

        let mut handler = SimulationHandler::new(cfg, false);
        let mut final_fitness = Vec::new();

        while let Some(state) = handler.next_state() {
            if state.end_of_replicate {
                final_fitness.push(summarize::avg_W(state.lineages));
            }
        }

        let mean = final_fitness.iter().sum::<f64>() / final_fitness.len() as f64;
        println!("{},{},{:.6}", dilution_factor, transfers, mean);
    }
}
