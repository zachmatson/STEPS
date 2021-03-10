//! Sample random variates from supported distributions  
//! Optimized for case where a single sample is needed for a given set of parameters

use super::*;

/// Sample a Poisson random variate from a distribution with mean `lambda` using provided `rng`
/// May panic or produce incorrect results on invalid lambda
pub fn poisson<R: Rng>(lambda: f64, rng: &mut R) -> u64 {
    if lambda <= 10.0 {
        direct_poisson(lambda, rng)
    } else {
        rand_distr::Poisson::new(lambda).unwrap().sample(rng)
    }
}

/// Sample a Poisson random variate from a distribution with mean `lambda` using provided `rng`  
/// Uses the Algorithm 3 described in https://www.jstor.org/stable/2347913  
/// Assumes `lambda` is valid for results to be meaningful
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

// fn sorted_uniform<R: Rng>(scale: f64, size: usize, rng: &mut R) -> Vec<f64> {
//     let mut samples = vec![0.0; size as usize];
//     let mut n = size as f64;
//     let mut a = 0.0;
//     for i in 0..(size as usize) {
//         let Fa = rng.gen::<f64>();
//         a = 1.0 + (a - 1.0) * (1.0 - Fa).powf(n.recip());
//         samples[i] = scale * a;
//         n -= 1.0;
//     }

//     samples
// }
