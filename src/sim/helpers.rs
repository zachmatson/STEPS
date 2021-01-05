//! Helper functions for doubling lineages and adding mutants  
//! Low to mid-level implementation details of the transfer process

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
    mutations_vec: &mut Option<Vec<Mutation>>,
) {
    let avg_W = sum_N_and_avg_W(data).1;
    let delta_t = avg_W.recip();

    let mut old_N = data.N.clone();
    grow_lineages_inplace(data, delta_t);
    let delta_N = old_N_to_delta_N(data, &mut old_N);

    add_mutants(data, delta_N, cfg, rng, mutations_vec);
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
    mutations_vec: &mut Option<Vec<Mutation>>,
) {
    let (sum_N, avg_W) = sum_N_and_avg_W(data);
    // Must grow population size to Nmax
    // Where growth is approximately a factor of 2^(avg_W * delta_t)
    let delta_t = (cfg.max_pop_size as f64 / sum_N).log2() / avg_W;

    // old_N needed to calculate delta_N
    let old_N = data.N.clone();
    grow_lineages_inplace(data, delta_t);

    let len = data.N.len();
    // Need new container because length will change from lineages that don't survive
    let mut bottlenecked_data = LineagesData::successor(&data);
    let mut delta_N = Vec::new();

    for i in 0..len {
        let mut lineage = unsafe { data.get_unchecked(i) };
        let N_bottlenecked =
            rand_distr::Binomial::new(lineage.N.ceil() as u64, cfg.dilution_coefficient)
                .unwrap()
                .sample(rng);
        if N_bottlenecked > 0 {
            let N_after_growth = lineage.N;
            lineage.N = N_bottlenecked as f64;
            bottlenecked_data.push(lineage);
            delta_N.push(lineage.N * (1.0 - old_N[i] / N_after_growth));
        }
    }

    // Make data refer to the bottlenecked data,
    // dropping the old data from the heap
    *data = bottlenecked_data;

    add_mutants(data, &delta_N, cfg, rng, mutations_vec);
}

/// Add the mutants corresponding to `delta_N` change in population size
/// to `data`, while adjusting existing population sizes in `data` to
/// remove the new mutants from old lineage sizes
fn add_mutants<R: Rng>(
    data: &mut LineagesData,
    delta_N: &[f64],
    cfg: &SimConfig,
    rng: &mut R,
    mutations_vec: &mut Option<Vec<Mutation>>,
) {
    let expected_mutation_counts = expected_mutation_counts(data, delta_N);
    let expected_mutations = expected_mutation_counts.iter().sum::<f64>();
    let num_mutations = distr::poisson(expected_mutations, rng);
    if num_mutations == 0 {
        return;
    }

    // Cutoffs store the number of expected mutations into the population
    // that each mutation occurs at
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
    // to the starting elements of data
    let len = expected_mutation_counts.len();
    'outer: for i in 0..len {
        // expected_mutations_cumsum increases with each loop, going from
        // expected_mutation_counts[1] after the first addition, to
        // expected_mutations after the last
        //
        // The cutoffs correspond to the cumulative sums but are along
        // the half-open interval [0, expected_mutations)
        //
        // For each lineage i (zero-indexed),
        // expected_mutation_counts[i] = delta_N[i] * data.U[i] =: Δ
        //
        // The lineage will get an interval of cutoffs [start, start + Δ)
        // Where start = expected_mutations_cumsum[i-1]
        // Each individual j (zero-indexed) in the lineage then gets an interval [start + j*U, start + (j+1)*U)

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
                    let tmp = cutoff - prev_cumsum;
                    tmp - tmp % lineage.U + lineage.U + prev_cumsum
                };
                while cutoff < individual_max_cutoff {
                    mutant_order += 1;

                    if cutoff_i < cutoffs.len() {
                        cutoff = cutoffs[cutoff_i];
                        cutoff_i += 1;
                    } else {
                        break;
                    }
                }

                let mutant = new_mutant(lineage, cfg, rng);
                data.push_child(mutant, lineage, mutations_vec);
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

/// Generate a descendant lineage from `parent`
fn new_mutant<R: Rng>(parent: Lineage, cfg: &SimConfig, rng: &mut R) -> Lineage {
    let mutation_type = cfg.sample_mutation_type(rng).unwrap();

    let (W, lambda) = match mutation_type {
        MutationType::Beneficial => updates_after_beneficial_mutation(parent, cfg, rng),
        MutationType::Deleterious => updates_after_deleterious_mutation(parent, cfg, rng),
        MutationType::Neutral | MutationType::MutationRate => (parent.W, parent.secondary.lambda),
    };

    // let U = match mutation_type {
    //     MutationType::MutationRate => mutation_rate_todo(),
    //     _ => parent.U,
    // };

    Lineage {
        N: 1.0,
        W,
        secondary: SecondaryLineageData {
            lambda,
            ..parent.secondary
        },
        ..parent
    }
}

/// Generate fitness and mutation size lambda of a descendant of `parent` after undergoing a beneficial mutation
fn updates_after_beneficial_mutation<R: Rng>(
    parent: Lineage,
    cfg: &SimConfig,
    rng: &mut R,
) -> (f64, f64) {
    let mutation_size = rand_distr::Exp::new(parent.secondary.lambda)
        .unwrap()
        .sample(rng);
    let lambda_new = parent.secondary.lambda
        * (1.0 + cfg.diminishing_returns_epistasis_strength * mutation_size);
    let W_new = parent.W * (1.0 + mutation_size);

    (W_new, lambda_new)
}

/// Generate fitness and mutation size lambda of a descendant of `parent` after undergoing a deleterious mutation
#[allow(unused_variables)]
fn updates_after_deleterious_mutation<R: Rng>(
    parent: Lineage,
    cfg: &SimConfig,
    rng: &mut R,
) -> (f64, f64) {
    deleterious_todo()
}
