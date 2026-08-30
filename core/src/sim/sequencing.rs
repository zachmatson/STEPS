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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::types::SecondaryLineageData;

    fn make_lineages_with_ids(data: &[(f64, u64)]) -> LineagesData {
        let mut lineages = LineagesData::default();
        for &(n, id) in data {
            lineages.N.push(n);
            lineages.W.push(1.0);
            lineages.U.push(0.0);
            lineages.secondary.push(SecondaryLineageData {
                lambda: 0.0,
                id,
                parent_id: 0,
                marker: 1,
                accumulated_muts: 1,
            });
        }
        lineages
    }

    fn register_mutation(
        mutations: &mut MutationsData,
        id: u64,
        background_id: u64,
        transfer: u32,
    ) {
        mutations.muts.insert(
            id,
            Mutation {
                id,
                background_id,
                delta_W: 0.05,
                delta_U: 0.0,
                first_transfer: transfer,
                N: Vec::new(),
                order: 1,
                just_updated: false,
            },
        );
    }

    #[test]
    fn test_update_sizes_adds_population() {
        let mut mutations = MutationsData::new();
        register_mutation(&mut mutations, 1, 0, 0);

        // Two lineages: id=1 has the mutation, id=99 does not
        // This prevents mutation 1 from being pruned as "fixed" (N < sum_N)
        let lineages = make_lineages_with_ids(&[(100.0, 1), (200.0, 99)]);
        update_sizes(&mut mutations, &lineages);

        let mutation = mutations.muts.get(&1).unwrap();
        assert_eq!(mutation.N.len(), 1);
        assert_eq!(mutation.N[0], 100.0);
    }

    #[test]
    fn test_update_sizes_accumulates_across_lineages() {
        let mut mutations = MutationsData::new();
        // Mutation 1 is background to mutation 2
        register_mutation(&mut mutations, 1, 0, 0);
        register_mutation(&mut mutations, 2, 1, 1);

        // Lineage id=2 carries mutation 2 (backgrounds to 1)
        // Lineage id=99 carries neither — prevents fixation pruning
        let lineages = make_lineages_with_ids(&[(50.0, 2), (100.0, 99)]);
        update_sizes(&mut mutations, &lineages);

        // Mutation 2 gets N=50 directly
        assert_eq!(mutations.muts.get(&2).unwrap().N[0], 50.0);
        // Mutation 1 also gets N=50 (because lineage 2 carries mutation 1 in its background)
        assert_eq!(mutations.muts.get(&1).unwrap().N[0], 50.0);
    }

    #[test]
    fn test_update_sizes_prunes_extinct_mutation() {
        let mut mutations = MutationsData::new();
        register_mutation(&mut mutations, 1, 0, 0);

        // No lineage carries mutation 1 → it's extinct
        let lineages = make_lineages_with_ids(&[(100.0, 99)]);
        update_sizes(&mut mutations, &lineages);

        // Mutation 1 should be pruned (not updated = extinct)
        assert!(!mutations.muts.contains_key(&1));
        assert_eq!(mutations.pruned_muts.len(), 1);
        assert_eq!(mutations.pruned_muts[0].id, 1);
    }

    #[test]
    fn test_update_sizes_prunes_fixed_mutation() {
        let mut mutations = MutationsData::new();
        register_mutation(&mut mutations, 1, 0, 0);

        // One lineage with id=1 that IS the entire population
        // If mutation N == sum_N, it's fixed
        let lineages = make_lineages_with_ids(&[(500.0, 1)]);
        update_sizes(&mut mutations, &lineages);

        // sum_N = 500, mutation N = 500 → fixed → pruned
        assert!(!mutations.muts.contains_key(&1));
        assert_eq!(mutations.pruned_muts.len(), 1);
    }

    #[test]
    fn test_update_sizes_multiple_transfers() {
        let mut mutations = MutationsData::new();
        register_mutation(&mut mutations, 1, 0, 0);

        // First update
        let lineages = make_lineages_with_ids(&[(50.0, 1), (50.0, 2)]);
        update_sizes(&mut mutations, &lineages);

        // Reset just_updated for next round
        for m in mutations.muts.values_mut() {
            m.just_updated = false;
        }

        // Second update with different population
        let lineages = make_lineages_with_ids(&[(75.0, 1), (25.0, 2)]);
        update_sizes(&mut mutations, &lineages);

        let mutation = mutations.muts.get(&1).unwrap();
        assert_eq!(mutation.N.len(), 2);
        assert_eq!(mutation.N[0], 50.0);
        assert_eq!(mutation.N[1], 75.0);
    }
}
