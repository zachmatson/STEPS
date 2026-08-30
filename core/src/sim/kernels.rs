//! Performance sensitive and optimized computational kernels for the simulations  
//!
//! Includes lower-level implementation details of the transfer process

use std::ops::Mul;

use itertools::izip;
use slices_dispatch_wide::slices_dispatch_wide;

use crate::sim::types::LineagesData;

/// Grow the lineages `delta_t` time forward in place
///
/// Uses formula `N_new = (N_old * (W * delta_t).exp2())`
pub fn grow_lineages_inplace(lineages: &mut LineagesData, delta_t: f64) {
    assert_eq!(lineages.N.len(), lineages.W.len());

    let delta_t_scaled = delta_t * 2f64.ln();
    slices_dispatch_wide!(4, |lineages.N => original_N mut: f64, lineages.W => W: f64| {
        original_N *= W.mul(delta_t_scaled).exp();
    });
}

/// Convert a slice of pre-growth population sizes to a slice of population changes
/// due to growth, where `lineages` are the same lineages *after* growth
///
/// The population increases will be stored directly in the existing `old_N`, and the mutable
/// reference to this slice will be returned, preventing the reuse of the old reference
pub fn old_N_to_delta_N<'a>(lineages: &LineagesData, old_N: &'a mut [f64]) -> &'a mut [f64] {
    assert_eq!(lineages.N.len(), old_N.len());

    for (old_N, N) in izip!(old_N.iter_mut(), &lineages.N) {
        *old_N = N - *old_N;
    }

    old_N
}

/// Get the expected number of mutations for each lineage as a newly allocated
/// `Vec`, given the `lineages` and a slice of the number of individuals in each lineage
/// eligible to mutate
pub fn expected_mutation_counts(lineages: &LineagesData, eligible_N: &[f64]) -> Vec<f64> {
    assert_eq!(lineages.U.len(), eligible_N.len());

    izip!(&lineages.U, eligible_N.iter())
        .map(|(u, n)| u * n * 2.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::types::SecondaryLineageData;
    use approx::assert_relative_eq;

    fn make_lineages(data: &[(f64, f64, f64)]) -> LineagesData {
        let mut lineages = LineagesData::default();
        for &(n, w, u) in data {
            lineages.N.push(n);
            lineages.W.push(w);
            lineages.U.push(u);
            lineages.secondary.push(SecondaryLineageData::default());
        }
        lineages
    }

    #[test]
    fn test_grow_lineages_uniform_fitness() {
        // W=1.0, delta_t=1.0 → N doubles: N_new = N * 2^(1.0*1.0) = 2*N
        let mut lineages = make_lineages(&[(100.0, 1.0, 0.0), (200.0, 1.0, 0.0)]);
        grow_lineages_inplace(&mut lineages, 1.0);
        assert_relative_eq!(lineages.N[0], 200.0, epsilon = 1e-10);
        assert_relative_eq!(lineages.N[1], 400.0, epsilon = 1e-10);
    }

    #[test]
    fn test_grow_lineages_higher_fitness_grows_faster() {
        // W=2.0, delta_t=1.0 → N_new = N * 2^(2.0*1.0) = 4*N
        let mut lineages = make_lineages(&[(100.0, 2.0, 0.0)]);
        grow_lineages_inplace(&mut lineages, 1.0);
        assert_relative_eq!(lineages.N[0], 400.0, epsilon = 1e-10);
    }

    #[test]
    fn test_grow_lineages_fractional_delta_t() {
        // W=1.0, delta_t=0.5 → N_new = N * 2^0.5 = N * sqrt(2)
        let mut lineages = make_lineages(&[(100.0, 1.0, 0.0)]);
        grow_lineages_inplace(&mut lineages, 0.5);
        assert_relative_eq!(lineages.N[0], 100.0 * 2.0_f64.sqrt(), epsilon = 1e-10);
    }

    #[test]
    fn test_grow_lineages_zero_delta_t() {
        // delta_t=0 → no growth
        let mut lineages = make_lineages(&[(100.0, 1.5, 0.0)]);
        grow_lineages_inplace(&mut lineages, 0.0);
        assert_relative_eq!(lineages.N[0], 100.0, epsilon = 1e-10);
    }

    #[test]
    fn test_old_n_to_delta_n() {
        let lineages = make_lineages(&[(200.0, 1.0, 0.0), (500.0, 1.0, 0.0)]);
        let mut old_n = vec![100.0, 300.0];
        let delta_n = old_N_to_delta_N(&lineages, &mut old_n);
        assert_relative_eq!(delta_n[0], 100.0, epsilon = 1e-10);
        assert_relative_eq!(delta_n[1], 200.0, epsilon = 1e-10);
    }

    #[test]
    fn test_old_n_to_delta_n_no_growth() {
        let lineages = make_lineages(&[(100.0, 1.0, 0.0)]);
        let mut old_n = vec![100.0];
        let delta_n = old_N_to_delta_N(&lineages, &mut old_n);
        assert_relative_eq!(delta_n[0], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_expected_mutation_counts_basic() {
        // U=0.001, eligible_N=1000 → expected = 0.001 * 1000 * 2 = 2.0
        let lineages = make_lineages(&[(0.0, 1.0, 0.001)]);
        let eligible = vec![1000.0];
        let counts = expected_mutation_counts(&lineages, &eligible);
        assert_relative_eq!(counts[0], 2.0, epsilon = 1e-10);
    }

    #[test]
    fn test_expected_mutation_counts_zero_rate() {
        let lineages = make_lineages(&[(0.0, 1.0, 0.0)]);
        let eligible = vec![500.0];
        let counts = expected_mutation_counts(&lineages, &eligible);
        assert_relative_eq!(counts[0], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_expected_mutation_counts_multiple() {
        let lineages = make_lineages(&[(0.0, 1.0, 0.01), (0.0, 1.0, 0.02)]);
        let eligible = vec![100.0, 200.0];
        let counts = expected_mutation_counts(&lineages, &eligible);
        // 0.01 * 100 * 2 = 2.0
        assert_relative_eq!(counts[0], 2.0, epsilon = 1e-10);
        // 0.02 * 200 * 2 = 8.0
        assert_relative_eq!(counts[1], 8.0, epsilon = 1e-10);
    }

    #[test]
    #[should_panic]
    fn test_grow_lineages_mismatched_lengths_panics() {
        let mut lineages = LineagesData::default();
        lineages.N.push(100.0);
        // W is empty — mismatch
        grow_lineages_inplace(&mut lineages, 1.0);
    }

    #[test]
    #[should_panic]
    fn test_expected_mutation_counts_mismatched_lengths_panics() {
        let lineages = make_lineages(&[(100.0, 1.0, 0.01)]);
        let eligible = vec![100.0, 200.0]; // wrong length
        expected_mutation_counts(&lineages, &eligible);
    }
}
