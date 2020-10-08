use super::*;

/// Sample a Poisson random variate from a distribution with mean `lambda` using `rng`
/// Uses the multiplication method described in https://www.jstor.org/stable/2346807 
pub fn direct_poisson<R: Rng>(lambda: f64, rng: &mut R) -> u64 {
    let thresh = (-lambda).exp();
    let mut n = 0;
    let mut prod = 1.0;

    while prod > thresh {
        n += 1;
        prod *= rng.gen::<f64>();
    }

    n - 1
}
