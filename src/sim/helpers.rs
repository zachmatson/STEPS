//! Helper structs, traits, and functions for doubling lineages and adding mutants  
//! Finer implementation details of the transfer process

use super::*;

/// Trait for types that can perform a doubling or approximate multiple doublings for an instance of `Lineages`  
/// Provides default implementation of doubling for implementors, which must provide a function to calculate
/// growth and mutant additions
pub trait GrowthCalculator {
    /// Returns tuple `(new_N, N_mut)` giving the new size and number of descendant mutants to add for
    /// the `lineage` after a growth cycle
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineage: Lineage,
        cfg: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64);

    /// Grow the lineages and add necessary mutants
    fn grow_lineages<R: Rng>(
        &self,
        lineages: Lineages,
        cfg: &SimConfig,
        rng: &mut R,
        mutations_vec: &mut Option<Vec<Mutation>>,
    ) -> Lineages {
        // Successor `Lineages` struct will be empty with appropriate capacity and
        // identifier counter for adding the lineages after this growth
        let mut output = Lineages::successor(&lineages);

        for lineage in lineages.iter() {
            let (new_N, N_mut) = self.calculate_new_N_and_mutant_count(*lineage, cfg, rng);

            // Depending on bottlenecking behavior, the original lineage may have new size zero
            // and an empty lineage should be dropped
            if new_N > 0 {
                output.push(Lineage {
                    N: new_N,
                    ..*lineage
                });
            }

            for _ in 0..N_mut {
                let mutant = new_mutant(*lineage, cfg, rng);
                output.push_child(lineage, mutant, mutations_vec);
            }
        }

        output
    }
}

/// Get the population size of a `Lineage` after growing for `delta_t` time
#[inline(always)]
fn calculate_N_after_growth(lineage: Lineage, delta_t: f64) -> u64 {
    ((lineage.W * delta_t).exp2() * lineage.N as f64).ceil() as u64
}

/// Phase 1 doubling  
/// Doubles once with no bottleneck
pub struct Phase1 {
    delta_t: f64,
}

impl Phase1 {
    /// The number of doublings required in Phase 1  
    /// This number does not change as simulations run;
    /// it is fixed for a given set of parameters
    pub fn doublings_required(cfg: &SimConfig) -> usize {
        cfg.dilution_factor.log2().floor() as usize
    }

    /// Create a new instance of `Phase1` which would grow `lineages` enough to
    /// double its current population size
    pub fn new(lineages: &Lineages) -> Self {
        let delta_t = lineages.avg_W().recip();
        Self { delta_t }
    }
}

impl GrowthCalculator for Phase1 {
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineage: Lineage,
        _: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64) {
        // Total population size including new mutants
        let N_after_growth = calculate_N_after_growth(lineage, self.delta_t);

        // Poisson sampling will fail if lambda is not positive
        let N_mut: u64 = if lineage.U > 0.0 {
            fast_distr::poisson(lineage.U * (N_after_growth - lineage.N) as f64, rng)
        } else {
            0
        };

        let new_N = N_after_growth - N_mut;

        // This shouldn't happen
        // Would indicate that N_mut is greater than the number of new cells added during doubling
        if new_N < lineage.N {
            eprintln!("WARNING: N_mut exceeded amount of new cells");
        }

        (new_N, N_mut)
    }
}

/// Phase 2 doubling  
/// Approximates enough doublings to get the population size to around Nmax
pub struct Phase2 {
    delta_t: f64,
}

impl Phase2 {
    /// Create a new instance of `Phase2` which would grow `lineages` enough to bring its total
    /// population size to the Nmax defined in `cfg`
    pub fn new(lineages: &Lineages, cfg: &SimConfig) -> Self {
        // Use approximation N_final = 2^(avg_W * delta_t)*N_initial
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
        // Population size after growth and *before* bottleneck
        let N_after_growth = calculate_N_after_growth(lineage, self.delta_t);
        // Bottlenecked size including mutants
        let N_bottlenecked = rand_distr::Binomial::new(N_after_growth, cfg.dilution_factor.recip())
            .unwrap()
            .sample(rng);

        // Other calculations not needed if no cells survivied bottleneck
        if N_bottlenecked == 0 {
            return (0, 0);
        }

        // Estimate how many new mutants survived bottlenecking
        let N_mut = if lineage.U > 0.0 {
            fast_distr::poisson(
                lineage.U
                    * N_bottlenecked as f64
                    * (1.0 - (lineage.W * self.delta_t).exp2().recip()),
                rng,
            )
        } else {
            0
        };

        let new_N = N_bottlenecked - N_mut;

        (new_N, N_mut)
    }
}

/// Generate a descendant lineage from `parent`
pub fn new_mutant<R: Rng>(parent: Lineage, cfg: &SimConfig, rng: &mut R) -> Lineage {
    let mutation_type = cfg.sample_mutation_type(rng).unwrap();

    let (W, lambda) = match mutation_type {
        MutationType::Beneficial => updates_after_beneficial_mutation(parent, cfg, rng),
        MutationType::Deleterious => updates_after_deleterious_mutation(parent, cfg, rng),
        MutationType::Neutral | MutationType::MutationRate => (parent.W, parent.lambda)
    };

    // let U = match mutation_type {
    //     MutationType::MutationRate => mutation_rate_todo(),
    //     _ => parent.U,
    // };

    Lineage { N: 1, W, lambda, ..parent }
}

/// Generate fitness and mutation size lambda of a descendant of `parent` after undergoing a beneficial mutation
fn updates_after_beneficial_mutation<R: Rng>(parent: Lineage, cfg: &SimConfig, rng: &mut R) -> (f64, f64) {
    let mutation_size = rand_distr::Exp::new(parent.lambda).unwrap().sample(rng);
    let lambda_new = parent.lambda * (1.0 + cfg.diminishing_returns_epistasis_strength * mutation_size);
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
