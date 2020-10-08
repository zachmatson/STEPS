use super::*;

/// Sample a Poisson random variate from a distribution with mean `lambda` using `rng`
/// Uses the Algorithm 3 described in https://www.jstor.org/stable/2347913
pub fn direct_poisson<R: Rng>(lambda: f64, rng: &mut R) -> u64 {
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
