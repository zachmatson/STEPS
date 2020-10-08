use super::*;

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
