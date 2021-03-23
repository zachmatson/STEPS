//! Performance sensitive and optimized computational kernels for the simulations  
//! Transform or summarize the data in a `LineagesData`  
//! Lower-level implementation details of the transfer process  

use itertools::izip;

use super::LineagesData;

/// Grow the lineages `delta_t` time forward, using the fitnesses and starting
/// sizes in `data`, and leaving the resulting grown lineages inside `data`  
/// Uses formula `N_new = (N_old * (W * delta_t).exp2()).ceil()`
///
/// The length of any vector in `data` will *not* be changed
pub fn grow_lineages_inplace(data: &mut LineagesData, delta_t: f64) {
    extern "C" {
        /// Defined in kernels.c
        ///
        /// Explicitly vectorized where AVX2 is available
        fn grow_lineages_inplace_c(
            len: cty::size_t,
            N: *mut cty::c_double,
            W: *const cty::c_double,
            delta_t: cty::c_double,
        );
    }

    // Bounds check now to make sure the C code will be safe
    let len = data.N.len();
    assert_eq!(len, data.W.len());
    unsafe {
        grow_lineages_inplace_c(len, data.N.as_mut_ptr(), data.W.as_ptr(), delta_t);
    }
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

    data.U.iter().zip(delta_N).map(|(u, n)| u * n).collect()
}

/// Get the total population size and arithmetic mean fitness
/// of all of the lineages in `data`
///
/// Return format is `(sum_N, avg_W)`
pub fn sum_N_and_avg_W(data: &LineagesData) -> (f64, f64) {
    assert_eq!(data.N.len(), data.W.len());

    let mut sum_N = 0.0;
    let mut weighted_sum_W = 0.0;

    for (n, w) in izip!(&data.N, &data.W) {
        sum_N += n;
        weighted_sum_W += n * w;
    }

    (sum_N, weighted_sum_W / sum_N)
}

/// Get the ratio of marker 1 individuals to individuals with other markers,
/// and arithmetic mean fitness of all of the lineages in `data`
///
/// Return format is `(marker_1_ratio, avg_W)`
pub fn marker_1_ratio_and_avg_W(data: &LineagesData) -> (f64, f64) {
    assert_eq!(data.N.len(), data.W.len());
    assert_eq!(data.N.len(), data.secondary.len());

    let mut sum_N = 0.0;
    let mut sum_N_marker_1 = 0.0;
    let mut weighted_sum_W = 0.0;

    for (n, w, secondary) in izip!(&data.N, &data.W, &data.secondary) {
        sum_N += n;
        weighted_sum_W += n * w;

        if secondary.marker == 1 {
            sum_N_marker_1 += n;
        }
    }

    (
        sum_N_marker_1 / (sum_N - sum_N_marker_1),
        weighted_sum_W / sum_N,
    )
}
