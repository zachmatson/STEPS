use super::*;

/// Provides a method to calculate the size of the population after a given doubling phase
/// and the number of mutants to add at the end of that phase
pub trait GrowthCalculator {
    /// Returns tuple `(new_N, N_mut)` giving the new size and number of descendant mutants for
    /// the `idx`th element of `lineages` after this growth phase
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineage: Lineage,
        cfg: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64);

    /// Double the lineages
    fn double_lineages<R: Rng>(
        &self,
        lineages: Lineages,
        cfg: &SimConfig,
        rng: &mut R,
    ) -> Lineages {
        // Create output vector
        // Reserve extra for more mutants
        // The full size won't be needed
        let mut output = Lineages::with_capacity(2 * lineages.len());

        for lineage in lineages.iter() {
            let (new_N, N_mut) = self.calculate_new_N_and_mutant_count(*lineage, cfg, rng);

            if new_N > 0 {
                output.push(Lineage {
                    N: new_N,
                    ..*lineage
                });
            }

            for _ in 0..N_mut {
                let mutant = new_mutant(*lineage, cfg, rng);
                output.push(mutant);
            }
        }

        output
    }
}

/// Phase 1 doubling, double once and don't bottleneck
pub struct Phase1();

impl Phase1 {
    pub fn estimate_delta_t(lineages: &Lineages, cfg: &SimConfig) -> usize {
        ((cfg.max_pop_size as f64 / lineages.sum_N() as f64).log(lineages.avg_W().exp2()) - 1.0)
            .floor()
            .max(0.0) as usize
    }
}

impl GrowthCalculator for Phase1 {
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineage: Lineage,
        _: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64) {
        // Calculate size after growth
        let N_after_growth = (lineage.W.exp2() * lineage.N as f64).round() as u64;

        // Check number of mutations
        // Sample if mutation rate is positive, otherwise no mutations
        let N_mut: u64 = if lineage.U > 0.0 {
            rand_distr::Poisson::new(lineage.U * (N_after_growth - lineage.N) as f64)
                .unwrap()
                .sample(rng)
        } else {
            0
        };

        // Subtract number of mutants to get new size
        let new_N = N_after_growth - N_mut;

        if new_N < lineage.N {
            eprintln!("WARNING: N_mut exceeded amount of new cells");
        }

        (new_N, N_mut)
    }
}

/// Phase 2 doubling, estimate doubling for delta_t times and bottleneck
pub struct Phase2 {
    delta_t: f64,
}

impl Phase2 {
    pub fn new(lineages: &Lineages, cfg: &SimConfig) -> Self {
        let delta_t = (cfg.max_pop_size as f64 / lineages.sum_N() as f64).log2() / lineages.avg_W();
        Phase2 { delta_t }
    }
}

impl GrowthCalculator for Phase2 {
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineage: Lineage,
        cfg: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64) {
        // Determine population size after growth and before bottleneck
        let N_after_growth = ((lineage.W * self.delta_t).exp2() * lineage.N as f64).round() as u64;
        // Bottleneck this population
        let N_bottlenecked = rand_distr::Binomial::new(N_after_growth, cfg.dilution_factor.recip())
            .unwrap()
            .sample(rng);

        if N_bottlenecked == 0 {
            return (0, 0);
        }

        // Estimate how many new mutants survived bottlenecking
        let N_mut = if lineage.U > 0.0 {
            rand_distr::Poisson::new(
                lineage.U
                    * N_bottlenecked as f64
                    * (1.0 - (lineage.W * self.delta_t).exp2().recip()),
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

/// Push a mutant based on `initial_W` and `initial_U` with random mutation type to the end of `output_lineages`
pub fn new_mutant<R: Rng>(parent: Lineage, cfg: &SimConfig, rng: &mut R) -> Lineage {
    let mutation_type = cfg.sample_mutation_type(rng).unwrap();

    let W = match mutation_type {
        MutationType::Beneficial => {
            let alpha = (1.0 + cfg.diminishing_returns_epistasis_strength * (parent.W - 1.0))
                / cfg.initial_beneficial_mutation_size;
            let mutation_size = rand_distr::Exp::new(alpha).unwrap().sample(rng);
            parent.W + mutation_size
        }
        MutationType::Neutral => parent.W,
        MutationType::Deleterious => deleterious_todo(),
        MutationType::MutationRate => mutation_rate_todo(),
    };

    Lineage { N: 1, W, ..parent }
}
