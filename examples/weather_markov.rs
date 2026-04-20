//! Markov chain simulation of weather transitions.
//!
//! Classic example of a finite-state stochastic process: Sunny, Cloudy,
//! Rainy, with hand-tuned transition probabilities that favor persistence
//! (weather tends to stay the same day-to-day).
//!
//! This example uses the library's reference RNG [`SplitMix64`], which
//! has no external dependencies. To use `rand` crate RNGs directly,
//! build with `--features rand` and pass `rand::rngs::SmallRng` (or
//! similar) to `MarkovChain::new_with` — the library impls `Rng` for
//! any `rand::RngCore + Clone`.

use state_machines_rs::{Runner, SplitMix64, primitives::MarkovChain};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Weather {
    Sunny,
    Cloudy,
    Rainy,
}

fn main() {
    let labels = vec![Weather::Sunny, Weather::Cloudy, Weather::Rainy];
    let transitions = vec![
        //           Sunny  Cloudy  Rainy
        /* from  Sunny */ vec![0.7, 0.2, 0.1],
        /* from Cloudy */ vec![0.3, 0.4, 0.3],
        /* from  Rainy */ vec![0.2, 0.3, 0.5],
    ];

    let mc = MarkovChain::new_with(
        labels,
        transitions,
        0, // start Sunny
        SplitMix64::new(0xA17ECAFE),
    )
    .expect("valid Markov chain");

    let n = 100_000;
    let trajectory: Vec<Weather> = Runner::new(mc).run(n);

    let mut sunny = 0usize;
    let mut cloudy = 0usize;
    let mut rainy = 0usize;
    for w in &trajectory {
        match w {
            Weather::Sunny => sunny += 1,
            Weather::Cloudy => cloudy += 1,
            Weather::Rainy => rainy += 1,
        }
    }

    // Stationary distribution π for this matrix, solved by hand:
    //   0.7πs + 0.3πc + 0.2πr = πs
    //   0.2πs + 0.4πc + 0.3πr = πc
    //   0.1πs + 0.3πc + 0.5πr = πr
    //   πs + πc + πr = 1
    // → π ≈ (0.463, 0.282, 0.255)
    let total = n as f64;
    let ps = sunny as f64 / total;
    let pc = cloudy as f64 / total;
    let pr = rainy as f64 / total;

    println!("empirical distribution over {} steps:", n);
    println!("  Sunny  = {:.4}  (stationary ≈ 0.463)", ps);
    println!("  Cloudy = {:.4}  (stationary ≈ 0.282)", pc);
    println!("  Rainy  = {:.4}  (stationary ≈ 0.255)", pr);

    // Empirical frequencies should be close to the stationary distribution
    // over this many samples.
    assert!((ps - 0.463).abs() < 0.02);
    assert!((pc - 0.282).abs() < 0.02);
    assert!((pr - 0.255).abs() < 0.02);

    // Print the first few days of the trajectory for flavor.
    print!("\nfirst 30 days: ");
    for w in &trajectory[..30] {
        let c = match w {
            Weather::Sunny => 'S',
            Weather::Cloudy => 'C',
            Weather::Rainy => 'R',
        };
        print!("{}", c);
    }
    println!();
}
