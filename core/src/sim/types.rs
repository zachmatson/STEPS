//! Types used for storing simulation data

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

use crate::sim::InternalSimConfig;

/// Container for data on a population of lineages
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LineagesData {
    /// Population sizes of lineages
    pub(super) N: Vec<f64>,
    /// Fitnesses of lineages
    pub(super) W: Vec<f64>,
    /// Total mutation rates of lineages
    ///
    /// Defer to `SimConfig` for relative rates of specific mutation types
    pub(super) U: Vec<f64>,
    /// Additional data in AoS format
    pub(super) secondary: Vec<SecondaryLineageData>,

    #[serde(skip)]
    /// Counter which saves the *last ID* that was assigned
    unique_id_counter: u64,
}

/// Complete data for a single lineage
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

/// Secondary data for lineages  
///
/// Used for data that is not accessed in vectorized computational kernels, and therefore can be
/// efficiently stored in individual structs
#[derive(Copy, Clone, Debug, Default, Serialize_tuple, Deserialize_tuple)]
pub struct SecondaryLineageData {
    /// Reciprocal of the mean of the beneficial mutation size
    pub lambda: f64,

    /// Unique lineage identifier
    ///
    /// Also uniquely identifies the mutation between the parent and this lineage
    pub id: u64,
    /// Lineage identifier for the parent
    pub parent_id: u64,
    /// Lineage identifier for the initial neutral marker mutation
    pub marker: u16,
    /// Number of accumulated mutations relative to the ancestor mutation (each marker starts at 1)
    pub accumulated_muts: u32,
}

impl LineagesData {
    /// Create new instance from `SimConfig`  
    ///
    /// Use this only to start a new replicate. For creating a new container to transfer
    /// into use `LineagesData::successor` to ensure that the IDs remain properly numbered
    pub(super) fn for_sim_config(
        cfg: &InternalSimConfig,
        mutations: &mut Option<MutationsData>,
    ) -> Self {
        let mut output = Self::default();

        // Size, parent ID, and marker won't matter
        let ancestor = Lineage {
            N: 0.0,
            // W and U may be used for comparison to the markers in the case of mutation tracking
            W: 1.0,
            U: cfg.total_mutation_rate,
            secondary: SecondaryLineageData {
                // Lambda will be carried over to the children
                lambda: cfg.inner.initial_beneficial_mutation_size.recip(),
                id: 0,
                parent_id: 0,
                marker: 0,
                // accumulated_muts is incremented for each child
                accumulated_muts: 0,
            },
        };

        // Initialize with a lineage for each marker and a population size of Nmax/D, evenly divided
        // between the markers
        let N =
            (cfg.inner.max_pop_size * cfg.dilution_coefficient / cfg.inner.markers as f64).round();

        // 1 index the markers beacuse "0" ID is reserved for the immediate ancestor of the neutral
        // marker mutations
        for m in 1..=cfg.inner.markers {
            // ID, parent ID, and accumulated muts will be assigned by push_child so it doesn't
            // matter what we use for them here
            let marker_mutant = Lineage {
                N,
                secondary: SecondaryLineageData {
                    marker: m,
                    ..ancestor.secondary
                },
                ..ancestor
            };

            output.push_child(marker_mutant, ancestor, 1, mutations);
        }

        output
    }

    /// Reserve additional capacity in all of the vectors being used
    fn reserve(&mut self, additional: usize) {
        self.N.reserve(additional);
        self.W.reserve(additional);
        self.U.reserve(additional);
        self.secondary.reserve(additional);
    }

    /// Create a new, empty instance from an old instance, which will have a capacity scaled based on
    /// the old instance (currently 1x the length of the old instance) and preserve the
    /// counter used to generate unique IDs.
    ///
    /// This is the proper way to generate a new instance to move lineages into from an old instance
    /// when bottlenecking. To start a new replicate, use `LineagesData::for_sim_config`
    pub(super) fn successor(old: &LineagesData) -> Self {
        let mut new = LineagesData {
            unique_id_counter: old.unique_id_counter,
            ..LineagesData::default()
        };
        new.reserve(old.N.len());
        new
    }

    /// Push a new `Lineage` to the collection
    pub(super) fn push(&mut self, data: Lineage) {
        self.N.push(data.N);
        self.W.push(data.W);
        self.U.push(data.U);
        self.secondary.push(data.secondary);
    }

