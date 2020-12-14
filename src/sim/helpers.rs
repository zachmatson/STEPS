//! Helper structs, traits, and functions for doubling lineages and adding mutants  
//! Finer implementation details of the transfer process

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

    let old_N = data.N.clone();
    grow_lineages_inplace(data, delta_t);
    let expected_mutation_counts = expected_mutation_counts(data, &old_N);

    let len = data.N.len();
    let expected_mutation_counts = &expected_mutation_counts[0..len];

    for i in 0..len {
        let lambda = expected_mutation_counts[i];
        let N_mut = distr::poisson(lambda, rng);
        if N_mut > 0 {
            data.N[i] -= N_mut as f64;
            let current_lineage = unsafe { data.get_unchecked(i) };

            for _ in 0..N_mut {
                let mutant = new_mutant(current_lineage, cfg, rng);
                data.push_child(mutant, current_lineage, mutations_vec);
            }
        }
    }
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
    // let survivor_indices = Vec::new();
    let mut survivor_old_N = Vec::new();
    let mut survivor_N_after_growth = Vec::new();

    for i in 0..len {
        let mut lineage = unsafe { data.get_unchecked(i) };
        let N_bottlenecked = rand_distr::Binomial::new(lineage.N as u64, cfg.dilution_coefficient)
            .unwrap()
            .sample(rng);
        if N_bottlenecked > 0 {
            survivor_old_N.push(old_N[i]);
            survivor_N_after_growth.push(lineage.N);
            lineage.N = N_bottlenecked as f64;
            bottlenecked_data.push(lineage);
        }
    }

    *data = bottlenecked_data;

    let len = data.N.len();
    let survivor_old_N = &survivor_old_N[0..len];
    let survivor_N_after_growth = &survivor_N_after_growth[0..len];

    for i in 0..len {
        let old_N = survivor_old_N[i];
        let N_after_growth = survivor_N_after_growth[i];

        let lambda = data.U[i] * data.N[i] * (1.0 - (old_N / N_after_growth));
        let N_mut = distr::poisson(lambda, rng);
        if N_mut > 0 {
            data.N[i] -= N_mut as f64;
            let current_lineage = unsafe { data.get_unchecked(i) };

            for _ in 0..N_mut {
                let mutant = new_mutant(current_lineage, cfg, rng);
                data.push_child(mutant, current_lineage, mutations_vec);
            }
        }
    }
}

/// Generate a descendant lineage from `parent`
pub fn new_mutant<R: Rng>(parent: Lineage, cfg: &SimConfig, rng: &mut R) -> Lineage {
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
