//! Sample random variates from supported distributions  

use rand::prelude::*;

/// Sample a Poisson random variate from a distribution with mean `lambda` using provided `rng`
///
/// May panic or produce incorrect results on invalid lambda
///
/// Optimized for case where a single sample is needed for a given set of parameters
pub fn poisson<R: Rng>(lambda: f64, rng: &mut R) -> u64 {
    // rand_distr Poisson was slower for large lambda
    // This performance optimization probably mattered more for older versions
    if lambda <= 10.0 {
        direct_poisson(lambda, rng)
    } else {
        rand_distr::Poisson::new(lambda).unwrap().sample(rng)
    }
}

/// Sample a Poisson random variate from a distribution with mean `lambda` using provided `rng`
///
/// Uses the Algorithm 3 described in <https://www.jstor.org/stable/2347913>
///
/// Faster than the `rand_distr` implementation for single samples with small lambdas
fn direct_poisson<R: Rng>(lambda: f64, rng: &mut R) -> u64 {
    assert!(lambda >= 0.0, "Poisson called with negative lambda");
    let mut x = 0;
    let mut p = (-lambda).exp();
    let mut u = rng.gen::<f64>();

    while u > p {
        x += 1;
        u -= p;
        p *= lambda / x as f64;
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    fn seeded_rng() -> Pcg64 {
        Pcg64::seed_from_u64(42)
    }

    #[test]
    fn test_poisson_zero_lambda() {
        let mut rng = seeded_rng();
        for _ in 0..100 {
            assert_eq!(poisson(0.0, &mut rng), 0);
        }
    }

    #[test]
    fn test_poisson_small_lambda_mean() {
        let mut rng = seeded_rng();
        let lambda = 3.0;
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| poisson(lambda, &mut rng) as f64).sum();
        let mean = sum / n as f64;
        // Mean should be close to lambda (within ~5% for 10k samples)
        assert!(
            (mean - lambda).abs() < 0.15,
            "mean={mean}, expected≈{lambda}"
        );
    }

    #[test]
    fn test_poisson_large_lambda_mean() {
        let mut rng = seeded_rng();
        let lambda = 50.0;
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| poisson(lambda, &mut rng) as f64).sum();
        let mean = sum / n as f64;
        assert!(
            (mean - lambda).abs() < 1.5,
            "mean={mean}, expected≈{lambda}"
        );
    }

    #[test]
    fn test_poisson_variance() {
        let mut rng = seeded_rng();
        let lambda = 5.0;
        let n = 10_000;
        let samples: Vec<f64> = (0..n).map(|_| poisson(lambda, &mut rng) as f64).collect();
        let mean = samples.iter().sum::<f64>() / n as f64;
        let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        // For Poisson, variance = mean = lambda
        assert!(
            (variance - lambda).abs() < 0.5,
            "variance={variance}, expected≈{lambda}"
        );
    }

    #[test]
    fn test_poisson_deterministic_with_seed() {
        let mut rng1 = seeded_rng();
        let mut rng2 = seeded_rng();
        let results1: Vec<u64> = (0..20).map(|_| poisson(7.0, &mut rng1)).collect();
        let results2: Vec<u64> = (0..20).map(|_| poisson(7.0, &mut rng2)).collect();
        assert_eq!(results1, results2);
    }

    #[test]
    fn test_direct_poisson_small_lambda() {
        let mut rng = seeded_rng();
        let lambda = 2.0;
        let n = 10_000;
        let sum: f64 = (0..n)
            .map(|_| direct_poisson(lambda, &mut rng) as f64)
            .sum();
        let mean = sum / n as f64;
        assert!(
            (mean - lambda).abs() < 0.15,
            "mean={mean}, expected≈{lambda}"
        );
    }

    #[test]
    #[should_panic(expected = "Poisson called with negative lambda")]
    fn test_direct_poisson_negative_lambda_panics() {
        let mut rng = seeded_rng();
        direct_poisson(-1.0, &mut rng);
    }

    #[test]
    fn test_poisson_boundary_lambda_10() {
        // lambda=10 is the boundary — uses direct_poisson
        let mut rng = seeded_rng();
        let lambda = 10.0;
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| poisson(lambda, &mut rng) as f64).sum();
        let mean = sum / n as f64;
        assert!(
            (mean - lambda).abs() < 0.5,
            "mean={mean}, expected≈{lambda}"
        );
    }

    #[test]
    fn test_poisson_above_boundary() {
        // lambda=11 uses rand_distr implementation
        let mut rng = seeded_rng();
        let lambda = 11.0;
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| poisson(lambda, &mut rng) as f64).sum();
        let mean = sum / n as f64;
        assert!(
            (mean - lambda).abs() < 0.5,
            "mean={mean}, expected≈{lambda}"
        );
    }
}