    /// Push a new `child` `Lineage` of `parent` to the collection, properly assigning its Parent
    /// ID, its own ID, and registering the mutation with the `MutationsData` if applicable
    pub(super) fn push_child(
        &mut self,
        mut child: Lineage,
        parent: Lineage,
        mutation_order: u32,
        mutations: &mut Option<MutationsData>,
    ) {
        // Appropriate parent_id must be assigned
        child.secondary.parent_id = parent.secondary.id;
        // unique_id_counter stores last assigned ID
        // starting with 0 as the ID of the common ancestor to each marker
        // which is never actually used by any lineage,
        // so must increment *before* using the ID
        self.unique_id_counter += 1;
        child.secondary.id = self.unique_id_counter;
        child.secondary.accumulated_muts = parent.secondary.accumulated_muts + mutation_order;

        self.push(child);

        if let Some(mutations) = mutations {
            mutations.register(child, parent, mutation_order);
        }
    }

    /// Access a `Lineage` from the collection, without performing a bounds check
    ///
    /// # Safety
    /// Calling with an index which is out of bounds for any of the component vectors
    /// is undefined behavior. `LineagesData::assert_len_ge` can be used to ensure minimum
    /// size across all vectors
    pub(super) unsafe fn get_unchecked(&self, index: usize) -> Lineage {
        Lineage {
            N: *self.N.get_unchecked(index),
            W: *self.W.get_unchecked(index),
            U: *self.U.get_unchecked(index),
            secondary: *self.secondary.get_unchecked(index),
        }
    }

    /// Asserts that the length of all component vectors is equal to `len`
    ///
    /// # Panics
    /// Panics if any of the component vectors have lengths different than `len`
    pub(super) fn assert_len_eq(&self, len: usize) {
        assert_eq!(self.N.len(), len);
        assert_eq!(self.W.len(), len);
        assert_eq!(self.U.len(), len);
        assert_eq!(self.secondary.len(), len);
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
}

/// Data on a set of `Mutation`s being sequenced  
///
/// To use when sequencing, you must call the `register`
/// method every time a new mutation you want to track
/// arises, so that mutation's information will be stored
///
/// You must also call `set_transfer` after each
/// transfer to have meaningful data about the transfer
/// times each mutation occurred at
#[derive(Debug, Default)]
pub struct MutationsData {
    /// Mutations which are being actively tracked, keyed by their IDs
    pub(crate) muts: HashMap<u64, Mutation>,
    /// Mutations which have been pruned, in arbitrary order
    pub(crate) pruned_muts: Vec<Mutation>,
    /// Transfer the simulations are currently on
    on_transfer: u32,
}

impl MutationsData {
    /// Create a new empty `MutationsData` instance
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Increment the transfer the mutation data is being called for
    ///
    /// Must be called every time transfer changes to get correct results
    pub(super) fn set_transfer(&mut self, transfer: u32) {
        self.on_transfer = transfer;
    }

    /// Register a new `child` `Lineage` by calculating the `Mutation` from its `parent`
    pub(super) fn register(&mut self, child: Lineage, parent: Lineage, mutation_order: u32) {
        let mutation = Mutation {
            id: child.secondary.id,
            background_id: parent.secondary.id,
            delta_W: (child.W / parent.W) - 1.0,
            delta_U: 0.0,
            first_transfer: self.on_transfer,
            N: Vec::with_capacity(0),
            order: mutation_order,
            just_updated: false,
        };

        self.muts.insert(child.secondary.id, mutation);
    }
}

/// Data for one Mutation being tracked
#[derive(Debug, Serialize_tuple)]
pub struct Mutation {
    /// ID of the `Mutation`
    ///
    /// Corresponds to the ID of the first `Lineage` instance with this mutation
    pub id: u64,
    /// ID of the background of the `Mutation`
    ///
    /// Corresponds to the ID of the *parent* of the first `Lineage` instance with this mutation
    pub background_id: u64,
    /// Multiplicative change in fitness as a result of this mutation
    pub delta_W: f64,
    /// Multiplicative change in mutation rate as a result of this mutation
    pub delta_U: f64,
    /// The first transfer at which this mutation appeared
    ///
    /// This is also the transfer corresponding to the first entry in the vector of population sizes
    pub first_transfer: u32,
    /// Vector of population sizes for each transfer tracked starting from `self.first_transfer`
    pub N: Vec<f64>,
    /// Number of mutations this record represents
    pub order: u32,
    /// Was the mutation just updated in the last round of updating sizes?
    #[serde(skip)]
    pub(crate) just_updated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parent() -> Lineage {
        Lineage {
            N: 100.0,
            W: 1.0,
            U: 0.001,
            secondary: SecondaryLineageData {
                lambda: 83.33,
                id: 1,
                parent_id: 0,
                marker: 1,
                accumulated_muts: 1,
            },
        }
    }

    #[test]
    fn test_lineages_data_push() {
        let mut data = LineagesData::default();
        let lineage = make_parent();
        data.push(lineage);
        assert_eq!(data.N.len(), 1);
        assert_eq!(data.W.len(), 1);
        assert_eq!(data.U.len(), 1);
        assert_eq!(data.secondary.len(), 1);
        assert_eq!(data.N[0], 100.0);
        assert_eq!(data.W[0], 1.0);
    }

