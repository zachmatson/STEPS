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
    U: Vec<f64>,
}

impl Lineages {
    pub fn reserve(&mut self, n: usize) {
        self.N.reserve(n);
        self.W.reserve(n);
        self.U.reserve(n);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MutationType {
    Beneficial,
    Neutral,
    Deleterious,
    MutationRate,
}

pub fn run_simulations(cfg: &SimConfig) {
    // Validate that unimplemented parameters aren't in use
    if cfg.deleterious_mutation_rate != 0.0 {
        deleterious_todo();
    } else if cfg.mutation_rate_mutation_rate != 0.0 {
        mutation_rate_todo();
    }

    let mut rng = Pcg64::from_entropy();
    single_replicate(cfg, &mut rng);
}

fn single_replicate<R: Rng>(cfg: &SimConfig, rng: &mut R) {
    let mut lineages = initialize_lineages(cfg);
    println!("Start: {:?}", lineages);

    for i in 0..cfg.transfers {
        let delta_t_phase_1 = (cfg.dilution_factor.log2() - 1.0).floor() as usize;
        lineages = doubling_phase_1(delta_t_phase_1, lineages, cfg, rng);
        let delta_t_phase_2 = estimate_delta_t_phase_2(&lineages, cfg);
        lineages = doubling_phase_2(delta_t_phase_2, lineages, cfg, rng);

        println!("Generation {}: {:?}", i, lineages);
    }

    println!("End: {:?}", lineages);
}

fn initialize_lineages(cfg: &SimConfig) -> Lineages {
    let initial_n =
        (cfg.max_pop_size as f64 / cfg.dilution_factor / cfg.markers as f64).round() as u64;

    Lineages {
        N: vec![initial_n; cfg.markers as usize],
        W: vec![1.0; cfg.markers as usize],
        U: vec![cfg.total_mutation_rate; cfg.markers as usize],
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
        output.reserve(2 * lineages.N.len());

        // Iterate through all populations
        for i in 0..lineages.N.len() {
            let N_mut: u64 = rand_distr::Poisson::new(lineages.U[i] * lineages.N[i] as f64)
                .unwrap()
                .sample(rng);
            let new_N = (lineages.W[i].exp2() * lineages.N[i] as f64).round() as u64 - N_mut;

            output.N.push(new_N);
            output.W.push(lineages.W[i]);
            output.U.push(lineages.U[i]);

            for _ in 0..N_mut {
                push_new_mutant(lineages.W[i], lineages.U[i], &mut output, cfg, rng);
            }
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
        let N_bottlenecked = calculate_bottlenecked_size_with_growth(
            delta_t,
            lineages.N[i],
            lineages.W[i],
            cfg,
            rng,
        );

        if N_bottlenecked == 0 {
            continue;
        }

        println!(
            "{}: {} {}",
            lineages.U[i]
                * N_bottlenecked as f64
                * (1.0 - (lineages.W[i] * delta_t).exp2().recip()),
            lineages.W[i],
            delta_t
        );

        let N_mut = rand_distr::Poisson::new(
            lineages.U[i]
                * N_bottlenecked as f64
                * (1.0 - (lineages.W[i] * delta_t).exp2().recip()),
        )
        .unwrap()
        .sample(rng);
        let new_N = N_bottlenecked - N_mut;

        for _ in 0..N_mut {
            push_new_mutant(lineages.W[i], lineages.U[i], &mut output, cfg, rng);
        }

        if new_N == 0 {
            continue;
        }

        output.N.push(new_N);
        output.W.push(lineages.W[i]);
        output.U.push(lineages.U[i]);
    }

    output
}

fn estimate_delta_t_phase_2(lineages: &Lineages, cfg: &SimConfig) -> f64 {
    // let avg_W = lineages.W.iter().sum::<f64>() / lineages.W.len() as f64;
    let weighted_sum_W = lineages.W.iter().zip(lineages.N.iter()).map(|(w, n)| (*w) * (*n) as f64).sum::<f64>();
    let sum_N = lineages.N.iter().sum::<u64>() as f64;
    let avg_W = weighted_sum_W / sum_N;
    println!("{}", avg_W);
    println!("{}", sum_N);

    (cfg.max_pop_size as f64 / sum_N).log2() / avg_W
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

/// Push a mutant based on the `idx`th element of `input_lineages` to the end of `output_lineages`
fn push_new_mutant<R: Rng>(
    initial_W: f64,
    initial_U: f64,
    output_lineages: &mut Lineages,
    cfg: &SimConfig,
    rng: &mut R,
) {
    let mutation_type = cfg.sample_mutation_type(rng);
    match mutation_type {
        MutationType::Beneficial => {
            let mutation_size =
                rand_distr::Exp::new(cfg.diminishing_returns_epistasis_strength * initial_W)
                    .unwrap()
                    .sample(rng);
            output_lineages.N.push(1);
            output_lineages.W.push(initial_W + mutation_size);
            output_lineages.U.push(initial_U);
        }
        MutationType::Neutral => {
            output_lineages.N.push(1);
            output_lineages.W.push(initial_W);
            output_lineages.U.push(initial_U);
        }
        MutationType::Deleterious => deleterious_todo(),
        MutationType::MutationRate => mutation_rate_todo(),
    }
}

fn deleterious_todo() -> ! {
    todo!("Deleterious mutations not yet supported")
}

fn mutation_rate_todo() -> ! {
    todo!("Mutation rate mutations not yet supported")
}
