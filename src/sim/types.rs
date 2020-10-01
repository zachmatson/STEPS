use derive_more::*;

use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Lineage {
    /// Population size
    pub N: u64,
    /// Population fitness
    pub W: f64,
    /// Population total mutation rate  
    /// Use to calculate chance of mutations happening,
    /// but defer to general mutation rate to determine type
    pub U: f64,
}

#[derive(Default, Debug, Deref)]
pub struct Lineages {
    #[deref]
    lineages: Vec<Lineage>,
    sum_N: u64,
    weighted_sum_W: f64,
}

impl Lineages {
    pub fn from_simconfig(cfg: &SimConfig) -> Self {
        let mut output = Self::default();
        let N = (cfg.max_pop_size as f64 / cfg.dilution_factor / cfg.markers as f64).round() as u64;
        for _ in 0..cfg.markers {
            output.push(Lineage {
                N,
                W: 1.0,
                U: cfg.total_mutation_rate,
            });
        }

        output
    }

    pub fn with_capacity(n: usize) -> Self {
        Lineages {
            lineages: Vec::with_capacity(n),
            ..Default::default()
        }
    }

    pub fn push(&mut self, lineage: Lineage) {
        self.sum_N += lineage.N;
        self.weighted_sum_W += lineage.N as f64 * lineage.W;
        self.lineages.push(lineage);
    }

    pub fn sum_N(&self) -> u64 {
        self.sum_N
    }

    pub fn avg_W(&self) -> f64 {
        self.weighted_sum_W / self.sum_N as f64
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MutationType {
    Beneficial,
    Neutral,
    Deleterious,
    MutationRate,
}
