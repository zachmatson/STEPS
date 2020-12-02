//! Types used for storing simulation data

use derive_more::*;
use serde::{Deserialize, Serialize};
use serde_tuple::*;

use super::*;

/// A single lineage with size, fitness, mutation rate, and identifier  
/// Also keeps identifier of parent lineage and initial marker mutation
#[derive(Copy, Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Lineage {
    /// Population size of the lineage
    pub N: f64,
    /// Fitness of the lineage
    pub W: f64,
    /// Total mutation rate of the lineage  
    /// Use to calculate chance of mutations happening,
    /// but defer to relevant `SimConfig` to determine type
    pub U: f64,

    /// Reciprocal of the mean of the beneficial mutation size
    pub lambda: f64,

    /// Unique lineage identifier
    /// (also uniquely identifies the mutation
    /// between the parent and this lineage)
    pub id: u64,
    /// Lineage identifier for the parent
    pub parent_id: u64,
    /// Lineage identifier for the initial neutral marker mutation
    pub marker: u16,
}

/// Container of `Lineage`s with `Vec` like interface which tracks important characteristics  
/// and assigns identifiers
#[derive(Default, Debug, Deref, Serialize, Deserialize)]
pub struct Lineages {
    #[deref]
    /// Actual `Vec` of lineages
    lineages: Vec<Lineage>,

    /// Tracked sum of population size of all lineages stored
    sum_N: f64,
    /// Sum of fitnesses weighted by population size for all lineages stored
    weighted_sum_W: f64,
    /// Sum of population size of all lineages which have marker 1
    sum_N_marker_1: f64,

    #[serde(skip)]
    /// Counter which saves the *last ID* that was assigned
    unique_id_counter: u64,
}

impl Lineages {
    /// Create new instance from `SimConfig`  
    ///
    /// Use this only to start a new replicate. For creating a new container to transfer
    /// into use `Lineages::successor` to ensure that the IDs remain properly numbered
    pub fn from_simconfig(cfg: &SimConfig, mutations_vec: &mut Option<Vec<Mutation>>) -> Self {
        let mut output = Self::default();

        // Size, parent ID, and marker won't matter
        let ancestor = Lineage {
            N: 0.0,
            W: 1.0,
            U: cfg.total_mutation_rate,
            lambda: cfg.initial_beneficial_mutation_size.recip(),
            id: 0,
            parent_id: 0,
            marker: 0,
        };

        // Initialize with a lineage for each marker and a population size of
        // Nmax/D, evenly divided between the markers
        let N = (cfg.max_pop_size as f64 / cfg.dilution_factor / cfg.markers as f64).round();

        // 1 index the markers beacuse "0" ID is reserved for the immediate ancestor of the neutral marker mutations
        for m in 1..=cfg.markers {
            // ID and parent ID will be assigned by push_child so it doesn't matter what we use for them here
            output.push_child(
                &ancestor,
                Lineage {
                    N,
                    marker: m,
                    ..ancestor
                },
                mutations_vec,
            );
        }

        output
    }

    /// Create a new, empty instance from an old instance, which will have a capacity scaled based on
    /// the old instance (currently 2x the length of the old instance) and preserve the
    /// counter used to generate unique IDs.
    ///
    /// This is the proper way to generate a new instance to transfer into from an old instance.  
    /// To start a new replicate, use `Lineages::from_simconfig`
    pub fn successor(old: &Lineages) -> Self {
        Lineages {
            lineages: Vec::with_capacity(2 * old.len()),
            unique_id_counter: old.unique_id_counter,
            ..Default::default()
        }
    }

    /// Push a new `Lineage` to the collection
    pub fn push(&mut self, lineage: Lineage) {
        // Must update internal trackers before pushing
        self.sum_N += lineage.N;
        if lineage.marker == 1 {
            self.sum_N_marker_1 += lineage.N;
        }
        self.weighted_sum_W += lineage.N * lineage.W;
        self.lineages.push(lineage);
    }

    /// Push a new `child` `Lineage` of `Parent` to the collection
    /// Properly assigning its Parent ID, its own ID, and tracking sequencing
    /// information if necessary
    pub fn push_child(
        &mut self,
        parent: &Lineage,
        mut child: Lineage,
        mutations_vec: &mut Option<Vec<Mutation>>,
    ) {
        child.parent_id = parent.id;
        // unique_id_counter stores last assigned ID
        // starting with 0 as the ID of the common ancestor to each marker
        // which is never actually used by any lineage,
        // so must increment *before* using the ID
        self.unique_id_counter += 1;
        child.id = self.unique_id_counter;

        if let Some(mutations_vec) = mutations_vec {
            mutations_vec.push(Mutation {
                id: child.id,
                background_id: parent.id,
                delta_W: child.W - parent.W,
            });
        }

        self.push(child);
    }

    /// Return the total population of all stored lineages
    pub fn sum_N(&self) -> f64 {
        self.sum_N
    }

    /// Return the average fitness of all stored lineages, with proper
    /// weighting by population size
    pub fn avg_W(&self) -> f64 {
        self.weighted_sum_W / self.sum_N
    }

    /// Return the ratio of the population size of all stored lineages with marker 1
    /// to the population size of all other stored lineages
    pub fn marker_1_ratio(&self) -> f64 {
        self.sum_N_marker_1 / (self.sum_N - self.sum_N_marker_1)
    }
}

/// Types of mutations which can occur
#[derive(Debug, Copy, Clone)]
pub enum MutationType {
    /// A mutation increasing fitness
    Beneficial,
    /// A mutation with no effect
    Neutral,
    /// A mutation decreasing fitness
    Deleterious,
    /// A mutation which alters the mutation rate
    MutationRate,
}

#[derive(Debug)]
pub struct Mutation {
    /// ID of the `Mutation` corresponding to the ID of
    /// the first `Lineage` instance with this mutation
    pub id: u64,
    /// ID of the background of the `Mutation` corresponding
    /// to the ID of the *parent* of the first `Lineage`
    /// instance with this mutation
    pub background_id: u64,
    /// Change in fitness as a result of this mutation
    pub delta_W: f64,
}
