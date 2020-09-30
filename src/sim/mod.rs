use rand::prelude::*;
use rand_distr;
use rand_pcg::Pcg64;

use crate::cfg::*;

mod types;
pub use types::MutationType;
use types::*;
mod helpers;
use helpers::*;

pub fn run_simulations(cfg: &SimConfig) {
    let mut rng = match cfg.seed {
        Some(seed) => Pcg64::seed_from_u64(seed),
        None => Pcg64::from_entropy(),
    };

    for _ in 0..cfg.replicates {
        single_replicate(cfg, &mut rng);
    }
}

fn single_replicate<R: Rng>(cfg: &SimConfig, rng: &mut R) {
    let mut lineages = Lineages::from_simconfig(cfg);
    println!("Start: {:?}", lineages);

    let mut delta_t_phase_1 = (cfg.dilution_factor.log2() - 1.0).floor() as usize;

    for i in 0..cfg.transfers {
        let mut delta_t_phase_2 = 0.0;
        for i in 0..delta_t_phase_1 {
            let (updated_lineages, sum_N, avg_W) =
                generic_doubling_phase(Phase1(), lineages, cfg, rng);
            lineages = updated_lineages;
            if i == delta_t_phase_1 - 1 {
                delta_t_phase_2 = estimate_phase_2_delta_t(sum_N, avg_W, cfg);
            }
        }

        let (updated_lineages, sum_N, avg_W) = generic_doubling_phase(
            Phase2 {
                delta_t: delta_t_phase_2,
            },
            lineages,
            cfg,
            rng,
        );
        lineages = updated_lineages;
        delta_t_phase_1 = estimate_phase_1_delta_t(sum_N, avg_W, cfg);

        println!("Generation {}: {:?}", i, lineages);
    }

    println!("End: {:?}", lineages);
}

fn generic_doubling_phase<G: GrowthCalculator, R: Rng>(
    growth_calculator: G,
    lineages: Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) -> (Lineages, u64, f64) {
    // Create output vector
    // Reserve extra for more mutants
    // The full size won't be needed
    let mut output = Lineages::with_capacity(2 * lineages.N.len());

    let mut sum_N = 0;
    let mut weighted_sum_W = 0.0;

    for i in 0..lineages.N.len() {
        let (new_N, N_mut) =
            growth_calculator.calculate_new_N_and_mutant_count(&lineages, i, cfg, rng);

        if new_N > 0 {
            output.N.push(new_N);
            output.W.push(lineages.W[i]);
            output.U.push(lineages.U[i]);
            sum_N += new_N;
            weighted_sum_W += new_N as f64 * lineages.W[i];
        }

        for _ in 0..N_mut {
            push_new_mutant(lineages.W[i], lineages.U[i], &mut output, cfg, rng);
            sum_N += 1;
            weighted_sum_W += output.W.last().unwrap();
        }
    }

    (output, sum_N, weighted_sum_W / sum_N as f64)
}
