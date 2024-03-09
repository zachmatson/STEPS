//! Summarizing operations for lineage data

use itertools::izip;

use crate::sim::LineagesData;

/// Total population size and weighted average fitness of some lineages
pub struct SumNAndAvgW {
    /// Total population
    pub sum_N: f64,
    /// Average fitness
    pub avg_W: f64,
}

/// Get the total population size and arithmetic mean fitness of all of the lineages in `lineages`
pub fn sum_N_and_avg_W(lineages: &LineagesData) -> SumNAndAvgW {
    assert_eq!(lineages.N.len(), lineages.W.len());

    let mut sum_N = 0.0;
    let mut weighted_sum_W = 0.0;

    for (n, w) in izip!(&lineages.N, &lineages.W) {
        sum_N += n;
        weighted_sum_W += n * w;
    }

    SumNAndAvgW {
        sum_N,
        avg_W: weighted_sum_W / sum_N,
    }
}

/// Weighted arithmetic mean opf lineage fitnesses
pub fn avg_W(lineages: &LineagesData) -> f64 {
    sum_N_and_avg_W(lineages).avg_W
}

/// Ratio of marker 1 population to total population of other markers
pub fn marker_1_ratio(lineages: &LineagesData) -> f64 {
    let mut sum_N = 0.0;
    let mut marker_1_sum_N = 0.0;

    for (&n, secondary) in izip!(&lineages.N, &lineages.secondary) {
        sum_N += n;
        if secondary.marker == 1 {
            marker_1_sum_N += n;
        }
    }

    marker_1_sum_N / (sum_N - marker_1_sum_N)
}

/// Weighted population standard deviation
///
/// Computations performed after conversion to f64
#[inline]
fn stdev<E, W, IE, IW>(elements: impl Fn() -> IE, weights: impl Fn() -> IW) -> f64
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
pub fn stdev_W(lineages: &LineagesData) -> f64 {
    stdev(|| lineages.W.iter().copied(), || lineages.N.iter().copied())
}

/// Population standard deviation of number of accumulated mutations for all lineages in the population
pub fn stdev_accumulated_muts(lineages: &LineagesData) -> f64 {
    stdev(
        || lineages.secondary.iter().map(|s| s.accumulated_muts),
        || lineages.N.iter().copied(),
    )
}

/// Maximum fitness of any lineage in the population
pub fn max_W(lineages: &LineagesData) -> f64 {
    *lineages
        .W
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap()
}

/// Maximum number of mutations away from the ancestor of any lineage in the population
pub fn max_accumulated_muts(lineages: &LineagesData) -> u32 {
    lineages
        .secondary
        .iter()
        .map(|s| s.accumulated_muts - 1)
        .max()
        .unwrap()
}

/// Mean number of mutations away from the ancestor of any lineage in the population
pub fn mean_accumulated_muts(lineages: &LineagesData) -> f64 {
    let mut sum_N = 0.0;
    let mut sum_M = 0.0;

    for (&n, secondary) in izip!(&lineages.N, &lineages.secondary) {
        sum_N += n;
        sum_M += (secondary.accumulated_muts - 1) as f64 * n;
    }

    sum_M / sum_N
}

/// Minimum number of mutations away from the ancestor of any lineage in the population
pub fn min_accumulated_muts(lineages: &LineagesData) -> u32 {
    lineages
        .secondary
        .iter()
        .map(|s| s.accumulated_muts - 1)
        .min()
        .unwrap()
}

/// Number of lineages/genotypes in the population
pub fn genotype_count(lineages: &LineagesData) -> usize {
    // Can happen when all members of a lineage are replaced with new mutants
    #[allow(clippy::float_cmp_const)]
    lineages.N.iter().filter(|&&n| n != 0.0).count()
}

/// Shannon diversity of genotypes, sum(p ln p) for all lineages where p is the lineage size
/// divided by the total size of all lineages
pub fn shannon_diversity(lineages: &LineagesData) -> f64 {
    let mut sum_N = 0.0;
    let mut weighted_sum_log_N = 0.0;

    for &n in &lineages.N {
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
