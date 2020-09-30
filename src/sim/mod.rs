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

    for i in 0..cfg.transfers {
        let delta_t_phase_1 = Phase1::estimate_delta_t(&lineages, cfg);
        for _ in 0..delta_t_phase_1 {
            lineages = generic_doubling_phase(Phase1(), lineages, cfg, rng);
        }

        let phase_2 = Phase2::new(&lineages, cfg);
        lineages = generic_doubling_phase(
            phase_2,
            lineages,
            cfg,
            rng,
        );

        println!("Generation {}: {:?}", i, lineages);
    }

    println!("End: {:?}", lineages);
}

fn generic_doubling_phase<G: GrowthCalculator, R: Rng>(
    growth_calculator: G,
    lineages: Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) -> Lineages {
    // Create output vector
    // Reserve extra for more mutants
    // The full size won't be needed
    let mut output = Lineages::with_capacity(2 * lineages.len());

    for lineage in lineages.iter() {
        let (new_N, N_mut) =
            growth_calculator.calculate_new_N_and_mutant_count(*lineage, cfg, rng);

        if new_N > 0 {
            output.push(Lineage {
                N: new_N,
                ..*lineage
            });
        }

        for _ in 0..N_mut {
            let mutant = new_mutant(*lineage, cfg, rng);
            output.push(mutant);
        }
    }

    output
}
