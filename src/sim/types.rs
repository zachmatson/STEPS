use super::*;

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
    pub fn new(cfg: &SimConfig) -> Self {
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
