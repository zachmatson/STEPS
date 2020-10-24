//! Code for running the simulations and types used for storing simulation data

// Many biological parameters like "N", "W", or "U" will be expressed here with capitalization
// that does not match the normal Rust snake-case guideli
#![allow(non_snake_case)]

use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::cfg::*;

mod types;
pub use types::*;
mod helpers;
use helpers::*;
mod fast_distr;

/// RNG used for the simulations  
/// Implements `Rng` trait from `rand`   
#[allow(non_camel_case_types)]
pub type SIM_RNG = Pcg64;

/// Instantiate RNG to use for the simulations  
/// Uses seed if one is given, otherwise seeds from system entropy  
fn default_sim_rng(cfg: &SimConfig) -> SIM_RNG {
    match cfg.seed {
        Some(seed) => SIM_RNG::seed_from_u64(seed),
        None => SIM_RNG::from_entropy(),
    }
}

/// Opaque handler for populations and transfer processes  
/// Manages transfer details and owns its RNG and a set of lineages
///
/// Must create with the `new` function and call `start_replicate` before each replicate
/// including the first replicate  
/// Then use `transfer` to perform each transfer within a replicate
pub struct SimulationHandler<'a> {
    /// `Lineages` being handled  
    /// Must be created/reset with `new` before a new replicate
    lineages: Option<Lineages>,
    /// Simulation options
    cfg: &'a SimConfig,
    /// RNG to use for all replicates
    rng: SIM_RNG,
}

impl<'a> SimulationHandler<'a> {
    /// Create a new `SimulationHandler` with new RNG
    pub fn new(cfg: &'a SimConfig) -> Self {
        let rng = default_sim_rng(cfg);
        Self {
            lineages: None,
            cfg,
            rng,
        }
    }

    /// Initialize the lineages for a replicate while continuing to use same RNG  
    /// Must call before every replicate
    pub fn start_replicate(&mut self) {
        self.lineages = Some(Lineages::from_simconfig(self.cfg));
    }

    /// Perform a transfer and update the lineages
    pub fn transfer(&mut self) {
        // Get the lineages out of the struct to mutate
        // Will fail for the first transfer of first replicate if `start_replicate` was not called properly
        let mut lineages = self.lineages.take().unwrap();

        // estimate_delta_t gives the *number of times* that phase 1 must be repeated
        let delta_t_phase_1 = Phase1::estimate_delta_t(&lineages, self.cfg);
        for _ in 0..delta_t_phase_1 {
            lineages = Phase1().grow_lineages(lineages, self.cfg, &mut self.rng);
        }

        let phase_2 = Phase2::new(&lineages, self.cfg);
        lineages = phase_2.grow_lineages(lineages, self.cfg, &mut self.rng);

        // Must put lineages back into the struct after transferring
        self.lineages = Some(lineages);
    }

    /// Get reference to the `Lineages` struct owned by the handler  
    /// The lineages will be updated after each transfer  
    pub fn lineages(&self) -> &Lineages {
        self.lineages.as_ref().unwrap()
    }
}
