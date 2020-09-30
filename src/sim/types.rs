use super::*;

pub struct Lineage {
    N: u64,
    W: f64,
    U: f64,
}

#[derive(Default, Debug)]
pub struct Lineages {
    /// Population size
    pub N: Vec<u64>,
    /// Population fitness
    pub W: Vec<f64>,
    /// Population total mutation rate  
    /// Use to calculate chance of mutations happening,
    /// but defer to general mutation rate to determine type
    pub U: Vec<f64>,
}

impl Lineages {
    pub fn from_simconfig(cfg: &SimConfig) -> Self {
        let initial_n =
            (cfg.max_pop_size as f64 / cfg.dilution_factor / cfg.markers as f64).round() as u64;

        Lineages {
            N: vec![initial_n; cfg.markers as usize],
            W: vec![1.0; cfg.markers as usize],
            U: vec![cfg.total_mutation_rate; cfg.markers as usize],
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        Lineages {
            N: Vec::with_capacity(n),
            W: Vec::with_capacity(n),
            U: Vec::with_capacity(n),
        }
    }

    pub fn reserve(&mut self, n: usize) {
        self.N.reserve(n);
        self.W.reserve(n);
        self.U.reserve(n);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MutationType {
    Beneficial,
    Neutral,
    Deleterious,
    MutationRate,
}

/// Provides a method to calculate the size of the population after a given doubling phase
/// and the number of mutants to add at the end of that phase
pub trait GrowthCalculator {
    /// Returns tuple `(new_N, N_mut)` giving the new size and number of descendant mutants for
    /// the `idx`th element of `lineages` after this growth phase
    fn calculate_new_N_and_mutant_count<R: Rng>(
        &self,
        lineages: &Lineages,
        idx: usize,
        cfg: &SimConfig,
        rng: &mut R,
    ) -> (u64, u64);
}
