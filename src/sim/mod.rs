use rand::prelude::*;
use rand_distr;
use rand_pcg::Pcg64;

use crate::cfg::*;

mod types;
pub use types::*;
mod helpers;
use helpers::*;
mod fast_distr;

/// RNG Type to use for the simulations  
/// Will impelement rand trait from the rand crate  
/// Any other RNG implementing the trait can be used instead
/// but this is the recommended RNG used in the default impelementation
#[allow(non_camel_case_types)]
pub type SIM_RNG = Pcg64;

fn default_sim_rng(cfg: &SimConfig) -> SIM_RNG {
    match cfg.seed {
        Some(seed) => SIM_RNG::seed_from_u64(seed),
        None => SIM_RNG::from_entropy(),
    }
}

/// Owns a set of lineages and transfers them as requested
pub struct PopulationHandler<'a> {
    lineages: Option<Lineages>,
    cfg: &'a SimConfig,
    rng: SIM_RNG,
}

impl<'a> PopulationHandler<'a> {
    pub fn new(cfg: &'a SimConfig) -> Self {
        let rng = default_sim_rng(cfg);
        Self {
            lineages: None,
            cfg,
            rng,
        }
    }

    pub fn start_replicate(&mut self) {
        self.lineages = Some(Lineages::from_simconfig(self.cfg));
    }

    pub fn transfer(&mut self) {
        let mut lineages = self.lineages.take().unwrap();

        let delta_t_phase_1 = Phase1::estimate_delta_t(&lineages, self.cfg);
        for _ in 0..delta_t_phase_1 {
            lineages = Phase1().double_lineages(lineages, self.cfg, &mut self.rng);
        }

        let phase_2 = Phase2::new(&lineages, self.cfg);
        lineages = phase_2.double_lineages(lineages, self.cfg, &mut self.rng);

        self.lineages = Some(lineages);
    }

    pub fn lineages(&self) -> &Lineages {
        self.lineages.as_ref().unwrap()
    }
}
