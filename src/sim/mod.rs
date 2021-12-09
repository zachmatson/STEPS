//! Code for running the simulations and types used for storing simulation data

// Many biological parameters like "N", "W", or "U" will be expressed here with capitalization
// that does not match the normal Rust snake-case guidelines
#![allow(non_snake_case)]

use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::cfg::*;

mod types;
pub use types::*;
mod kernels;
use kernels::*;
pub use kernels::{marker_1_ratio_and_avg_W, sum_N_and_avg_W};
mod mechanics;
use mechanics::*;
mod distr;
mod sequencing;

/// RNG used for the simulations  
/// Implements `Rng` trait from `rand`   
type SimRng = Pcg64;

/// Instantiate RNG to use for the simulations  
/// Uses seed if one is given, otherwise seeds from system entropy  
fn default_sim_rng(cfg: &SimConfig) -> SimRng {
    match cfg.seed {
        Some(seed) => SimRng::seed_from_u64(seed),
        None => SimRng::from_entropy(),
    }
}

/// Opaque handler for populations and transfer processes  
/// Manages transfer details and owns its RNG and a set of lineages
///
/// **Must** create with the `new` function and call `start_replicate` before each replicate
/// including the first replicate  
/// Then use `transfer` to perform each transfer within a replicate
pub struct SimulationHandler {
    /// Simulation options
    cfg: SimConfig,
    /// Number of phase 1 doublings to perform
    phase_1_doublings: usize,
    /// Lineages in the simulation  
    /// Must be created/reset before a new replicate
    lineages: LineagesData,
    /// Mutations data for sequencing
    ///
    /// Owner of SimulationHandler is responsible for
    /// clearing or not clearing pruned mutations as they see fit,
    /// the `SimulationHandler` doesn't care about them
    mutations: Option<MutationsData>,
    /// RNG to use for all replicates
    rng: SimRng,
}

impl SimulationHandler {
    /// Create a new `SimulationHandler` with new RNG
    pub fn new(cfg: SimConfig, track_mutations: bool) -> Self {
        let phase_1_doublings = phase_1_doublings_required(&cfg);
        let rng = default_sim_rng(&cfg);
        let mutations = match track_mutations {
            true => Some(MutationsData::default()),
            false => None,
        };

        Self {
            cfg,
            phase_1_doublings,
            lineages: LineagesData::default(),
            mutations,
            rng,
        }
    }

    /// Initialize the lineages for a replicate while continuing to use same RNG  
    /// Must call this before every replicate
    pub fn start_replicate(&mut self) {
        self.mutations = self.mutations.as_ref().map(|_| MutationsData::new());
        self.lineages = LineagesData::from_simconfig(&self.cfg, &mut self.mutations);

        // We need the initial sequencing information from the initial lineages
        if let Some(mutations) = &mut self.mutations {
            sequencing::update_sizes(mutations, &self.lineages);
        }
    }

    /// Perform a transfer and update the lineages
    pub fn transfer(&mut self) {
        if let Some(mutations) = &mut self.mutations {
            mutations.increment_transfer();
        }

        for _ in 0..self.phase_1_doublings {
            growth_phase_1(
                &mut self.lineages,
                &self.cfg,
                &mut self.rng,
                &mut self.mutations,
            );
        }

        growth_phase_2(
            &mut self.lineages,
            &self.cfg,
            &mut self.rng,
            &mut self.mutations,
        );

        if let Some(mutations) = &mut self.mutations {
            sequencing::update_sizes(mutations, &self.lineages);
        }
    }

    /// Get reference to the `LineagesData` owned by the handler  
    /// The lineages will be updated after each transfer  
    pub fn lineages(&self) -> &LineagesData {
        &self.lineages
    }

    /// Get optional reference to the `MutationsData`
    /// owned by the handler, which is only available
    /// if it was created with the `track_mutations` option
    pub fn mutations(&self) -> Option<&MutationsData> {
        self.mutations.as_ref()
    }

    /// Clear all pruned mutations being tracked,
    /// if sequencing/mutation tracking is enabled
    ///
    /// If it is disabled, nothing will happen
    pub fn clear_pruned_mutations(&mut self) {
        if let Some(mutations) = &mut self.mutations {
            mutations.pruned_muts.clear();
        }
    }
}
