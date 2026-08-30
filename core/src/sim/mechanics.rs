//! Mechanics of the growth, mutation, and bottlenecking processes  
//!
//! Mid-level details between the high level transfer process and low-level computation kernels

#![allow(clippy::needless_range_loop)]

use rand::distributions::{Distribution, Standard, Uniform};
use rand::Rng;

use crate::cfg::SimConfig;

use crate::sim::distr;
use crate::sim::kernels::{expected_mutation_counts, grow_lineages_inplace, old_N_to_delta_N};
use crate::sim::summarize;
use crate::sim::types::{Lineage, LineagesData, MutationType, MutationsData};
use crate::sim::InternalSimConfig;

/// Get the number of phase 1 doublings that must take place before phase 2, given the dilution
/// factor in `cfg`
pub fn phase_1_doublings_required(cfg: &SimConfig) -> usize {
    assert!(cfg.dilution_factor >= 2.0);

    let total_doublings = cfg.dilution_factor.log2();
    // We want at least 0.5 Phase2 doublings
    if total_doublings.fract() < 0.5 {
        total_doublings.floor() as usize - 1
    } else {
        total_doublings.floor() as usize
    }
}

/// Perform a single Phase 1 doubling on the `lineages` in place
///
/// The total population size is approximately doubled, with growth run for whatever time step will
/// provide that.  
///
/// New mutants are added and no bottlenecking occurs. Mutations will be tracked if `mutations_vec`
/// is provided.
pub(super) fn growth_phase_1<R: Rng>(
    cfg: &InternalSimConfig,
    lineages: &mut LineagesData,
    mutations: &mut Option<MutationsData>,
    rng: &mut R,
) {
    let avg_W = summarize::avg_W(lineages);
    let delta_t = avg_W.recip();

    let mut old_N = lineages.N.clone();
    grow_lineages_inplace(lineages, delta_t);
    let delta_N = old_N_to_delta_N(lineages, &mut old_N);

    add_mutants(cfg, lineages, mutations, delta_N, rng);
}

/// Perform a single Phase 2 doubling on the `lineages` in place
///
/// Growth is run for whatever time step will bring the total population size to approximately Nmax.
///
/// New mutants are added and bottlenecking occurs.
///
/// Mutations will be tracked if `mutations_vec` is provided. Only mutations which survive
/// bottlenecking are generated and tracked.
pub(super) fn growth_phase_2<R: Rng>(
    cfg: &InternalSimConfig,
    lineages: &mut LineagesData,
    mutations: &mut Option<MutationsData>,
    rng: &mut R,
) {
    let summarize::SumNAndAvgW { sum_N, avg_W } = summarize::sum_N_and_avg_W(lineages);
    // Must grow population size to Nmax
    // Where growth is approximately a factor of 2^(avg_W * delta_t)
    let delta_t = (cfg.inner.max_pop_size / sum_N).log2() / avg_W;

    assert!(delta_t >= 0.0);

    // old_N needed to calculate delta_N
    let old_N = lineages.N.clone();
    grow_lineages_inplace(lineages, delta_t);

    // More efficient to make new vectors to work off of, since many lineages
    // in the middle of the existing vectors won't survive
    // Cheaper to start over than delete a bunch from the middle
    let mut bottlenecked_data = LineagesData::successor(lineages);
    let mut delta_N = Vec::new();

    let len = lineages.N.len();
    // Ensures safety of unsafe region, length is only increased in the loop
    lineages.assert_len_eq(len);
    for i in 0..len {
        let mut lineage = unsafe { lineages.get_unchecked(i) };
        let N_bottlenecked =
            rand_distr::Binomial::new(lineage.N.round() as u64, cfg.dilution_coefficient)
                .unwrap()
                .sample(rng);
        if N_bottlenecked > 0 {
            let N_after_growth = lineage.N;
            lineage.N = N_bottlenecked as f64;
            bottlenecked_data.push(lineage);
            // Estimated number of cells in lineage.N that are new
            delta_N.push(lineage.N * (1.0 - old_N[i] / N_after_growth));
        }
    }

    // Make data refer to the bottlenecked data, dropping the old data from the heap
    *lineages = bottlenecked_data;

    add_mutants(cfg, lineages, mutations, &delta_N, rng);
}

