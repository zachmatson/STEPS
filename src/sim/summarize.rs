//! Summarizing operations for lineage data

use itertools::izip;

use super::LineagesData;

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
pub fn marker_1_ratio(data: &LineagesData) -> f64 {
    let mut sum_N = 0.0;
    let mut marker_1_sum_N = 0.0;

    for (&n, secondary) in izip!(&data.N, &data.secondary) {
        sum_N += n;
        if secondary.marker == 1 {
            marker_1_sum_N += n;
        }
    }

    marker_1_sum_N / (sum_N - marker_1_sum_N)
}

/// Weighted population standard deviation  
/// Computations performed after conversion to f64
#[inline]
fn stdev<'a, E, W, IE, IW>(elements: impl Fn() -> IE, weights: impl Fn() -> IW) -> f64
where
    E: Copy,
    W: Copy,
    IE: Iterator<Item = E>,
    IW: Iterator<Item = W>,
    f64: From<E> + From<W>,
{
    let n = weights().map(f64::from).sum::<f64>();
    let mean = izip!(weights(), elements())
        .map(|(w, e)| f64::from(w) * f64::from(e))
        .sum::<f64>()
        / n;
    let sse = izip!(weights(), elements())
        .map(|(w, e)| f64::from(w) * (f64::from(e) - mean).powi(2))
        .sum::<f64>();

    (sse / n).sqrt()
}

/// Population standard deviation of lineage fitnesses
pub fn stdev_W(data: &LineagesData) -> f64 {
    stdev(|| data.W.iter().copied(), || data.N.iter().copied())
}

/// Population standard deviation of number of accumulated mutations for all lineages in the population
pub fn stdev_accumulated_muts(data: &LineagesData) -> f64 {
    stdev(
        || data.secondary.iter().map(|s| s.accumulated_muts),
        || data.N.iter().copied(),
    )
}

/// Maximum fitness of any lineage in the population
pub fn max_W(data: &LineagesData) -> f64 {
    *data
        .W
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap()
}

/// Maximum number of mutations away from the ancestor of any lineage in the population
pub fn max_accumulated_muts(data: &LineagesData) -> u32 {
    data.secondary
        .iter()
        .map(|s| s.accumulated_muts)
        .max()
        .unwrap()
}

/// Number of lineages/genotypes in the population
pub fn genotype_count(data: &LineagesData) -> usize {
    // Can happen when all members of a lineage are replaced with new mutants
    #[allow(clippy::float_cmp_const)]
    data.N.iter().filter(|&&n| n != 0.0).count()
}

/// Shannon diversity of genotypes, sum(p ln p) for all lineages where p is the lineage size
/// divided by the total size of all lineages
pub fn shannon_diversity(data: &LineagesData) -> f64 {
    let mut sum_N = 0.0;
    let mut weighted_sum_log_N = 0.0;

    for &n in &data.N {
        // Can happen when all members of a lineage are replaced with new mutants
        #[allow(clippy::float_cmp_const)]
        if n == 0.0 {
            continue;
        }
        sum_N += n;
        weighted_sum_log_N += n * n.ln();
    }

    sum_N.ln() - weighted_sum_log_N / sum_N
}
