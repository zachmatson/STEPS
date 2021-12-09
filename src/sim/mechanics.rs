//! Mechanics of the growth, mutation, and bottlenecking processes  
//! Mid-level details between the high level transfer process and low-level computation kernels

use rand::distributions::{Distribution, Uniform};

use super::*;

/// Get the number of phase 1 doublings that must take place before phase 2,
/// given the dilution factor in `cfg`
pub fn phase_1_doublings_required(cfg: &SimConfig) -> usize {
    cfg.dilution_factor.log2().ceil() as usize - 1
}

/// Perform a single Phase 1 doubling on `data`
/// Optionally track mutations if `mutations_vec` is provided
///
/// The total population size is approximately doubled, with growth
/// run for whatever time step will provide that.  
/// New mutants are added and no bottlenecking occurs
pub fn growth_phase_1<R: Rng>(
    data: &mut LineagesData,
    cfg: &SimConfig,
    rng: &mut R,
    mutations: &mut Option<MutationsData>,
) {
    let avg_W = sum_N_and_avg_W(data).1;
    let delta_t = avg_W.recip();

    let mut old_N = data.N.clone();
    grow_lineages_inplace(data, delta_t);
    let delta_N = old_N_to_delta_N(data, &mut old_N);

    add_mutants(data, delta_N, cfg, rng, mutations);
}

/// Perform a single Phase 2 doubling on `data`
/// Optionally track mutations if `mutations_vec` is provided
///
/// Growth is run for whatever time step will bring the total population
/// size to approximately Nmax   
/// New mutants are added and bottlenecking occurs  
/// Only mutations which survive bottlenecking are generated and tracked
pub fn growth_phase_2<R: Rng>(
    data: &mut LineagesData,
    cfg: &SimConfig,
    rng: &mut R,
    mutations: &mut Option<MutationsData>,
) {
    let (sum_N, avg_W) = sum_N_and_avg_W(data);
    // Must grow population size to Nmax
    // Where growth is approximately a factor of 2^(avg_W * delta_t)
    let delta_t = (cfg.max_pop_size / sum_N).log2() / avg_W;

    assert!(delta_t >= 0.0);

    // old_N needed to calculate delta_N
    let old_N = data.N.clone();
    grow_lineages_inplace(data, delta_t);

    // More efficient to make new vectors to work off of, since many lineages
    // in the middle of the existing vectors won't survive
    // Cheaper to start over than delete a bunch from the middle
    let mut bottlenecked_data = LineagesData::successor(data);
    let mut delta_N = Vec::new();

    let len = data.N.len();
    // Ensures safety of unsafe region, length is only increased in the loop
    data.assert_len_ge(len);
    for i in 0..len {
        let mut lineage = unsafe { data.get_unchecked(i) };
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

    // Make data refer to the bottlenecked data,
    // dropping the old data from the heap
    *data = bottlenecked_data;

    add_mutants(data, &delta_N, cfg, rng, mutations);
}

/// Add the mutants corresponding to `delta_N` change in population size
/// to `data`, while adjusting existing population sizes in `data` to
/// remove the new mutants from old lineage sizes
fn add_mutants<R: Rng>(
    data: &mut LineagesData,
    delta_N: &[f64],
    cfg: &SimConfig,
    rng: &mut R,
    mutations: &mut Option<MutationsData>,
) {
    let expected_mutation_counts = expected_mutation_counts(data, delta_N);
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

    let mut cutoff_i = 0;
    let mut cutoff = cutoffs[cutoff_i];
    let mut expected_mutations_cumsum = 0.0;
    // Underlying data vector size will increase because mutants are being added
    // But we are only iterating through the lineages that already existed by
    // using the length of expected_mutation_counts, whose elements correspond
    // to the preexisting elements of data
    let len = expected_mutation_counts.len();
    data.assert_len_ge(len);
    'outer: for i in 0..len {
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
            let lineage = unsafe { data.get_unchecked(i) };
            // Iterate through mutants from the lineage
            while cutoff < expected_mutations_cumsum {
                // Find the number of mutations in the mutant
                let mut mutant_order: u32 = 0;
                // Upper bound (exclusive) corresponding to the same new individual mutant in the lineage
                let individual_max_cutoff = {
                    // Find start + (j+1)*U explained at top of 'outer
                    // given cutoff = start + (j+ε)*U for ε in [0, 1),
                    // without knowing j
                    // Min with expected_mutations_cumsum for fractional case
                    let tmp = cutoff - prev_cumsum;
                    tmp - tmp % lineage.U + lineage.U + prev_cumsum
                }
                .clamp(next_float(cutoff), expected_mutations_cumsum);
                // Above clamp guarantees individual_max_cutoff ∈ (cutoff, expected_mutations_cumsum]
                while cutoff < individual_max_cutoff {
                    mutant_order += 1;

                    cutoff_i += 1;
                    if cutoff_i < cutoffs.len() {
                        cutoff = cutoffs[cutoff_i];
                    } else {
                        break;
                    }
                }

                let mutant = new_mutant(lineage, mutant_order, cfg, rng);
                data.push_child(mutant, lineage, mutations);
                // N still includes the mutants that come from the lineage up until this point
                // No need to update `lineage` because its N field is not used here
                data.N[i] = (data.N[i] - 1.0).max(0.0);

                // No more cutoffs to try
                if cutoff_i >= cutoffs.len() {
                    break 'outer;
                }
            }
        }
    }
}

/// Generate a descendant lineage from `parent` with population size `1.0`  
/// Does not handle updating of IDs
fn new_mutant<R: Rng>(parent: Lineage, order: u32, cfg: &SimConfig, rng: &mut R) -> Lineage {
    let mut mutant = Lineage { N: 1.0, ..parent };

    for _ in 0..order {
        let mutation_type = cfg.sample_mutation_type(rng).unwrap();

        use MutationType::*;
        match mutation_type {
            Beneficial => apply_beneficial_mutation(&mut mutant, cfg, rng),
            Neutral => (),
            Deleterious => apply_deleterious_mutation(&mut mutant, cfg, rng),
            MutationRate => apply_mutation_rate_mutation(&mut mutant, cfg, rng),
        }
    }

    mutant
}

/// Applies a beneficial mutation to `lineage` in-place
fn apply_beneficial_mutation<R: Rng>(lineage: &mut Lineage, cfg: &SimConfig, rng: &mut R) {
    let size = rand_distr::Exp::new(lineage.secondary.lambda)
        .unwrap()
        .sample(rng);

    lineage.W *= 1.0 + size;
    lineage.secondary.lambda *= 1.0 + cfg.diminishing_returns_epistasis_strength * size;
}

/// Applies a deleterious mutation to `lineage` in-place
#[allow(unused_variables)]
fn apply_deleterious_mutation<R: Rng>(lineage: &mut Lineage, cfg: &SimConfig, rng: &mut R) {
    deleterious_todo()
}

/// Applies a mutation rate mutation to `lineage` in-place
#[allow(unused_variables)]
fn apply_mutation_rate_mutation<R: Rng>(lineage: &mut Lineage, cfg: &SimConfig, rng: &mut R) {
    mutation_rate_todo()
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
