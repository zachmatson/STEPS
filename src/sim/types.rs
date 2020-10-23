use derive_more::*;
use serde::Serialize;
use serde_tuple::*;

use super::*;

#[derive(Copy, Clone, Debug, Serialize_tuple)]
pub struct Lineage {
    /// Population size
    pub N: u64,
    /// Population fitness
    pub W: f64,
    /// Population total mutation rate  
    /// Use to calculate chance of mutations happening,
    /// but defer to general mutation rate to determine type
    pub U: f64,

    /// Unique lineage identifier
    /// (also uniquely identifies the mutation
    /// between the parent and this lineage)
    pub id: u64,
    /// Lineage identifier for the parent
    pub parent_id: u64,

    /// Lineage identifier for the initial neutral marker mutation
    pub marker: u16,
}

#[derive(Default, Debug, Deref, Serialize)]
pub struct Lineages {
    #[deref]
    lineages: Vec<Lineage>,
    sum_N: u64,
    weighted_sum_W: f64,
    sum_N_marker_1: u64,
    #[serde(skip)]
    unique_id_counter: u64,
}

impl Lineages {
    pub fn from_simconfig(cfg: &SimConfig) -> Self {
        let mut output = Self::default();
        let N = (cfg.max_pop_size as f64 / cfg.dilution_factor / cfg.markers as f64).round() as u64;
        for m in 1..=cfg.markers {
            output.push_child(Lineage {
                N,
                W: 1.0,
                U: cfg.total_mutation_rate,
                id: 0,
                parent_id: 0,
                marker: m,
            });
        }

        output
    }

    pub fn successor(old: &Lineages) -> Self {
        Lineages {
            lineages: Vec::with_capacity(2 * old.len()),
            unique_id_counter: old.unique_id_counter,
            ..Default::default()
        }
    }

    pub fn push(&mut self, lineage: Lineage) {
        self.sum_N += lineage.N;
        if lineage.marker == 1 {
            self.sum_N_marker_1 += lineage.N;
        }
        self.weighted_sum_W += lineage.N as f64 * lineage.W;
        self.lineages.push(lineage);
    }

    pub fn push_child(&mut self, mut lineage: Lineage) {
        lineage.parent_id = lineage.id;
        self.unique_id_counter += 1;
        lineage.id = self.unique_id_counter;
        self.push(lineage);
    }

    pub fn sum_N(&self) -> u64 {
        self.sum_N
    }

    pub fn avg_W(&self) -> f64 {
        self.weighted_sum_W / self.sum_N as f64
    }

    pub fn marker_1_ratio(&self) -> f64 {
        self.sum_N_marker_1 as f64 / (self.sum_N - self.sum_N_marker_1) as f64
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MutationType {
    Beneficial,
    Neutral,
    Deleterious,
    MutationRate,
}
