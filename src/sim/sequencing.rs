use super::*;

pub fn update_frequencies(sequencing_data: &mut MutationsData, population_data: &LineagesData) {
    let length = population_data.N.len();
    let N = &population_data.N[..length];
    let sum_N: f64 = N.iter().sum();
    let secondary = &population_data.secondary[..length];

    let map = &mut sequencing_data.muts;

    for mutation in map.values_mut() {
        mutation.just_updated = false;
    }

    for i in 0..length {
        let mut id = secondary[i].id;
        while let Some(mutation) = map.get_mut(&id) {
            if mutation.just_updated {
                *mutation.N.last_mut().unwrap() += N[i];
            } else {
                mutation.N.push(N[i]);
                mutation.just_updated = true;
            }
            id = mutation.background_id;
        }
    }

    map.retain(|_, m| m.N.len() != 0);
    let prunable = |_: &u64, m: &mut Mutation| !m.just_updated || *m.N.last().unwrap() == sum_N;
    sequencing_data
        .pruned_muts
        .extend(map.drain_filter(prunable).map(|(_, v)| v));
}
