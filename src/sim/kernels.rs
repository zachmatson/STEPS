use super::LineagesData;

pub fn grow_lineages_inplace(data: &mut LineagesData, delta_t: f64) {
    // Force matching sizes, eliminate bounds checks in inner loop
    let len = data.N.len();
    let N = &mut data.N[0..len];
    let W = &data.W[0..len];

    for i in 0..len {
        N[i] *= (W[i] * delta_t).exp2();
        N[i] = N[i].ceil();
    }
}

pub fn expected_mutation_counts_old(data: &LineagesData, old_N: &[f64]) -> Vec<f64> {
    // Tell the compiler we have equal length slices
    // Elide inner loop bounds checks and allow vectorization
    let len = data.N.len();
    let N = &data.N[0..len];
    let U = &data.U[0..len];
    let old_N = &old_N[0..len];
    let mut output = vec![0.0; len];

    for i in 0..len {
        output[i] = U[i] * (N[i] - old_N[i]);
    }

    output
}

pub fn delta_N_inplace<'a>(data: &LineagesData, old_N: &'a mut [f64]) -> &'a mut [f64] {
    let len = data.N.len();
    let N = &data.N[0..len];
    let old_N = &mut old_N[0..len];

    for i in 0..len {
        old_N[i] = N[i] - old_N[i];
    }

    old_N
}

pub fn expected_mutation_counts(data: &LineagesData, delta_N: &[f64]) -> Vec<f64> {
    // Tell the compiler we have equal length slices
    // Elide inner loop bounds checks and allow vectorization
    let len = data.U.len();
    let U = &data.U[0..len];
    let delta_N = &delta_N[0..len];
    let mut output = vec![0.0; len];

    for i in 0..len {
        output[i] = U[i] * delta_N[i];
    }

    output
}

// pub fn sum_N_and_avg_W(data: &LineagesData) -> (f64, f64) {
//     let N = &data.N;
//     let len = N.len();
//     let W = &data.W[0..len];

//     let mut sum_N = 0.0;
//     let mut weighted_sum_W = 0.0;

//     for i in 0..len {
//         sum_N += N[i];
//         weighted_sum_W += N[i] * W[i];
//     }

//     (sum_N, weighted_sum_W / sum_N)
// }

pub fn sum_N_and_avg_W(data: &LineagesData) -> (f64, f64) {
    // Tell the compiler we have equal length slices
    let len = data.N.len();
    let N = &data.N[0..len];
    let W = &data.W[0..len];

    // Sum in chunks of width 8
    // to promote autovectorization
    const WIDTH: usize = 8;
    let mut sum_N = [0.0; WIDTH];
    let mut weighted_sum_W = [0.0; WIDTH];
    // Length of the vectorizable portion
    let main_len = len - (len % WIDTH);

    // Iterate through size WIDTH chunks which can be added to the accumulator array
    for (N_chunk, W_chunk) in N[..main_len].chunks(WIDTH).zip(W[..main_len].chunks(WIDTH)) {
        // Iterate through the items in each chunk and add them
        for ((sum_N, weighted_sum_W), (n, w)) in sum_N
            .iter_mut()
            .zip(weighted_sum_W.iter_mut())
            .zip(N_chunk.iter().zip(W_chunk))
        {
            *sum_N += *n;
            *weighted_sum_W += *n * *w;
        }
    }

    // Reduce the arrays of partial sums to scalars
    let mut sum_N = sum_N.iter().sum();
    let mut weighted_sum_W = weighted_sum_W.iter().sum::<f64>();

    // Get the portions which could not be added in the main loop
    let N = &N[main_len..len];
    let W = &W[main_len..len];

    for i in 0..(len - main_len) {
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

    (
        sum_N_marker_1 / (sum_N - sum_N_marker_1),
        weighted_sum_W / sum_N,
    )
}
