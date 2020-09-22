use super::*;

pub fn estimate_phase_2_delta_t(lineages: &Lineages, cfg: &SimConfig) -> f64 {
    let weighted_sum_W = lineages
        .W
        .iter()
        .zip(lineages.N.iter())
        .map(|(w, n)| (*w) * (*n) as f64)
        .sum::<f64>();
    let sum_N = lineages.N.iter().sum::<u64>() as f64;
    let avg_W = weighted_sum_W / sum_N;
    println!("Sum N: {}", sum_N);

    (cfg.max_pop_size as f64 / sum_N).log2() / avg_W
}

pub fn calculate_bottlenecked_size_with_growth<R: Rng>(
    delta_t: f64,
    N: u64,
    W: f64,
    cfg: &SimConfig,
    rng: &mut R,
) -> u64 {
    let N_after_growth = ((W * delta_t).exp2() * N as f64).round() as u64;
    rand_distr::Binomial::new(N_after_growth, cfg.dilution_factor.recip())
        .unwrap()
        .sample(rng)
}

/// Push a mutant based on the `idx`th element of `input_lineages` to the end of `output_lineages`
pub fn push_new_mutant<R: Rng>(
    initial_W: f64,
    initial_U: f64,
    output_lineages: &mut Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) {
    let mutation_type = cfg.sample_mutation_type(rng).unwrap();
    match mutation_type {
        MutationType::Beneficial => {
            let alpha = (1.0 + cfg.diminishing_returns_epistasis_strength * (initial_W - 1.0))
                / cfg.initial_beneficial_mutation_size;
            let mutation_size = rand_distr::Exp::new(alpha).unwrap().sample(rng);
            output_lineages.N.push(1);
            output_lineages.W.push(initial_W + mutation_size);
            output_lineages.U.push(initial_U);
        }
        MutationType::Neutral => {
            output_lineages.N.push(1);
            output_lineages.W.push(initial_W);
            output_lineages.U.push(initial_U);
        }
        MutationType::Deleterious => cfg::deleterious_todo(),
        MutationType::MutationRate => cfg::mutation_rate_todo(),
    }
}
