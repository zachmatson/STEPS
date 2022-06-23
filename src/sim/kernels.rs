//! Performance sensitive and optimized computational kernels for the simulations  
//! Includes lower-level implementation details of the transfer process

use std::ops::Mul;

use itertools::izip;
use slices_dispatch_wide::slices_dispatch_wide;

use super::LineagesData;

/// Grow the lineages `delta_t` time forward, using the fitnesses and starting
/// sizes in `data`, and leaving the resulting grown lineages inside `data`  
/// Uses formula `N_new = (N_old * (W * delta_t).exp2())`
///
/// The length of any vector in `data` will *not* be changed
pub fn grow_lineages_inplace(data: &mut LineagesData, delta_t: f64) {
    assert_eq!(data.N.len(), data.W.len());

    let delta_t_scaled = delta_t * 2f64.ln();
    slices_dispatch_wide!(4, |data.N => original_W mut: f64, data.W => W: f64| {
        original_W *= W.mul(delta_t_scaled).exp();
    });
}

/// Convert a slice of pre-growth population sizes to a slice of population changes
/// due to growth, where `data` is the same lineages *after* growth
///
/// The population increases will be stored directly in the existing `old_N`, and the mutable
/// reference to this slice will be returned, preventing the reuse of the old reference
pub fn old_N_to_delta_N<'a>(data: &LineagesData, old_N: &'a mut [f64]) -> &'a mut [f64] {
    assert_eq!(data.N.len(), old_N.len());

    for (old_N, N) in izip!(old_N.iter_mut(), &data.N) {
        *old_N = N - *old_N;
    }

    old_N
}

/// Get the expected number of mutations for each lineage as a newly allocated
/// `Vec`, given the `data` containing the lineages and a slice `delta_N` of the
/// number of individuals in each lineage eligible to mutate
pub fn expected_mutation_counts(data: &LineagesData, delta_N: &[f64]) -> Vec<f64> {
    assert_eq!(data.U.len(), delta_N.len());

    izip!(&data.U, delta_N.iter()).map(|(u, n)| u * n).collect()
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_grow_lineages_inplace() {
        let mut data = LineagesData::default();
        data.N = vec![1.0, 2.0, 3.0, 4.0, 1e6, 2e3, 3e4];
        data.W = vec![1.1, 1.2, 1.4, 1.3, 1.5, 2.0, 0.9];

        grow_lineages_inplace(&mut data, 1.5);

        let expected = vec![
            3.138336391587003,
            6.964404506368992,
            12.861281550435514,
            15.454981262797531,
            4.756828460010884e6,
            16000.0,
            76473.63763915573,
        ];
        for (a, b) in izip!(expected, data.N) {
            assert_relative_eq!(a, b);
        }
    }

    #[test]
    fn test_old_N_to_delta_N() {
        let mut new_data = LineagesData::default();
        new_data.N = vec![1.0, 2.0, 3.0, 4.0, 1e6, 2e3, 3e4];
        let mut old_N = vec![0.0, 1.0, 1.0, 2.0, 5e5, 500.0, 1e4];

        old_N_to_delta_N(&mut new_data, &mut old_N);

        let expected = vec![1.0, 1.0, 2.0, 2.0, 5e5, 1_500.0, 2e4];
        for (a, b) in izip!(expected, old_N) {
            assert_relative_eq!(a, b);
        }
    }

    #[test]
    fn test_expected_mutation_counts() {
        let mut data = LineagesData::default();
        let delta_N = vec![1.0, 2.0, 3.0, 4.0, 1e6, 2e3, 3e4];
        data.U = vec![1e-6, 1e-8, 1e-4, 1e-5, 2.0, 0.5, 1e-2];

        let result = expected_mutation_counts(&data, &delta_N);

        let expected = vec![1e-6, 2e-8, 3e-4, 4e-5, 2e6, 1e3, 3e2];
        for (a, b) in izip!(expected, result) {
            assert_relative_eq!(a, b);
        }
    }
}
