//! Agent-based SIR epidemic simulation using N parallel `MarkovChain`s.
//!
//! Each of 1000 individuals runs its own independent Markov chain with
//! transition matrix
//!
//! ```text
//!            S          I          R
//!   S  [ 1-β           β          0  ]
//!   I  [  0         1-γ           γ  ]
//!   R  [  0            0          1  ]   (absorbing)
//! ```
//!
//! This is a simplified SIR — we hold the infection rate `β` constant
//! rather than making it depend on the population's current infected
//! fraction. That yields exponential growth rather than a classic SIR
//! curve, but captures the S→I→R lifecycle per agent.
//!
//! The example demonstrates that `MarkovChain` composes naturally with
//! higher-level orchestration: the library gives you one chain per runner,
//! and you stack however many you want.

use state_machines_rs::{Runner, SplitMix64, primitives::MarkovChain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Health {
    Susceptible,
    Infected,
    Recovered,
}

const N_AGENTS: usize = 1_000;
const N_INITIAL_INFECTED: usize = 10;
const BETA: f64 = 0.02; // per-step infection probability for an S agent
const GAMMA: f64 = 0.05; // per-step recovery probability for an I agent
const T_MAX: usize = 400;

fn make_agent(initial: Health, seed: u64) -> Runner<MarkovChain<Health, SplitMix64>> {
    let idx = match initial {
        Health::Susceptible => 0,
        Health::Infected => 1,
        Health::Recovered => 2,
    };
    let labels = vec![Health::Susceptible, Health::Infected, Health::Recovered];
    let matrix = vec![
        vec![1.0 - BETA, BETA, 0.0],
        vec![0.0, 1.0 - GAMMA, GAMMA],
        vec![0.0, 0.0, 1.0],
    ];
    let mc = MarkovChain::new_with(labels, matrix, idx, SplitMix64::new(seed))
        .expect("valid transition matrix");
    Runner::new(mc)
}

fn main() {
    // Initialize the cohort.
    let mut agents: Vec<Runner<MarkovChain<Health, SplitMix64>>> = (0..N_AGENTS)
        .map(|i| {
            let initial = if i < N_INITIAL_INFECTED { Health::Infected } else { Health::Susceptible };
            make_agent(initial, 0xC0FFEE + i as u64)
        })
        .collect();

    let mut peak_infected = 0usize;
    let mut peak_day = 0usize;
    let mut series: Vec<(usize, usize, usize)> = Vec::with_capacity(T_MAX);

    for t in 0..T_MAX {
        let mut s = 0usize;
        let mut i = 0usize;
        let mut r = 0usize;
        for agent in agents.iter_mut() {
            match agent.step(()) {
                Health::Susceptible => s += 1,
                Health::Infected => i += 1,
                Health::Recovered => r += 1,
            }
        }
        if i > peak_infected {
            peak_infected = i;
            peak_day = t;
        }
        series.push((s, i, r));
    }

    // Sparse table for output.
    println!("day |    S    I    R");
    for (t, (s, i, r)) in series.iter().enumerate().step_by(20) {
        println!("{:4} | {:4} {:4} {:4}", t, s, i, r);
    }
    let (final_s, final_i, final_r) = *series.last().unwrap();
    println!("\npeak infected: {} on day {}", peak_infected, peak_day);
    println!("final counts:  S={}  I={}  R={}", final_s, final_i, final_r);

    // Qualitative sanity checks — stochastic, but the population is big
    // enough for these to hold every run.
    assert!(peak_infected > N_INITIAL_INFECTED * 10, "epidemic should grow");
    assert!(final_r > N_AGENTS * 9 / 10, "almost everyone recovered");
    assert!(final_i < N_AGENTS / 50, "epidemic should have burned out");
}
