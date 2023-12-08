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
