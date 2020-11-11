//! Code for running the simulations and types used for storing simulation data

// Many biological parameters like "N", "W", or "U" will be expressed here with capitalization
// that does not match the normal Rust snake-case guidelines
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
pub struct SimulationHandler {
    /// Simulation options
    cfg: SimConfig,
    /// Number of phase 1 doublings to perform
    phase_1_doublings: usize,
    /// `Lineages` being handled  
    /// Must be created/reset with `new` before a new replicate
    lineages: Option<Lineages>,
    /// Mutations added in the last transfer
    new_mutations: Option<Vec<Mutation>>,
    /// RNG to use for all replicates
    rng: SIM_RNG,
}

impl SimulationHandler {
    /// Create a new `SimulationHandler` with new RNG
    pub fn new(cfg: SimConfig, track_mutations: bool) -> Self {
        let phase_1_doublings = Phase1::doublings_required(&cfg);

        let rng = default_sim_rng(&cfg);

        let new_mutations = match track_mutations {
            true => Some(Vec::new()),
            false => None,
        };

        Self {
            cfg,
            phase_1_doublings,
            lineages: None,
            new_mutations,
            rng,
        }
    }

    /// Initialize the lineages for a replicate while continuing to use same RNG  
    /// Must call before every replicate
    pub fn start_replicate(&mut self) {
        self.reset_new_mutations();
        self.lineages = Some(Lineages::from_simconfig(&self.cfg, &mut self.new_mutations));
    }

    /// Perform a transfer and update the lineages
    pub fn transfer(&mut self) {
        self.reset_new_mutations();

        // Get the lineages out of the struct to mutate
        // Will fail for the first transfer of first replicate if `start_replicate` was not called properly
        let mut lineages = self.lineages.take().unwrap();

        // estimate_delta_t gives the *number of times* that phase 1 must be repeated
        for _ in 0..self.phase_1_doublings {
            let phase_1 = Phase1::new(&lineages);
            lineages =
                phase_1.grow_lineages(lineages, &self.cfg, &mut self.rng, &mut self.new_mutations);
        }

        let phase_2 = Phase2::new(&lineages, &self.cfg);
        lineages =
            phase_2.grow_lineages(lineages, &self.cfg, &mut self.rng, &mut self.new_mutations);

        // Must put lineages back into the struct after transferring
        self.lineages = Some(lineages);
    }

    /// Get reference to the `Lineages` struct owned by the handler  
    /// The lineages will be updated after each transfer  
    pub fn lineages(&self) -> &Lineages {
        self.lineages.as_ref().unwrap()
    }

    /// Get reference to the new mutations added in the last transfer  
    /// Will be `None` if sequencing mode is not enabled (`track_mutations` set to `false`)
    pub fn new_mutations(&self) -> Option<&Vec<Mutation>> {
        self.new_mutations.as_ref()
    }

    fn reset_new_mutations(&mut self) {
        if let Some(new_mutations) = &mut self.new_mutations {
            new_mutations.clear();
        }
    }
}