/// Add the mutants corresponding to `delta_N` change in population size to `lineages`, while
/// adjusting existing population sizes in `lineages` to remove the new mutants from old lineage sizes
fn add_mutants<R: Rng>(
    cfg: &InternalSimConfig,
    lineages: &mut LineagesData,
    mutations: &mut Option<MutationsData>,
    delta_N: &[f64],
    rng: &mut R,
) {
    let expected_mutation_counts = expected_mutation_counts(lineages, delta_N);
    let expected_mutations = expected_mutation_counts.iter().sum::<f64>();
    assert!(expected_mutations >= 0.0);
    let num_mutations = distr::poisson(expected_mutations, rng);
    if num_mutations == 0 {
        return;
    }

    // Cutoffs store how far into the population each mutation occurs at,
    // in units of expected mutations
    let cutoffs_dist = Uniform::new(0.0, expected_mutations);
    let mut cutoffs: Vec<f64> = (0..num_mutations)
        .map(|_| cutoffs_dist.sample(rng))
        .collect();
    // Cutoffs must be in order for the iteration
    cutoffs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let mut cutoffs_iter = cutoffs.iter().copied();
    let mut cutoff = match cutoffs_iter.next() {
        Some(x) => x,
        None => return,
    };
    let mut cutoffs_exhausted = false;
    let mut expected_mutations_cumsum = 0.0;
    // Underlying data vector size will increase because mutants are being added
    // But we are only iterating through the lineages that already existed by
    // using the length of expected_mutation_counts, whose elements correspond
    // to the preexisting elements of data
    let len = expected_mutation_counts.len();
    lineages.assert_len_eq(len);
    for i in 0..len {
        // expected_mutations_cumsum increases with each loop, going from
        // expected_mutation_counts[0] after the first addition, to
        // expected_mutations after the last
        //
        // The cutoffs correspond to the cumulative sums and are along
        // the half-open interval [0, expected_mutations)
        //
        // For each lineage i (zero-indexed),
        // expected_mutation_counts[i] = delta_N[i] * data.U[i] =: Δ
        //
        // The lineage will get an interval of cutoffs [start, start + Δ)
        // Where start = previous expected_mutations_cumsum
        // and start + Δ = new expected_mutations_cumsum
        // Each new individual j (zero-indexed) in the lineage then gets an interval [start + j*U, start + (j+1)*U)
        // If the individual is the fractional part of the population size, its interval will be [start + j*U, start + Δ)

        // If all cells of a lineage became mutants, it may persist in the vector
        // with size 0.0 until the next bottleneck
        // This is a strict and not approximate equality because it should only
        // check for this narrow case, not just small lineages
        #[allow(clippy::float_cmp_const)]
        if expected_mutation_counts[i] == 0.0 {
            continue;
        }

        let prev_cumsum = expected_mutations_cumsum;
        expected_mutations_cumsum += expected_mutation_counts[i];

        if cutoff < expected_mutations_cumsum {
            let lineage = unsafe { lineages.get_unchecked(i) };
            // Iterate through mutants from the lineage
            while cutoff < expected_mutations_cumsum {
                // Find the number of mutations in the mutant
                let mut mutant_order: u32 = 0;
                // Upper bound (exclusive) corresponding to the same new individual mutant in the lineage
                let individual_max_cutoff = {
                    // Find start + (j+1)*U explained at top of 'outer
                    // given cutoff = start + (j+ε)*U for ε in [0, 1).
                    // Min with expected_mutations_cumsum for fractional case
                    let j = ((cutoff - prev_cumsum) / lineage.U).floor();
                    prev_cumsum + (j + 1.0) * lineage.U
                }
                .clamp(next_float(cutoff), expected_mutations_cumsum);
                // Above clamp guarantees individual_max_cutoff ∈ (cutoff, expected_mutations_cumsum]
                while cutoff < individual_max_cutoff {
                    mutant_order += 1;

                    if let Some(next_cutoff) = cutoffs_iter.next() {
                        cutoff = next_cutoff;
                    } else {
                        cutoffs_exhausted = true;
                        break;
                    }
                }

                let mutant = new_mutant(lineage, mutant_order, cfg, rng);
                lineages.push_child(mutant, lineage, mutant_order, mutations);
                // N still includes the mutants that come from the lineage up until this point
                // No need to update lineage because its N field is not used here
                lineages.N[i] = (lineages.N[i] - 1.0).max(0.0);

                // No more cutoffs to try
                if cutoffs_exhausted {
                    return;
                }
            }
        }
    }
}

