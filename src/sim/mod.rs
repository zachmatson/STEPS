use rand::prelude::*;
use rand_distr;
use rand_pcg::Pcg64;

use crate::cfg::*;

mod types;
pub use types::MutationType;
use types::*;
mod helpers;
use helpers::*;
mod fast_distr;

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
            lineages = Phase1().double_lineages(lineages, cfg, rng);
        }

        let phase_2 = Phase2::new(&lineages, cfg);
        lineages = phase_2.double_lineages(lineages, cfg, rng);

        println!("Generation {}: {:?}", i, lineages);
    }

    println!("End: {:?}", lineages);
}
