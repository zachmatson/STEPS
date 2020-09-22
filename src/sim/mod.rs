use rand::prelude::*;
use rand_distr;
use rand_pcg::Pcg64;

use crate::cfg::{self, SimConfig};

mod types;
pub use types::MutationType;
use types::*;
mod helpers;
use helpers::*;

pub fn run_simulations(cfg: &SimConfig) {
    let mut rng = Pcg64::from_entropy();

    for _ in 0..cfg.replicates {
        single_replicate(cfg, &mut rng);
    }
}

fn single_replicate<R: Rng>(cfg: &SimConfig, rng: &mut R) {
    let mut lineages = Lineages::new(cfg);
    println!("Start: {:?}", lineages);

    let delta_t_phase_1 = (cfg.dilution_factor.log2() - 1.0).floor() as usize;
    for i in 0..cfg.transfers {
        lineages = doubling_phase_1(delta_t_phase_1, lineages, cfg, rng);
        lineages = doubling_phase_2(lineages, cfg, rng);

        println!("Generation {}: {:?}", i, lineages);
    }

    println!("End: {:?}", lineages);
}

/// Phase 1 doubling, double delta_t times and don't bottleneck  
/// Create new mutations while doubling where appropriate
fn doubling_phase_1<R: Rng>(
    delta_t: usize,
    mut lineages: Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) -> Lineages {
    // Iterate through each doubling
    for _ in 0..delta_t {
        // Create output vector
        let mut output = Lineages::default();
        // Reserve extra for more mutants
        // The full size won't be needed
        output.reserve(2 * lineages.N.len());

        // Iterate through all populations
        for i in 0..lineages.N.len() {
            // Check number of mutations
            // Sample if mutation rate is positive, otherwise no mutations
            let N_mut: u64 = if lineages.U[i] > 0.0 {
                rand_distr::Poisson::new(lineages.U[i] * lineages.N[i] as f64)
                    .unwrap()
                    .sample(rng)
            } else {
                0
            };
            // Calculate size after growth and subtract mutants
            let new_N = (lineages.W[i].exp2() * lineages.N[i] as f64).round() as u64 - N_mut;

            if new_N < lineages.N[i] {
                eprintln!("WARNING: N_mut exceeded amount of new cells");
            }

            // Make sure new_N is positive
            // i.e. N_mut doesn't exceed size after growth
            // This should always be the case
            if new_N > 0 {
                output.N.push(new_N);
                output.W.push(lineages.W[i]);
                output.U.push(lineages.U[i]);
            }

            // Add the mutants
            for _ in 0..N_mut {
                push_new_mutant(lineages.W[i], lineages.U[i], &mut output, cfg, rng);
            }
        }

        lineages = output;
    }

    lineages
}

fn doubling_phase_2<R: Rng>(lineages: Lineages, cfg: &SimConfig, rng: &mut R) -> Lineages {
    // Create output vector
    let mut output = Lineages::default();
    output.reserve(lineages.N.len());

    // Calculate delta_t
    let delta_t = estimate_phase_2_delta_t(&lineages, cfg);

    for i in 0..lineages.N.len() {
        let N_bottlenecked = calculate_bottlenecked_size_with_growth(
            delta_t,
            lineages.N[i],
            lineages.W[i],
            cfg,
            rng,
        );

        if N_bottlenecked == 0 {
            continue;
        }

        let N_mut = if lineages.U[i] > 0.0 {
            rand_distr::Poisson::new(
                lineages.U[i]
                    * N_bottlenecked as f64
                    * (1.0 - (lineages.W[i] * delta_t).exp2().recip()),
            )
            .unwrap()
            .sample(rng)
        } else {
            0
        };
        let new_N = N_bottlenecked - N_mut;

        if new_N > 0 {
            output.N.push(new_N);
            output.W.push(lineages.W[i]);
            output.U.push(lineages.U[i]);
        }

        for _ in 0..N_mut {
            push_new_mutant(lineages.W[i], lineages.U[i], &mut output, cfg, rng);
        }
    }

    output
}
