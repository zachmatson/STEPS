use super::LineagesData;

pub fn grow_lineages_inplace(data: &mut LineagesData, delta_t: f64) {
    // Force matching sizes, eliminate bounds checks in inner loop
    let N = &mut data.N;
    let W = &data.W[0..N.len()];

    for (n, w) in N.iter_mut().zip(W.iter()) {
        *n *= (*w * delta_t).exp2();
        *n = n.ceil();
    }
}

pub fn expected_mutation_counts(data: &LineagesData, old_N: &[f64]) -> Vec<f64> {
    // Force matching sizes, eliminate bounds checks in inner loop
    let N = &data.N;
    let U = &data.U[0..N.len()];
    let old_N = &old_N[0..N.len()];
    let mut output = vec![0.0; N.len()];

    for i in 0..N.len() {
        output[i] = U[i] * (N[i] - old_N[i]);
    }

    output
}

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

pub fn marker_1_ratio_and_avg_W(data: &LineagesData) -> (f64, f64) {
    let N = &data.N;
    let len = N.len();
    let W = &data.W[0..len];
    let secondary = &data.secondary[0..len];

    let mut sum_N = 0.0;
    let mut sum_N_marker_1 = 0.0;
    let mut weighted_sum_W = 0.0;

    for i in 0..len {
        sum_N += N[i];
        weighted_sum_W += N[i] * W[i];

        if secondary[i].marker == 1 {
            sum_N_marker_1 += 1.0;
        }
    }

    (sum_N_marker_1 / (sum_N - sum_N_marker_1), weighted_sum_W / sum_N)
}