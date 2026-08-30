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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::types::SecondaryLineageData;
    use approx::assert_relative_eq;

    fn make_lineages(data: &[(f64, f64, u16, u32)]) -> LineagesData {
        let mut lineages = LineagesData::default();
        for &(n, w, marker, accumulated_muts) in data {
            lineages.N.push(n);
            lineages.W.push(w);
            lineages.U.push(0.0);
            lineages.secondary.push(SecondaryLineageData {
                lambda: 0.0,
                id: 0,
                parent_id: 0,
                marker,
                accumulated_muts,
            });
        }
        lineages
    }

    #[test]
    fn test_avg_w_uniform_fitness() {
        // All lineages have W=1.0, average should be 1.0
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (200.0, 1.0, 1, 1)]);
        assert_relative_eq!(avg_W(&lineages), 1.0);
    }

    #[test]
    fn test_avg_w_weighted() {
        // N=100 W=1.0, N=100 W=2.0 → avg = (100*1.0 + 100*2.0)/200 = 1.5
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (100.0, 2.0, 1, 1)]);
        assert_relative_eq!(avg_W(&lineages), 1.5);
    }

    #[test]
    fn test_avg_w_unequal_sizes() {
        // N=300 W=1.0, N=100 W=2.0 → avg = (300*1.0 + 100*2.0)/400 = 500/400 = 1.25
        let lineages = make_lineages(&[(300.0, 1.0, 1, 1), (100.0, 2.0, 1, 1)]);
        assert_relative_eq!(avg_W(&lineages), 1.25);
    }

    #[test]
    fn test_sum_n_and_avg_w() {
        let lineages = make_lineages(&[(200.0, 1.0, 1, 1), (300.0, 1.5, 1, 1)]);
        let result = sum_N_and_avg_W(&lineages);
        assert_relative_eq!(result.sum_N, 500.0);
        // (200*1.0 + 300*1.5) / 500 = 650/500 = 1.3
        assert_relative_eq!(result.avg_W, 1.3);
    }

    #[test]
    fn test_marker_1_ratio_equal_split() {
        // Marker 1 has 100, marker 2 has 100 → ratio = 100/100 = 1.0
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (100.0, 1.0, 2, 1)]);
        assert_relative_eq!(marker_1_ratio(&lineages), 1.0);
    }

    #[test]
    fn test_marker_1_ratio_unequal() {
        // Marker 1 has 300, marker 2 has 100 → ratio = 300/100 = 3.0
        let lineages = make_lineages(&[(300.0, 1.0, 1, 1), (100.0, 1.0, 2, 1)]);
        assert_relative_eq!(marker_1_ratio(&lineages), 3.0);
    }

    #[test]
    fn test_stdev_w_uniform() {
        // All lineages same fitness → stdev = 0
        let lineages = make_lineages(&[(100.0, 1.5, 1, 1), (200.0, 1.5, 1, 1)]);
        assert_relative_eq!(stdev_W(&lineages), 0.0);
    }

    #[test]
    fn test_stdev_w_two_values() {
        // N=50 W=1.0, N=50 W=2.0
        // mean = 1.5, variance = (50*(1.0-1.5)^2 + 50*(2.0-1.5)^2)/100 = 50*0.25/100 = 0.25
        // stdev = 0.5
        let lineages = make_lineages(&[(50.0, 1.0, 1, 1), (50.0, 2.0, 1, 1)]);
        assert_relative_eq!(stdev_W(&lineages), 0.5);
    }

    #[test]
    fn test_max_w() {
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (50.0, 1.8, 1, 1), (200.0, 1.3, 1, 1)]);
        assert_relative_eq!(max_W(&lineages), 1.8);
    }

    #[test]
    fn test_max_accumulated_muts() {
        // accumulated_muts values are 3, 5, 2 → max is 5-1=4 (subtracts 1 for the initial marker)
        let lineages = make_lineages(&[(100.0, 1.0, 1, 3), (100.0, 1.0, 1, 5), (100.0, 1.0, 1, 2)]);
        assert_eq!(max_accumulated_muts(&lineages), 4);
    }

    #[test]
    fn test_min_accumulated_muts() {
        let lineages = make_lineages(&[(100.0, 1.0, 1, 3), (100.0, 1.0, 1, 5), (100.0, 1.0, 1, 2)]);
        assert_eq!(min_accumulated_muts(&lineages), 1);
    }

    #[test]
    fn test_mean_accumulated_muts() {
        // N=100 muts=3, N=200 muts=1 → mean = (100*2 + 200*0)/300 = 200/300 = 0.6667
        let lineages = make_lineages(&[(100.0, 1.0, 1, 3), (200.0, 1.0, 1, 1)]);
        assert_relative_eq!(mean_accumulated_muts(&lineages), 2.0 / 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_stdev_accumulated_muts() {
        // N=100 muts=2 (val=1), N=100 muts=4 (val=3)
        // mean = (100*1 + 100*3)/200 = 2.0
        // variance = (100*(1-2)^2 + 100*(3-2)^2)/200 = 200/200 = 1.0
        // stdev = 1.0
        let lineages = make_lineages(&[(100.0, 1.0, 1, 2), (100.0, 1.0, 1, 4)]);
        assert_relative_eq!(stdev_accumulated_muts(&lineages), 1.0);
    }

    #[test]
    fn test_genotype_count() {
        // 3 lineages, one with N=0 → count = 2
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (0.0, 1.0, 1, 1), (50.0, 1.2, 1, 1)]);
        assert_eq!(genotype_count(&lineages), 2);
    }

    #[test]
    fn test_genotype_count_all_nonzero() {
        let lineages = make_lineages(&[(10.0, 1.0, 1, 1), (20.0, 1.0, 1, 1), (30.0, 1.0, 1, 1)]);
        assert_eq!(genotype_count(&lineages), 3);
    }

    #[test]
    fn test_shannon_diversity_single_lineage() {
        // One lineage → diversity = 0 (no uncertainty)
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1)]);
        assert_relative_eq!(shannon_diversity(&lineages), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_diversity_equal_lineages() {
        // Two equal lineages → diversity = ln(2)
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (100.0, 1.0, 2, 1)]);
        assert_relative_eq!(shannon_diversity(&lineages), 2.0_f64.ln(), epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_diversity_three_equal() {
        // Three equal lineages → diversity = ln(3)
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (100.0, 1.0, 2, 1), (100.0, 1.0, 3, 1)]);
        assert_relative_eq!(shannon_diversity(&lineages), 3.0_f64.ln(), epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_diversity_skips_zero_n() {
        // Two lineages + one dead → diversity = ln(2)
        let lineages = make_lineages(&[(100.0, 1.0, 1, 1), (0.0, 1.0, 2, 1), (100.0, 1.0, 3, 1)]);
        assert_relative_eq!(shannon_diversity(&lineages), 2.0_f64.ln(), epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_diversity_unequal() {
        // N=75, N=25 → p1=0.75, p2=0.25
        // H = -(0.75*ln(0.75) + 0.25*ln(0.25))
        let lineages = make_lineages(&[(75.0, 1.0, 1, 1), (25.0, 1.0, 2, 1)]);
        let expected = -(0.75_f64 * 0.75_f64.ln() + 0.25_f64 * 0.25_f64.ln());
        assert_relative_eq!(shannon_diversity(&lineages), expected, epsilon = 1e-10);
    }
}
