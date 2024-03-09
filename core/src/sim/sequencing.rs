//! Implementation of mutation tracking

use itertools::izip;

use crate::sim::types::{LineagesData, Mutation, MutationsData};

/// Update the population sizes of mutations being tracked in `sequencing_data` based on
/// the lineages in `population_data`
///
/// Mutations must already have been registered to be updated, this will not create/register
/// any new mutations
///
/// Calling this function may cause some mutations to become pruned, after which point they will no
/// longer be updated
pub fn update_sizes(sequencing_data: &mut MutationsData, population_data: &LineagesData) {
    let LineagesData { N, secondary, .. } = population_data;
    assert_eq!(N.len(), secondary.len());
    let sum_N: f64 = N.iter().sum();

    let map = &mut sequencing_data.muts;

    // No mutations are "just_updated" now
    // After updating they will be
    for mutation in map.values_mut() {
        mutation.just_updated = false;
    }

    for (N, secondary) in izip!(N, secondary) {
        // Search through background_id's until none is found
        // Indicating that the background mutation has been pruned or is not being tracked
        let mut id = secondary.id;
        while let Some(mutation) = map.get_mut(&id) {
            // Only a newly updated mutation has an N entry for this transfer
            if mutation.just_updated {
                *mutation.N.last_mut().unwrap() += N;
            } else {
                mutation.N.push(*N);
                mutation.just_updated = true;
            }
            id = mutation.background_id;
        }
    }

    // Any mutation which has fixed or gone extinct after having its population
    // size tracked can be pruned
    let prunable = |_: &u64, m: &mut Mutation| {
        !m.just_updated || (*m.N.last().unwrap() - sum_N).abs() < f64::EPSILON
    };
    sequencing_data
        .pruned_muts
        .extend(map.extract_if(prunable).map(|(_, v)| v));
}