/// Generate a descendant lineage from `parent` with population size `1.0`  
///
/// Does not handle updating of IDs
fn new_mutant<R: Rng>(
    parent: Lineage,
    order: u32,
    cfg: &InternalSimConfig,
    rng: &mut R,
) -> Lineage {
    let mut mutant = Lineage { N: 1.0, ..parent };

    for _ in 0..order {
        let mutation_type = cfg.sample_mutation_type(rng).unwrap();

        use MutationType::*;
        match mutation_type {
            Beneficial => apply_beneficial_mutation(&mut mutant, cfg, rng),
            Neutral => (),
            Deleterious => apply_deleterious_mutation(&mut mutant, cfg, rng),
        }
    }

    mutant
}

/// Applies a beneficial mutation to `lineage` in-place
fn apply_beneficial_mutation<R: Rng>(lineage: &mut Lineage, cfg: &InternalSimConfig, rng: &mut R) {
    let size = rand_distr::Exp::new(lineage.secondary.lambda)
        .unwrap()
        .sample(rng);

    lineage.W *= 1.0 + size;
    lineage.secondary.lambda *= 1.0 + cfg.inner.diminishing_returns_epistasis_strength * size;
}

/// Default distribution for deleterious mutation size, when a fixed size is not specified
///
/// This is a uniform distribution over [0.0, 1.0)
const DEFAULT_DELETERIOUS_MUTATION_SIZE_DISTRIBUTION: Standard = Standard;

/// Applies a deleterious mutation to `lineage` in-place
#[allow(unused_variables)]
fn apply_deleterious_mutation<R: Rng>(lineage: &mut Lineage, cfg: &InternalSimConfig, rng: &mut R) {
    let size = match cfg.inner.fixed_deleterious_mutation_size {
        // If a fixed size is provided, we will always use that
        Some(size) => size,
        // Otherwise, sample from [0.0, 1.0)
        None => DEFAULT_DELETERIOUS_MUTATION_SIZE_DISTRIBUTION.sample(rng),
    };

    lineage.W *= 1.0 - size;
    let G = cfg.inner.diminishing_returns_epistasis_strength
        / (size * (cfg.inner.diminishing_returns_epistasis_strength - 1.0) + 1.0);
    lineage.secondary.lambda *= 1.0 - G * size;
}

