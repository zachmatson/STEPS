//! Helper structs, traits, and functions for doubling lineages and adding mutants  
//! Finer implementation details of the transfer process

use rand::distributions::{Distribution, Uniform};

use super::*;

pub fn phase_1_doublings_required(cfg: &SimConfig) -> usize {
    cfg.dilution_factor.log2().floor() as usize
}

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
    let delta_N = delta_N_inplace(data, &mut old_N);

    add_mutants(data, delta_N, cfg, rng, mutations_vec);
}

pub fn growth_phase_2<R: Rng>(
    data: &mut LineagesData,
    cfg: &SimConfig,
    rng: &mut R,
    mutations_vec: &mut Option<Vec<Mutation>>,
) {
    let (sum_N, avg_W) = sum_N_and_avg_W(data);
    let delta_t = (cfg.max_pop_size as f64 / sum_N).log2() / avg_W;

    let old_N = data.N.clone();
    grow_lineages_inplace(data, delta_t);

    let len = data.N.len();
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
            delta_N.push(1.0 - old_N[i] / N_after_growth);
        }
    }

    *data = bottlenecked_data;

    add_mutants(data, &mut delta_N, cfg, rng, mutations_vec);
}

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

    let cutoffs_dist = Uniform::new(0.0, expected_mutations);
    let mut cutoffs: Vec<f64> = (0..num_mutations).map(|_| cutoffs_dist.sample(rng)).collect();
    cutoffs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let mut expected_mutations_cumsum = 0.0;
    let mut cutoff_i = 0;
    let len = expected_mutation_counts.len();
    'outer: for i in 0..len {
        expected_mutations_cumsum += expected_mutation_counts[i];

        if expected_mutations_cumsum >= cutoffs[cutoff_i] {
            let lineage = unsafe { data.get_unchecked(i) };
            while expected_mutations_cumsum >= cutoffs[cutoff_i] {
                let mutant = new_mutant(lineage, cfg, rng);
                data.push_child(mutant, lineage, mutations_vec);

                cutoff_i += 1;

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