    #[test]
    fn test_lineages_data_push_child_assigns_ids() {
        let mut data = LineagesData::default();
        let parent = make_parent();
        data.push(parent);
        // unique_id_counter is 0 after push (push doesn't increment it)
        // push_child increments counter before assigning, so first child gets id=1

        let child = Lineage {
            N: 1.0,
            W: 1.05,
            U: 0.001,
            secondary: SecondaryLineageData {
                lambda: 83.33,
                id: 0,        // will be overwritten
                parent_id: 0, // will be overwritten
                marker: 1,
                accumulated_muts: 0, // will be overwritten
            },
        };

        data.push_child(child, parent, 1, &mut None);

        assert_eq!(data.secondary[1].parent_id, 1); // parent's id field
        assert_eq!(data.secondary[1].id, 1); // counter was 0, incremented to 1
        assert_eq!(data.secondary[1].accumulated_muts, 2); // parent's 1 + order 1
    }

    #[test]
    fn test_lineages_data_push_child_increments_counter() {
        let mut data = LineagesData::default();
        let parent = make_parent();
        data.push(parent);

        let child1 = Lineage { N: 1.0, ..parent };
        let child2 = Lineage { N: 1.0, ..parent };
        data.push_child(child1, parent, 1, &mut None);
        data.push_child(child2, parent, 1, &mut None);

        assert_eq!(data.secondary[1].id, 1);
        assert_eq!(data.secondary[2].id, 2);
    }

    #[test]
    fn test_lineages_data_successor_preserves_counter() {
        let mut data = LineagesData::default();
        let parent = make_parent();
        data.push(parent);
        let child = Lineage { N: 1.0, ..parent };
        data.push_child(child, parent, 1, &mut None);

        let successor = LineagesData::successor(&data);
        assert_eq!(successor.unique_id_counter, data.unique_id_counter);
        assert_eq!(successor.N.len(), 0);
    }

    #[test]
    fn test_lineages_data_default_empty() {
        let data = LineagesData::default();
        assert_eq!(data.N.len(), 0);
        assert_eq!(data.W.len(), 0);
        assert_eq!(data.U.len(), 0);
        assert_eq!(data.secondary.len(), 0);
        assert_eq!(data.unique_id_counter, 0);
    }

    #[test]
    fn test_get_unchecked() {
        let mut data = LineagesData::default();
        data.push(Lineage {
            N: 50.0,
            W: 1.2,
            U: 0.005,
            secondary: SecondaryLineageData {
                lambda: 10.0,
                id: 7,
                parent_id: 3,
                marker: 2,
                accumulated_muts: 4,
            },
        });
        data.assert_len_eq(1);
        let lineage = unsafe { data.get_unchecked(0) };
        assert_eq!(lineage.N, 50.0);
        assert_eq!(lineage.W, 1.2);
        assert_eq!(lineage.U, 0.005);
        assert_eq!(lineage.secondary.id, 7);
        assert_eq!(lineage.secondary.marker, 2);
    }

    #[test]
    #[should_panic]
    fn test_assert_len_eq_panics_on_mismatch() {
        let mut data = LineagesData::default();
        data.N.push(1.0);
        // Other vecs are empty
        data.assert_len_eq(1);
    }

    #[test]
    fn test_mutations_data_register() {
        let mut mutations = MutationsData::new();
        mutations.set_transfer(5);

        let parent = make_parent();
        let child = Lineage {
            N: 1.0,
            W: 1.05,
            U: 0.001,
            secondary: SecondaryLineageData {
                lambda: 83.33,
                id: 2,
                parent_id: 1,
                marker: 1,
                accumulated_muts: 2,
            },
        };

        mutations.register(child, parent, 1);
        assert_eq!(mutations.muts.len(), 1);
        let mutation = mutations.muts.get(&2).unwrap();
        assert_eq!(mutation.id, 2);
        assert_eq!(mutation.background_id, 1);
        assert_eq!(mutation.first_transfer, 5);
        assert_eq!(mutation.order, 1);
        // delta_W = (1.05/1.0) - 1.0 = 0.05
        assert!((mutation.delta_W - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_mutations_data_set_transfer() {
        let mut mutations = MutationsData::new();
        mutations.set_transfer(10);

        let parent = make_parent();
        let child = Lineage {
            N: 1.0,
            W: 1.1,
            secondary: SecondaryLineageData {
                id: 5,
                ..parent.secondary
            },
            ..parent
        };
        mutations.register(child, parent, 1);
        assert_eq!(mutations.muts.get(&5).unwrap().first_transfer, 10);
    }
}