/// Get next float for finite floats
///
/// # Panics
///
/// Panics if `x` is `NaN` or infinite
fn next_float(x: f64) -> f64 {
    assert!(x.is_finite());
    f64::from_bits(x.to_bits() + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::types::SecondaryLineageData;
    use approx::assert_relative_eq;
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    fn seeded_rng() -> Pcg64 {
        Pcg64::seed_from_u64(12345)
    }

    fn default_sim_config() -> SimConfig {
        SimConfig {
            replicates: 1,
            transfers: 10,
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
    fn test_phase_1_doublings_d_100() {
        // D=100 → log2(100) ≈ 6.64, fract=0.64 ≥ 0.5 → floor = 6
        let cfg = default_sim_config();
        assert_eq!(phase_1_doublings_required(&cfg), 6);
    }

    #[test]
    fn test_phase_1_doublings_d_2() {
        // D=2 → log2(2) = 1.0, fract=0.0 < 0.5 → floor-1 = 0
        let mut cfg = default_sim_config();
        cfg.dilution_factor = 2.0;
        assert_eq!(phase_1_doublings_required(&cfg), 0);
    }

    #[test]
    fn test_phase_1_doublings_d_4() {
        // D=4 → log2(4) = 2.0, fract=0.0 < 0.5 → floor-1 = 1
        let mut cfg = default_sim_config();
        cfg.dilution_factor = 4.0;
        assert_eq!(phase_1_doublings_required(&cfg), 1);
    }

    #[test]
    fn test_phase_1_doublings_d_8() {
        // D=8 → log2(8) = 3.0, fract=0.0 < 0.5 → floor-1 = 2
        let mut cfg = default_sim_config();
        cfg.dilution_factor = 8.0;
        assert_eq!(phase_1_doublings_required(&cfg), 2);
    }

    #[test]
    fn test_phase_1_doublings_d_50() {
        // D=50 → log2(50) ≈ 5.64, fract=0.64 ≥ 0.5 → floor = 5
        let mut cfg = default_sim_config();
        cfg.dilution_factor = 50.0;
        assert_eq!(phase_1_doublings_required(&cfg), 5);
    }

    #[test]
    #[should_panic]
    fn test_phase_1_doublings_d_less_than_2_panics() {
        let mut cfg = default_sim_config();
        cfg.dilution_factor = 1.5;
        phase_1_doublings_required(&cfg);
    }

    #[test]
    fn test_next_float_positive() {
        let x = 1.0_f64;
        let y = next_float(x);
        assert!(y > x);
        assert_eq!(y.to_bits(), x.to_bits() + 1);
    }

    #[test]
    fn test_next_float_zero() {
        let x = 0.0_f64;
        let y = next_float(x);
        assert!(y > x);
        // Smallest positive subnormal
        assert_eq!(y, f64::from_bits(1));
    }

    #[test]
    #[should_panic]
    fn test_next_float_nan_panics() {
        next_float(f64::NAN);
    }

    #[test]
    #[should_panic]
    fn test_next_float_infinity_panics() {
        next_float(f64::INFINITY);
    }

    #[test]
    fn test_apply_beneficial_mutation_increases_fitness() {
        let cfg = InternalSimConfig::new(default_sim_config());
        let mut rng = seeded_rng();
        let mut lineage = Lineage {
            N: 1.0,
            W: 1.0,
            U: cfg.total_mutation_rate,
            secondary: SecondaryLineageData {
                lambda: cfg.inner.initial_beneficial_mutation_size.recip(),
                id: 1,
                parent_id: 0,
                marker: 1,
                accumulated_muts: 1,
            },
        };
        let original_w = lineage.W;
        apply_beneficial_mutation(&mut lineage, &cfg, &mut rng);
        assert!(lineage.W > original_w);
    }

    #[test]
    fn test_apply_beneficial_mutation_increases_lambda() {
        let cfg = InternalSimConfig::new(default_sim_config());
        let mut rng = seeded_rng();
        let mut lineage = Lineage {
            N: 1.0,
            W: 1.0,
            U: cfg.total_mutation_rate,
            secondary: SecondaryLineageData {
                lambda: cfg.inner.initial_beneficial_mutation_size.recip(),
                id: 1,
                parent_id: 0,
                marker: 1,
                accumulated_muts: 1,
            },
        };
        let original_lambda = lineage.secondary.lambda;
        apply_beneficial_mutation(&mut lineage, &cfg, &mut rng);
        // Diminishing returns: lambda should increase
        assert!(lineage.secondary.lambda > original_lambda);
    }

    #[test]
    fn test_apply_deleterious_mutation_decreases_fitness() {
        let mut cfg = default_sim_config();
        cfg.fixed_deleterious_mutation_size = Some(0.1);
        let cfg = InternalSimConfig::new(cfg);
        let mut rng = seeded_rng();
        let mut lineage = Lineage {
            N: 1.0,
            W: 1.0,
            U: cfg.total_mutation_rate,
            secondary: SecondaryLineageData {
                lambda: cfg.inner.initial_beneficial_mutation_size.recip(),
                id: 1,
                parent_id: 0,
                marker: 1,
                accumulated_muts: 1,
            },
        };
        apply_deleterious_mutation(&mut lineage, &cfg, &mut rng);
        // W *= (1 - 0.1) = 0.9
        assert_relative_eq!(lineage.W, 0.9, epsilon = 1e-10);
    }

    #[test]
    fn test_apply_deleterious_fixed_size() {
        let mut cfg = default_sim_config();
        cfg.fixed_deleterious_mutation_size = Some(0.2);
        let cfg = InternalSimConfig::new(cfg);
        let mut rng = seeded_rng();
        let mut lineage = Lineage {
            N: 1.0,
            W: 1.5,
            U: cfg.total_mutation_rate,
            secondary: SecondaryLineageData {
                lambda: cfg.inner.initial_beneficial_mutation_size.recip(),
                id: 1,
                parent_id: 0,
                marker: 1,
                accumulated_muts: 1,
            },
        };
        apply_deleterious_mutation(&mut lineage, &cfg, &mut rng);
        assert_relative_eq!(lineage.W, 1.5 * 0.8, epsilon = 1e-10);
    }

    #[test]
    fn test_growth_phase_1_doubles_population() {
        let sim_cfg = default_sim_config();
        let cfg = InternalSimConfig::new(sim_cfg);
        let mut rng = seeded_rng();

        let mut lineages = LineagesData::default();
        lineages.N.push(1000.0);
        lineages.W.push(1.0);
        lineages.U.push(0.0); // no mutations
        lineages.secondary.push(SecondaryLineageData::default());

        let n_before = lineages.N[0];
        growth_phase_1(&cfg, &mut lineages, &mut None, &mut rng);
        // With U=0 and W=1.0, population should approximately double
        assert_relative_eq!(lineages.N[0], n_before * 2.0, epsilon = 1e-6);
    }
}
