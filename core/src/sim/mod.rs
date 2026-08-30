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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> SimConfig {
        SimConfig {
            replicates: 2,
            transfers: 3,
            markers: 1,
            dilution_factor: 100.0,
            beneficial_mutation_rate: 1.7e-6,
            neutral_mutation_rate: 0.0,
            deleterious_mutation_rate: 0.0,
            initial_beneficial_mutation_size: 0.012,
            fixed_deleterious_mutation_size: None,
            diminishing_returns_epistasis_strength: 6.0,
            seed: Some(42),
            max_pop_size: 5e8,
        }
    }

    #[test]
    fn test_handler_initial_state_is_none() {
        let handler = SimulationHandler::new(default_cfg(), false);
        assert!(handler.current_state().is_none());
    }

    #[test]
    fn test_handler_not_finished_at_start() {
        let handler = SimulationHandler::new(default_cfg(), false);
        assert!(!handler.is_finished());
    }

    #[test]
    fn test_handler_first_next_state_is_transfer_0() {
        let mut handler = SimulationHandler::new(default_cfg(), false);
        let state = handler.next_state().unwrap();
        assert_eq!(state.replicate, 1);
        assert_eq!(state.transfer, 0);
        assert!(!state.end_of_replicate);
    }

    #[test]
    fn test_handler_iterates_through_all_states() {
        let cfg = default_cfg(); // 2 replicates, 3 transfers
        let mut handler = SimulationHandler::new(cfg, false);
        let mut count = 0;
        while handler.next_state().is_some() {
            count += 1;
        }
        // Each replicate: transfer 0 (init) + transfers 1,2,3 = 4 states
        // 2 replicates × 4 = 8
        assert_eq!(count, 8);
    }

    #[test]
    fn test_handler_is_finished_after_all_states() {
        let mut handler = SimulationHandler::new(default_cfg(), false);
        while handler.next_state().is_some() {}
        assert!(handler.is_finished());
    }

    #[test]
    fn test_handler_returns_none_when_finished() {
        let mut handler = SimulationHandler::new(default_cfg(), false);
        while handler.next_state().is_some() {}
        assert!(handler.next_state().is_none());
        assert!(handler.next_state().is_none());
    }

    #[test]
    fn test_handler_end_of_replicate_flag() {
        let mut handler = SimulationHandler::new(default_cfg(), false);
        let mut end_of_replicate_count = 0;
        while let Some(state) = handler.next_state() {
            if state.end_of_replicate {
                end_of_replicate_count += 1;
                assert_eq!(state.transfer, 3);
            }
        }
        assert_eq!(end_of_replicate_count, 2); // one per replicate
    }

    #[test]
    fn test_handler_zero_replicates() {
        let mut cfg = default_cfg();
        cfg.replicates = 0;
        let mut handler = SimulationHandler::new(cfg, false);
        assert!(handler.next_state().is_none());
        assert!(handler.is_finished());
    }

    #[test]
    fn test_handler_deterministic_with_seed() {
        let cfg = default_cfg();
        let mut handler1 = SimulationHandler::new(cfg.clone(), false);
        let mut handler2 = SimulationHandler::new(cfg, false);

        while let (Some(s1), Some(s2)) = (handler1.next_state(), handler2.next_state()) {
            assert_eq!(s1.lineages.N, s2.lineages.N);
            assert_eq!(s1.lineages.W, s2.lineages.W);
        }
    }

    #[test]
    fn test_handler_mutations_tracked_when_enabled() {
        let mut handler = SimulationHandler::new(default_cfg(), true);
        let state = handler.next_state().unwrap();
        assert!(state.mutations.is_some());
    }

    #[test]
    fn test_handler_mutations_none_when_disabled() {
        let mut handler = SimulationHandler::new(default_cfg(), false);
        let state = handler.next_state().unwrap();
        assert!(state.mutations.is_none());
    }

    #[test]
    fn test_handler_lineages_initialized_at_transfer_0() {
        let cfg = default_cfg();
        let mut handler = SimulationHandler::new(cfg, false);
        let state = handler.next_state().unwrap();
        // Should have `markers` lineages at start
        assert_eq!(state.lineages.N.len(), 1);
        // Initial population = Nmax / D / markers = 5e8 / 100 / 1 = 5e6
        assert_eq!(state.lineages.N[0], 5_000_000.0);
        assert_eq!(state.lineages.W[0], 1.0);
    }

    #[test]
    fn test_handler_multiple_markers() {
        let mut cfg = default_cfg();
        cfg.markers = 3;
        let mut handler = SimulationHandler::new(cfg, false);
        let state = handler.next_state().unwrap();
        assert_eq!(state.lineages.N.len(), 3);
        // Each marker gets Nmax/D/markers = 5e8/100/3 ≈ 1666667
        let expected_n = (5e8_f64 / 100.0 / 3.0).round();
        for n in &state.lineages.N {
            assert_eq!(*n, expected_n);
        }
    }

    #[test]
    fn test_internal_sim_config_total_mutation_rate() {
        let mut cfg = default_cfg();
        cfg.beneficial_mutation_rate = 0.001;
        cfg.neutral_mutation_rate = 0.002;
        cfg.deleterious_mutation_rate = 0.003;
        let internal = InternalSimConfig::new(cfg);
        assert!((internal.total_mutation_rate - 0.006).abs() < 1e-10);
    }

    #[test]
    fn test_internal_sim_config_dilution_coefficient() {
        let cfg = default_cfg(); // D=100
        let internal = InternalSimConfig::new(cfg);
        assert!((internal.dilution_coefficient - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_sample_mutation_type_all_beneficial() {
        let mut cfg = default_cfg();
        cfg.beneficial_mutation_rate = 1.0;
        cfg.neutral_mutation_rate = 0.0;
        cfg.deleterious_mutation_rate = 0.0;
        let internal = InternalSimConfig::new(cfg);
        let mut rng = Pcg64::seed_from_u64(0);
        for _ in 0..100 {
            assert!(matches!(
                internal.sample_mutation_type(&mut rng),
                Some(types::MutationType::Beneficial)
            ));
        }
    }

    #[test]
    fn test_sample_mutation_type_zero_rates_returns_none() {
        let mut cfg = default_cfg();
        cfg.beneficial_mutation_rate = 0.0;
        cfg.neutral_mutation_rate = 0.0;
        cfg.deleterious_mutation_rate = 0.0;
        let internal = InternalSimConfig::new(cfg);
        let mut rng = Pcg64::seed_from_u64(0);
        assert!(internal.sample_mutation_type(&mut rng).is_none());
    }

    #[test]
    fn test_default_sim_rng_seeded_is_deterministic() {
        let cfg = default_cfg();
        let mut rng1 = default_sim_rng(&cfg);
        let mut rng2 = default_sim_rng(&cfg);
        let v1: Vec<u64> = (0..10).map(|_| rng1.gen()).collect();
        let v2: Vec<u64> = (0..10).map(|_| rng2.gen()).collect();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_default_sim_rng_no_seed_differs() {
        let mut cfg = default_cfg();
        cfg.seed = None;
        let mut rng1 = default_sim_rng(&cfg);
        let mut rng2 = default_sim_rng(&cfg);
        let v1: Vec<u64> = (0..10).map(|_| rng1.gen()).collect();
        let v2: Vec<u64> = (0..10).map(|_| rng2.gen()).collect();
        // Astronomically unlikely to be equal from entropy
        assert_ne!(v1, v2);
    }
}
