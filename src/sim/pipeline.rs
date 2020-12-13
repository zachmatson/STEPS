use super::LineagesData;

use super::*;

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
        let N_mut = fast_distr::poisson(lambda, rng);
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
    let mut bottlenecked_data = LineagesData::default();
    bottlenecked_data.reserve(len);
    let mut survivor_indices = Vec::new();

    for i in 0..len {
        let mut lineage = unsafe { data.get_unchecked(i) };
        let N_bottlenecked = rand_distr::Binomial::new(lineage.N as u64, cfg.dilution_coefficient)
            .unwrap()
            .sample(rng);
        if N_bottlenecked > 0 {
            lineage.N = N_bottlenecked as f64;
            bottlenecked_data.push(lineage);
            survivor_indices.push(i);
        }
    }

    let data_after_growth = std::mem::replace(data, bottlenecked_data);
    let N_after_growth = data_after_growth.N;

    let len = data.N.len();
    let survivor_indices = &survivor_indices[0..len];

    for i in 0..len {
        let old_i = survivor_indices[i];
        let old_N = old_N[old_i];
        let N_after_growth = N_after_growth[old_i];

        let lambda = data.U[i] * data.N[i] * (1.0 - (old_N / N_after_growth));
        let N_mut = fast_distr::poisson(lambda, rng);
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

/*
Pipeline:
    ---Get avg W
    Grow phase 1
    Add mutants phase 1

    Get avg W
    Grow phase 2
    Bottleneck
    Add mutants phase 2
*/
