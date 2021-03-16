//! Performance sensitive and optimized computational kernels for the simulations  
//! Transform or summarize the data in a `LineagesData`  
//! Lower-level implementation details of the transfer process  

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
///
/// The length of any vector in `data` will *not* be changed
pub fn old_N_to_delta_N<'a>(data: &LineagesData, old_N: &'a mut [f64]) -> &'a mut [f64] {
    // Force matching sizes, eliminate bounds checks in inner loop to allow vectorization
    let len = data.N.len();
    let N = &data.N[0..len];
    let old_N = &mut old_N[0..len];

    for i in 0..len {
        old_N[i] = N[i] - old_N[i];
    }

    old_N
}

/// Get the expected number of mutations for each lineage as a newly allocated
/// `Vec`, given the `data` containing the lineages and a slice `delta_N` of the
/// number of individuals in each lineage eligible to mutate
///
/// The length of any vector in `data` will *not* be changed
pub fn expected_mutation_counts(data: &LineagesData, delta_N: &[f64]) -> Vec<f64> {
    // Force matching sizes, eliminate bounds checks in inner loop to allow vectorization
    let len = data.U.len();
    let U = &data.U[0..len];
    let delta_N = &delta_N[0..len];
    let mut output = vec![0.0; len];

    for i in 0..len {
        output[i] = U[i] * delta_N[i];
    }

    output
}

/// Get the total population size and arithmetic mean fitness
/// of all of the lineages in `data`
///
/// Return format is `(sum_N, avg_W)`
///
/// The length of any vector in `data` will *not* be changed
pub fn sum_N_and_avg_W(data: &LineagesData) -> (f64, f64) {
    let N = &data.N;
    let len = N.len();
    let W = &data.W[0..len];

    let mut sum_N = 0.0;
    let mut weighted_sum_W = 0.0;

    for i in 0..len {
        sum_N += N[i];
        weighted_sum_W += N[i] * W[i];
    }

    (sum_N, weighted_sum_W / sum_N)
}

/// Get the ratio of marker 1 individuals to individuals with other markers,
/// and arithmetic mean fitness of all of the lineages in `data`
///
/// Return format is `(marker_1_ratio, avg_W)`
///
/// The length of any vector in `data` will *not* be changed
pub fn marker_1_ratio_and_avg_W(data: &LineagesData) -> (f64, f64) {
    // Force matching sizes, eliminate bounds checks in inner loop
    // In this case, the branch proably still prevents vectorization
    // along with the AoS nature of the marker storage
    let len = data.N.len();
    let N = &data.N[0..len];
    let W = &data.W[0..len];
    let secondary = &data.secondary[0..len];

    let mut sum_N = 0.0;
    let mut sum_N_marker_1 = 0.0;
    let mut weighted_sum_W = 0.0;

    for i in 0..len {
        sum_N += N[i];
        weighted_sum_W += N[i] * W[i];

        if secondary[i].marker == 1 {
            sum_N_marker_1 += N[i];
        }
    }

    (
        sum_N_marker_1 / (sum_N - sum_N_marker_1),
        weighted_sum_W / sum_N,
    )
}
