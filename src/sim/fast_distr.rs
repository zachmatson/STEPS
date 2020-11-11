//! Sample random variates from supported distributions  
//! Optimized for case where a single sample is needed for a given set of parameters

use super::*;

/// Sample a Poisson random variate from a distribution with mean `lambda` using provided `rng` 
/// Will *panic* on invalid lambda 
pub fn poisson<R: Rng>(lambda: f64, rng: &mut R) -> u64 {
    if lambda <= 0.0 {
        panic!("NEGATIVE LAMBDA NOT ALLOWED")
    } else if lambda <= 10.0 {
        direct_poisson(lambda, rng)
    } else {
        rand_distr::Poisson::new(lambda).unwrap().sample(rng)
    }
}

/// Sample a Poisson random variate from a distribution with mean `lambda` using provided `rng`  
/// Uses the Algorithm 3 described in https://www.jstor.org/stable/2347913  
/// Assumes `lambda` is value for results to be meaningful
fn direct_poisson<R: Rng>(lambda: f64, rng: &mut R) -> u64 {
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
