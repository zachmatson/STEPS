//! Integration tests checking whole-simulation results against known analytical cases
//!
//! These drive `SimulationHandler` through the public API and assert properties that follow
//! from the model definition, rather than from any particular implementation detail.

// Many biological parameters like "N", "W", or "U" will be expressed here with capitalization
// that does not match the normal Rust snake-case guidelines
#![allow(non_snake_case)]

use steps_core::cfg::SimConfig;
use steps_core::sim::{summarize, SimulationHandler};

/// Base config with no mutations, so that results are analytically predictable
///
/// Individual tests override the mutation rates they care about.
fn base_cfg() -> SimConfig {
    SimConfig {
        replicates: 1,
        transfers: 5,
        markers: 2,
        dilution_factor: 100.0,
        beneficial_mutation_rate: 0.0,
        neutral_mutation_rate: 0.0,
        deleterious_mutation_rate: 0.0,
        initial_beneficial_mutation_size: 0.012,
        fixed_deleterious_mutation_size: None,
        diminishing_returns_epistasis_strength: 6.0,
        seed: Some(42),
        max_pop_size: 5e8,
    }
}

/// Mutation rate high enough that mutants are certain to appear within a few transfers
///
/// Roughly `max_pop_size * RATE` mutations are expected per transfer.
const ACTIVE_MUTATION_RATE: f64 = 1e-6;

#[test]
fn without_mutations_fitness_stays_at_one() {
    // Every lineage descends from the ancestor at W=1.0 and nothing can change W,
    // so both the mean and the maximum must remain exactly 1.0.
    let mut handler = SimulationHandler::new(base_cfg(), false);

    while let Some(state) = handler.next_state() {
        assert_eq!(summarize::avg_W(state.lineages), 1.0);
        assert_eq!(summarize::max_W(state.lineages), 1.0);
    }
}

#[test]
fn without_mutations_genotype_count_stays_at_marker_count() {
    // No mutants are created, so the only lineages are the initial one per marker.
    let cfg = base_cfg();
    let markers = cfg.markers as usize;
    let mut handler = SimulationHandler::new(cfg, false);

    while let Some(state) = handler.next_state() {
        assert_eq!(summarize::genotype_count(state.lineages), markers);
    }
}

#[test]
fn neutral_mutations_do_not_change_fitness() {
    // Neutral mutations create new lineages but leave W untouched, so the mean fitness
    // must stay exactly 1.0 even though the population diversifies.
    let cfg = SimConfig {
        neutral_mutation_rate: ACTIVE_MUTATION_RATE,
        ..base_cfg()
    };
    let markers = cfg.markers as usize;
    let mut handler = SimulationHandler::new(cfg, false);
    let mut final_genotypes = 0;

    while let Some(state) = handler.next_state() {
        assert_eq!(summarize::avg_W(state.lineages), 1.0);
        assert_eq!(summarize::max_W(state.lineages), 1.0);
        final_genotypes = summarize::genotype_count(state.lineages);
    }

    // Confirm mutations actually occurred, so the assertions above are not vacuous
    assert!(final_genotypes > markers);
}

#[test]
fn beneficial_mutations_never_reduce_fitness_below_one() {
    // Beneficial mutations multiply W by (1 + size) for size > 0, so no lineage can
    // ever fall below the ancestral W=1.0.
    let cfg = SimConfig {
        beneficial_mutation_rate: ACTIVE_MUTATION_RATE,
        ..base_cfg()
    };
    let mut handler = SimulationHandler::new(cfg, false);
    let mut final_max_W = 0.0;

    while let Some(state) = handler.next_state() {
        assert!(summarize::avg_W(state.lineages) >= 1.0);
        final_max_W = summarize::max_W(state.lineages);
        assert!(final_max_W >= 1.0);
    }

    // Confirm mutations actually occurred, so the assertions above are not vacuous
    assert!(final_max_W > 1.0);
}

#[test]
fn deleterious_mutations_never_raise_fitness_above_one() {
    // Deleterious mutations multiply W by (1 - size) for size in (0, 1), so no lineage
    // can ever rise above the ancestral W=1.0.
    let cfg = SimConfig {
        deleterious_mutation_rate: ACTIVE_MUTATION_RATE,
        fixed_deleterious_mutation_size: Some(0.5),
        ..base_cfg()
    };
    let markers = cfg.markers as usize;
    let mut handler = SimulationHandler::new(cfg, false);
    let mut final_genotypes = 0;

    while let Some(state) = handler.next_state() {
        assert!(summarize::avg_W(state.lineages) <= 1.0);
        assert!(summarize::max_W(state.lineages) <= 1.0);
        final_genotypes = summarize::genotype_count(state.lineages);
    }

    // Confirm mutations actually occurred, so the assertions above are not vacuous
    assert!(final_genotypes > markers);
}

#[test]
fn initial_population_size_is_max_pop_over_dilution_factor() {
    // Each replicate starts at the post-bottleneck size, Nmax/D, split evenly between markers
    let cfg = base_cfg();
    let expected = cfg.max_pop_size / cfg.dilution_factor;
    let mut handler = SimulationHandler::new(cfg, false);

    let state = handler.next_state().unwrap();
    assert_eq!(state.transfer, 0);
    assert_eq!(summarize::sum_N_and_avg_W(state.lineages).sum_N, expected);
}

#[test]
fn population_returns_to_max_pop_over_dilution_factor_after_each_transfer() {
    // Every transfer grows the population to Nmax and then bottlenecks it by a factor of D,
    // so the sampled size should land near Nmax/D. Bottlenecking is binomial, so allow a
    // small tolerance around the expected value.
    let cfg = base_cfg();
    let expected = cfg.max_pop_size / cfg.dilution_factor;
    let mut handler = SimulationHandler::new(cfg, false);

    while let Some(state) = handler.next_state() {
        let sum_N = summarize::sum_N_and_avg_W(state.lineages).sum_N;
        assert!(
            (sum_N - expected).abs() / expected < 0.02,
            "transfer {} had total population {}, expected near {}",
            state.transfer,
            sum_N,
            expected
        );
    }
}

#[test]
fn markers_start_evenly_balanced() {
    // Both markers are seeded with the same population size, so their ratio starts at exactly 1
    let mut handler = SimulationHandler::new(base_cfg(), false);

    let state = handler.next_state().unwrap();
    assert_eq!(summarize::marker_1_ratio(state.lineages), 1.0);
}
