//! Character-level text generator via first-order Markov chain.
//!
//! Build a bigram transition table from a training corpus, normalize to a
//! row-stochastic matrix, hand it to `MarkovChain`, and sample. First-order
//! bigram models don't produce grammatical prose, but they do produce
//! text with the correct letter-pair statistics — recognizable as
//! "English-adjacent" rather than random noise.
//!
//! The corpus here is the opening lines of *A Tale of Two Cities*,
//! normalized to lowercase ASCII so the alphabet stays small.

use std::collections::BTreeMap;
use state_machines_rs::{Runner, SplitMix64, primitives::MarkovChain};

const CORPUS: &str = "\
it was the best of times it was the worst of times \
it was the age of wisdom it was the age of foolishness \
it was the epoch of belief it was the epoch of incredulity \
it was the season of light it was the season of darkness \
it was the spring of hope it was the winter of despair";

fn main() {
    // Alphabet: every unique character in the corpus, sorted for
    // reproducibility.
    let mut alphabet: Vec<char> = CORPUS.chars().collect();
    alphabet.sort_unstable();
    alphabet.dedup();
    let n = alphabet.len();
    let idx: BTreeMap<char, usize> = alphabet.iter().enumerate().map(|(i, c)| (*c, i)).collect();

    // Count bigrams. One pseudo-count everywhere (add-one / Laplace
    // smoothing) so every row has support and the matrix is stochastic.
    let mut counts: Vec<Vec<f64>> = (0..n).map(|_| vec![1.0; n]).collect();
    let chars: Vec<char> = CORPUS.chars().collect();
    for w in chars.windows(2) {
        let (a, b) = (w[0], w[1]);
        counts[idx[&a]][idx[&b]] += 1.0;
    }

    // Normalize each row to sum to 1.
    let transitions: Vec<Vec<f64>> = counts
        .into_iter()
        .map(|row| {
            let total: f64 = row.iter().sum();
            row.into_iter().map(|c| c / total).collect()
        })
        .collect();

    let start_idx = idx[&'i']; // seed with 'i'
    let mc = MarkovChain::new_with(
        alphabet.clone(),
        transitions,
        start_idx,
        SplitMix64::new(0xD1CE),
    )
    .expect("stochastic matrix");

    let samples: Vec<char> = Runner::new(mc).run(400);
    let generated: String = samples.iter().collect();
    println!("generated text:\n{}\n", generated);

    // Sanity checks: the output should be majority alphabet letters + spaces
    // (since that's what the corpus is), and it should not collapse to a
    // single character (which would indicate a sampling bug).
    let distinct: std::collections::HashSet<char> = samples.iter().copied().collect();
    assert!(
        distinct.len() >= n / 3,
        "output should exercise a reasonable fraction of the alphabet, got {}",
        distinct.len()
    );
    // The most common letter in the corpus is space; expect it to be
    // the most common in the output too (within sampling noise).
    let mut freq: BTreeMap<char, usize> = BTreeMap::new();
    for c in &samples { *freq.entry(*c).or_insert(0) += 1; }
    let top = freq.iter().max_by_key(|(_, v)| **v).unwrap();
    assert_eq!(*top.0, ' ', "space should dominate the output, got {:?}", top);
}
