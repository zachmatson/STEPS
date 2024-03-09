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
