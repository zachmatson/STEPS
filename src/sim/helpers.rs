use super::*;

struct Phase1();

impl GrowthCalculator for Phase1 {
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineages: &Lineages,
        idx: usize,
        cfg: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64) {
        // Calculate size after growth
        let N_after_growth = (lineages.W[idx].exp2() * lineages.N[idx] as f64).round() as u64;

        // Check number of mutations
        // Sample if mutation rate is positive, otherwise no mutations
        let N_mut: u64 = if lineages.U[idx] > 0.0 {
            rand_distr::Poisson::new(lineages.U[idx] * (N_after_growth - lineages.N[idx]) as f64)
                .unwrap()
                .sample(rng)
        } else {
            0
        };

        // Subtract number of mutants to get new size
        let new_N = N_after_growth - N_mut;

        if new_N < lineages.N[i] {
            eprintln!("WARNING: N_mut exceeded amount of new cells");
        }

        (new_N, N_mut)
    }
}

struct Phase2 {
    delta_t: f64,
}

impl GrowthCalculator for Phase2 {
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineages: &Lineages,
        idx: usize,
        cfg: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64) {
        let N_bottlenecked =
            sample_bottlenecked_size_with_growth(self.delta_t, lineages.N[i], lineages.W[i], cfg, rng);

        if N_bottlenecked == 0 {
            return (0, 0);
        }

        let N_mut = if lineages.U[idx] > 0.0 {
            rand_distr::Poisson::new(
                lineages.U[idx]
                    * N_bottlenecked as f64
                    * (1.0 - (lineages.W[idx] * self.delta_t).exp2().recip()),
            )
            .unwrap()
            .sample(rng)
        } else {
            0
        };

        let new_N = N_bottlenecked - N_mut;

        (new_N, N_mut)
    }
}

pub fn estimate_phase_1_delta_t(sum_N: u64, avg_W: f64, cfg: &SimConfig) -> usize {
    ((cfg.max_pop_size as f64 / sum_N as f64).log(avg_W.exp2()) - 1.0)
        .floor()
        .max(0.0) as usize
}

pub fn estimate_phase_2_delta_t(lineages: &Lineages, cfg: &SimConfig) -> f64 {
    let weighted_sum_W = lineages
        .W
        .iter()
        .zip(lineages.N.iter())
        .map(|(w, n)| (*w) * (*n) as f64)
        .sum::<f64>();
    let sum_N = lineages.N.iter().sum::<u64>() as f64;
    let avg_W = weighted_sum_W / sum_N;

    (cfg.max_pop_size as f64 / sum_N).log2() / avg_W
}

pub fn sample_bottlenecked_size_with_growth<R: Rng>(
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

/// Push a mutant based on `initial_W` and `initial_U` with random mutation type to the end of `output_lineages`
pub fn push_new_mutant<R: Rng>(
    initial_W: f64,
    initial_U: f64,
    output_lineages: &mut Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) {
    let mutation_type = cfg.sample_mutation_type(rng).unwrap();

    output_lineages.N.push(1);
    output_lineages.U.push(initial_U);
    output_lineages.W.push(match mutation_type {
        MutationType::Beneficial => {
            let alpha = (1.0 + cfg.diminishing_returns_epistasis_strength * (initial_W - 1.0))
                / cfg.initial_beneficial_mutation_size;
            let mutation_size = rand_distr::Exp::new(alpha).unwrap().sample(rng);
            initial_W + mutation_size
        }
        MutationType::Neutral => initial_W,
        MutationType::Deleterious => deleterious_todo(),
        MutationType::MutationRate => mutation_rate_todo(),
    });
}
