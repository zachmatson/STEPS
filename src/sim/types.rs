//! Types used for storing simulation data

use serde::{Deserialize, Serialize};
use serde_tuple::*;

use super::*;

/// Container for data on a population of lineages
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LineagesData {
    /// Population sizes of lineages
    pub N: Vec<f64>,
    /// Fitnesses of lineages
    pub W: Vec<f64>,
    /// Total mutation rate of lineages
    /// Defer to `SimConfig` for specific rates
    pub U: Vec<f64>,
    /// Additional data in AoS format
    pub secondary: Vec<SecondaryLineageData>,

    #[serde(skip)]
    /// Counter which saves the *last ID* that was assigned
    unique_id_counter: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct Lineage {
    /// Population size
    pub N: f64,
    /// Fitness
    pub W: f64,
    /// Mutation rate
    pub U: f64,
    /// Additional data
    pub secondary: SecondaryLineageData,
}

#[derive(Copy, Clone, Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct SecondaryLineageData {
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

impl LineagesData {
    /// Create new instance from `SimConfig`  
    ///
    /// Use this only to start a new replicate. For creating a new container to transfer
    /// into use `LineagesData::successor` to ensure that the IDs remain properly numbered
    pub fn from_simconfig(cfg: &SimConfig, mutations_vec: &mut Option<Vec<Mutation>>) -> Self {
        let mut output = Self::default();

        // Size, parent ID, and marker won't matter
        let ancestor = Lineage {
            N: 0.0,
            W: 1.0,
            U: cfg.total_mutation_rate,
            secondary: SecondaryLineageData {
                lambda: cfg.initial_beneficial_mutation_size.recip(),
                id: 0,
                parent_id: 0,
                marker: 0,
            },
        };

        // Initialize with a lineage for each marker and a population size of
        // Nmax/D, evenly divided between the markers
        let N = (cfg.max_pop_size as f64 / cfg.dilution_factor / cfg.markers as f64).round();

        // 1 index the markers beacuse "0" ID is reserved for the immediate ancestor of the neutral marker mutations
        for m in 1..=cfg.markers {
            // ID and parent ID will be assigned by push_child so it doesn't matter what we use for them here
            let marker_mutant = Lineage {
                N,
                secondary: SecondaryLineageData {
                    marker: m,
                    ..ancestor.secondary
                },
                ..ancestor
            };

            output.push_child(marker_mutant, ancestor, mutations_vec);
        }

        output
    }

    /// Create a new, empty instance from an old instance, which will have a capacity scaled based on
    /// the old instance (currently 1x the length of the old instance) and preserve the
    /// counter used to generate unique IDs.
    ///
    /// This is the proper way to generate a new instance to transfer into from an old instance.  
    /// To start a new replicate, use `LineagesData::from_simconfig`
    pub fn successor(old: &LineagesData) -> Self {
        let mut new = LineagesData::default();
        new.reserve(old.N.len());
        new.unique_id_counter = old.unique_id_counter;

        new
    }

    /// Push a new `Lineage` to the collection
    pub fn push(&mut self, data: Lineage) {
        self.N.push(data.N);
        self.W.push(data.W);
        self.U.push(data.U);
        self.secondary.push(data.secondary);
    }

    /// Push a new `child` `Lineage` of `parent` to the collection
    /// Properly assigning its Parent ID, its own ID, and tracking sequencing
    /// information if necessary
    pub fn push_child(
        &mut self,
        mut child: Lineage,
        parent: Lineage,
        mutations_vec: &mut Option<Vec<Mutation>>,
    ) {
        // Appropriate parent_id must be assigned
        child.secondary.parent_id = parent.secondary.id;
        // unique_id_counter stores last assigned ID
        // starting with 0 as the ID of the common ancestor to each marker
        // which is never actually used by any lineage,
        // so must increment *before* using the ID
        self.unique_id_counter += 1;
        child.secondary.id = self.unique_id_counter;

        self.push(child);

        if let Some(mutations_vec) = mutations_vec {
            mutations_vec.push(Mutation {
                id: child.secondary.id,
                background_id: parent.secondary.id,
                delta_W: child.W - parent.W,
            });
        }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> Lineage {
        Lineage {
            N: *self.N.get_unchecked(index),
            W: *self.W.get_unchecked(index),
            U: *self.U.get_unchecked(index),
            secondary: *self.secondary.get_unchecked(index),
        }
    }

    fn reserve(&mut self, additional: usize) {
        self.N.reserve(additional);
        self.W.reserve(additional);
        self.U.reserve(additional);
        self.secondary.reserve(additional);
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
