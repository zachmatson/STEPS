//! Code for running the simulations and types used for storing simulation data

// Many biological parameters like "N", "W", or "U" will be expressed here with capitalization
// that does not match the normal Rust snake-case guidelines
#![allow(non_snake_case)]

use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;
use rand_pcg::Pcg64;

use crate::cfg::SimConfig;

use mechanics::{growth_phase_1, growth_phase_2, phase_1_doublings_required};
use types::MutationType;

mod distr;
mod kernels;
mod mechanics;
mod sequencing;
mod types;

pub mod summarize;

pub use types::{LineagesData, Mutation, MutationsData};

/// Handler to run the simulations from config, exposing intermediate state with an iterator-like
/// interface
pub struct SimulationHandler {
    /// Current replicate
    replicate: u32,
    /// Current transfer
    transfer: u32,
    /// Simulation options
    cfg: InternalSimConfig,
    /// Lineages  
    ///
    /// Must be created/reset before a new replicate
    lineages: LineagesData,
    /// Mutation data for sequencing  
    ///
    /// Must be created/reset before a new replicate  
    ///
    /// Handler responsible for clearing pruned mutations
    mutations: Option<MutationsData>,
    /// RNG to use for all replicates
    rng: SimRng,
}

impl SimulationHandler {
    /// Create a new `SimulationHandler`
    ///
    /// To start, there will be no `current_state`, `next_state` needs to be called to go
    /// through all of the possible states including the first one
    pub fn new(cfg: SimConfig, track_mutations: bool) -> Self {
        Self {
            replicate: 0,
            transfer: 0,
            lineages: LineagesData::default(),
            mutations: match track_mutations {
                true => Some(MutationsData::default()),
                false => None,
            },
            rng: default_sim_rng(&cfg),
            cfg: InternalSimConfig::new(cfg),
        }
    }

    /// Get the current state of the handled simulations, or `None` if the simulations have not been
    /// advanced yet or the number of total replicates is zero
    pub fn current_state(&self) -> Option<SimulationState> {
        if self.replicate > 0 {
            Some(SimulationState {
                replicate: self.replicate,
                transfer: self.transfer,
                end_of_replicate: self.transfer == self.cfg.inner.transfers,
                lineages: &self.lineages,
                mutations: self.mutations.as_ref(),
            })
        } else {
            None
        }
    }

    /// If possible, advance the state of the handled simulations and return the new state, or do
    /// nothing and return `None` with the state left unchanged if it cannot be advanced any more
    pub fn next_state(&mut self) -> Option<SimulationState> {
        if let Some(SimulationState {
            end_of_replicate: false,
            ..
        }) = self.current_state()
        {
            self.transfer += 1;
        } else if self.replicate < self.cfg.inner.replicates {
            self.replicate += 1;
            self.transfer = 0;
        } else {
            return None;
        }

        if let Some(mutations) = &mut self.mutations {
            // Must clear pruned mutations before transferring/starting replicate so that the returned
            // mutation data will only have the most recently pruned mutations
            mutations.pruned_muts.clear();
            mutations.set_transfer(self.transfer);
        }

        // Perform updates on underlying lineages
        if self.transfer == 0 {
            self.start_replicate();
        } else {
            self.perform_transfer();
        }

        self.current_state()
    }

    /// Whether the simulations are finished
    ///
    /// This function returning `true` means `next_state` will return `None` and vice versa
    pub fn is_finished(&self) -> bool {
        // Number of transfers doesn't matter if replicates is 0
        self.replicate == self.cfg.inner.replicates
            && (self.replicate == 0 || self.transfer == self.cfg.inner.transfers)
    }

    /// Initialization that must be performed at the start of each replicate
    fn start_replicate(&mut self) {
        self.mutations = self.mutations.as_ref().map(|_| MutationsData::new());
        self.lineages = LineagesData::for_sim_config(&self.cfg, &mut self.mutations);

        // We need the initial sequencing information from the initial lineages
        if let Some(mutations) = &mut self.mutations {
            sequencing::update_sizes(mutations, &self.lineages);
        }
    }

    /// Perform a transfer on the underlying lineages and update mutations if applicable
    fn perform_transfer(&mut self) {
        for _ in 0..self.cfg.phase_1_doublings {
            growth_phase_1(
                &self.cfg,
                &mut self.lineages,
                &mut self.mutations,
                &mut self.rng,
            );
        }

        growth_phase_2(
            &self.cfg,
            &mut self.lineages,
            &mut self.mutations,
            &mut self.rng,
        );

        if let Some(mutations) = &mut self.mutations {
            sequencing::update_sizes(mutations, &self.lineages);
        }
    }
}

/// A snapshot of the simulation state at some point in time
pub struct SimulationState<'a> {
    /// Replicate this state is for
    pub replicate: u32,
    /// Transfer this state is for
    pub transfer: u32,
    /// Whether this state is the last state for the current replicate
    pub end_of_replicate: bool,
    /// Lineage data
    pub lineages: &'a LineagesData,
    /// Mutation data, if sequencing is enabled for the simulations
    pub mutations: Option<&'a MutationsData>,
}

/// Simulation options, including those which cannot be set externally and must be computed
struct InternalSimConfig {
    /// Underlying external config
    pub inner: SimConfig,

    /// Total mutation rate
    pub total_mutation_rate: f64,
    /// Reciprocal of dilution factor
    pub dilution_coefficient: f64,
    /// Number of phase 1 doublings to perform in each transfer
    pub phase_1_doublings: usize,

    /// Distribution from which to pick the type of each new mutation
    mutation_type_index_distribution: Option<WeightedIndex<f64>>,
}

impl InternalSimConfig {
    /// Create an `InternalSimConfig` from a normal `SimConfig`
    pub fn new(cfg: SimConfig) -> Self {
        let total_mutation_rate = cfg.beneficial_mutation_rate
            + cfg.neutral_mutation_rate
            + cfg.deleterious_mutation_rate;

        Self {
            total_mutation_rate,
            dilution_coefficient: cfg.dilution_factor.recip(),
            phase_1_doublings: phase_1_doublings_required(&cfg),
            mutation_type_index_distribution: if total_mutation_rate > 0.0 {
                Some(
                    WeightedIndex::new(vec![
                        cfg.beneficial_mutation_rate,
                        cfg.neutral_mutation_rate,
                        cfg.deleterious_mutation_rate,
                    ])
                    .unwrap(),
                )
            } else {
                None
            },
            inner: cfg,
        }
    }

    /// Available mutation types, in same order as the mutation type index distribution
    const MUTATION_TYPES: [MutationType; 3] = [
        MutationType::Beneficial,
        MutationType::Neutral,
        MutationType::Deleterious,
    ];

    /// Randomly pick a mutation type weighted by the mutation rates selected  
    ///
    /// Will return `None` iff all mutation rates are 0
    pub fn sample_mutation_type<R: Rng>(&self, rng: &mut R) -> Option<MutationType> {
        self.mutation_type_index_distribution
            .as_ref()
            .map(|dist| Self::MUTATION_TYPES[dist.sample(rng)])
    }
}

/// RNG used for the simulations  
/// Will be a type that implements the `Rng` trait from `rand`   
type SimRng = Pcg64;

/// Instantiate RNG to use for the simulations  
///
/// Uses seed if one is given, otherwise seeds from system entropy  
fn default_sim_rng(cfg: &SimConfig) -> SimRng {
    match cfg.seed {
        Some(seed) => SimRng::seed_from_u64(seed),
        None => SimRng::from_entropy(),
    }
}
