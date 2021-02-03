use super::*;

pub fn update_frequencies(sequencing_data: &mut MutationsData, population_data: &LineagesData) {
    let length = population_data.N.len();
    let N = &population_data.N[..length];
    let secondary = &population_data.secondary[..length];

    let map = &mut sequencing_data.muts;

    for mutation in map.values_mut() {
        mutation.N.push(0.0);
        mutation.prunable = true;
    }

    for i in 0..length {
        let mut id = secondary[i].id;
        while let Some(mutation) = map.get_mut(&id) {
            *mutation.N.last_mut().unwrap() += N[i];
            mutation.prunable = false;
            id = mutation.background_id;
        }
    }

    map.retain(|_, v| !v.prunable);
}