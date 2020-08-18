use rand::prelude::*;
use rand_distr;
use rand_pcg::Pcg64;

use crate::*;

#[derive(Default, Debug)]
pub struct Lineages {
    /// Population size
    N: Vec<u64>,
    /// Population fitness
    W: Vec<f64>,
    /// Population total mutation rate  
    /// Use to calculate chance of mutations happening,
    /// but defer to general mutation rate to determine type
    Upop: Vec<f64>,
}

impl Lineages {
    pub fn reserve(&mut self, n: usize) {
        self.N.reserve(n);
        self.W.reserve(n);
        self.Upop.reserve(n);
    }
}

pub fn run_simulations(cfg: &SimConfig) {
    // Validate that unimplemented parameters aren't in use
    if cfg.deleterious_mutation_rate != 0.0 {
        todo!("Deleterious mutations not yet supported")
    } else if cfg.mutation_rate_mutation_rate != 0.0 {
        todo!("Mutation rate mutations not yet supported")
    }

    let mut rng = Pcg64::from_entropy();
    let mut lineages = initialize_lineages(cfg);
    println!("Start: {:?}", lineages);

    for i in 0..cfg.transfers {
        let delta_t_phase_1 = (cfg.dilution_factor.log2() - 1.0).floor() as usize;
        lineages = doubling_phase_1(delta_t_phase_1, lineages, cfg, &mut rng);
        let delta_t_phase_2 = estimate_delta_t_phase_2(&lineages, cfg);
        lineages = doubling_phase_2(delta_t_phase_2, lineages, cfg, &mut rng);

        if i % 500_000 == 0 {
            println!("Generation {}: {:?}", i, lineages);
        }
    }

    println!("End: {:?}", lineages);
}

fn initialize_lineages(cfg: &SimConfig) -> Lineages {
    let initial_n =
        (cfg.max_pop_size as f64 / cfg.dilution_factor / cfg.markers as f64).round() as u64;

    Lineages {
        N: vec![initial_n; cfg.markers as usize],
        W: vec![1.0; cfg.markers as usize],
        Upop: vec![cfg.total_mutation_rate; cfg.markers as usize],
    }
}

fn doubling_phase_1<R: Rng>(
    delta_t: usize,
    mut lineages: Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) -> Lineages {
    // Iterate through each doubling
    for _ in 0..delta_t {
        // Create output vector
        let mut output = Lineages::default();
        output.reserve(lineages.N.len());

        // Iterate through all populations
        for i in 0..lineages.N.len() {
            // let N_mut = rand_distr::Binomial::new(initial.N[i], initial.Upop[i])
            //     .unwrap()
            //     .sample(rng);
            let N_mut = 0;
            let new_N = (lineages.W[i].exp2() * lineages.N[i] as f64).round() as u64 - N_mut;

            output.N.push(new_N);
            output.W.push(lineages.W[i]);
            output.Upop.push(lineages.Upop[i]);

            /* TODO: Handle mutants */
        }

        lineages = output;
    }

    lineages
}

fn doubling_phase_2<R: Rng>(
    delta_t: f64,
    lineages: Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) -> Lineages {
    // Create output vector
    let mut output = Lineages::default();
    output.reserve(lineages.N.len());

    for i in 0..lineages.N.len() {
        let new_N = calculate_bottlenecked_size_with_growth(
            delta_t,
            lineages.N[i],
            lineages.W[i],
            cfg,
            rng,
        );

        output.N.push(new_N);
        output.W.push(lineages.W[i]);
        output.Upop.push(lineages.Upop[i]);

        /* TODO: Handle mutants */
    }

    output
}

fn estimate_delta_t_phase_2(lineages: &Lineages, cfg: &SimConfig) -> f64 {
    let avg_W = lineages.W.iter().sum::<f64>() / lineages.W.len() as f64;
    let sum_N = lineages.N.iter().sum::<u64>();

    (cfg.max_pop_size as f64 / sum_N as f64).log2() / avg_W
}

fn calculate_bottlenecked_size_with_growth<R: Rng>(
    delta_t: f64,
    N: u64,
    W: f64,
    cfg: &SimConfig,
    rng: &mut R,
) -> u64 {
    let N_after_growth = ((W * delta_t).exp2() * N as f64).round() as u64;
    rand_distr::Binomial::new(N_after_growth, cfg.dilution_factor.recip())
        .unwrap()
        .sample(rng)
}
