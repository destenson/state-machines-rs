//! Discrete-time Markov chain as a state machine.
//!
//! Given a row-stochastic transition matrix `P` where `P[i][j]` is the
//! probability of moving from state `i` to state `j`, `MarkovChain` emits a
//! sampled trajectory: each step draws `u ~ Uniform[0,1)` from the supplied
//! RNG and advances via inverse-CDF over the current row.
//!
//! The RNG type is a parameter — bring your own. The library's [`Rng`]
//! trait needs only `next_u64`; implement it for any PRNG you like, or
//! enable the `rand` feature to get a blanket impl for any
//! `rand::RngCore + Clone`.
//!
//! # Example — weather model
//!
//! ```rust
//! use state_machines_rs::{Runner, SplitMix64,
//!     primitives::MarkovChain};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum Weather { Sunny, Cloudy, Rainy }
//!
//! // P[from][to]. Rows must sum to 1.0.
//! let mc = MarkovChain::new_with(
//!     vec![Weather::Sunny, Weather::Cloudy, Weather::Rainy],
//!     vec![
//!         vec![0.7, 0.2, 0.1],  // Sunny stays sunny most days
//!         vec![0.3, 0.4, 0.3],
//!         vec![0.2, 0.3, 0.5],  // Rainy persists
//!     ],
//!     0,                          // start Sunny
//!     SplitMix64::new(42),
//! ).unwrap();
//!
//! let trajectory: Vec<_> = Runner::new(mc).run(5);
//! // Deterministic given the seed; every run is the same trajectory.
//! assert_eq!(trajectory.len(), 5);
//! ```
//!
//! [`Rng`]: crate::Rng

use crate::core::StateMachine;
use crate::rng::Rng;

/// A discrete-time Markov chain over a finite, labeled state space.
pub struct MarkovChain<S, R: Rng> {
    labels: Vec<S>,
    /// Row-stochastic matrix. `transitions[i][j]` is P(next = j | current = i).
    transitions: Vec<Vec<f64>>,
    initial_idx: usize,
    initial_rng: R,
}

/// Errors that can occur while constructing a [`MarkovChain`].
#[derive(Debug, Clone, PartialEq)]
pub enum MarkovChainError {
    /// Matrix row count didn't match the number of labels.
    ShapeMismatch { labels: usize, rows: usize },
    /// A row's column count didn't match the label count (matrix not square).
    RowLengthMismatch { row: usize, len: usize, expected: usize },
    /// A row's probabilities didn't sum to ~1.0.
    RowNotStochastic { row: usize, sum: f64 },
    /// A matrix entry was negative.
    NegativeProbability { row: usize, col: usize, value: f64 },
    /// `initial_idx` is not a valid row index.
    InitialIdxOutOfRange { idx: usize, n_states: usize },
}

impl core::fmt::Display for MarkovChainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ShapeMismatch { labels, rows } => write!(
                f,
                "MarkovChain shape mismatch: {} labels but {} matrix rows",
                labels, rows
            ),
            Self::RowLengthMismatch { row, len, expected } => write!(
                f,
                "MarkovChain row {} has length {}, expected {}",
                row, len, expected
            ),
            Self::RowNotStochastic { row, sum } => write!(
                f,
                "MarkovChain row {} probabilities sum to {}, not 1.0",
                row, sum
            ),
            Self::NegativeProbability { row, col, value } => write!(
                f,
                "MarkovChain has negative probability at [{}][{}] = {}",
                row, col, value
            ),
            Self::InitialIdxOutOfRange { idx, n_states } => write!(
                f,
                "MarkovChain initial_idx {} is out of range for {} states",
                idx, n_states
            ),
        }
    }
}

impl std::error::Error for MarkovChainError {}

const STOCHASTIC_TOLERANCE: f64 = 1e-9;

impl<S, R: Rng> MarkovChain<S, R> {
    /// Construct a Markov chain. Validates that the transition matrix is
    /// square, all rows are non-negative and sum to 1.0 within `1e-9`, and
    /// `initial_idx` is in range.
    pub fn new_with(
        labels: Vec<S>,
        transitions: Vec<Vec<f64>>,
        initial_idx: usize,
        initial_rng: R,
    ) -> Result<Self, MarkovChainError> {
        let n = labels.len();
        if transitions.len() != n {
            return Err(MarkovChainError::ShapeMismatch {
                labels: n,
                rows: transitions.len(),
            });
        }
        if initial_idx >= n {
            return Err(MarkovChainError::InitialIdxOutOfRange {
                idx: initial_idx,
                n_states: n,
            });
        }
        for (i, row) in transitions.iter().enumerate() {
            if row.len() != n {
                return Err(MarkovChainError::RowLengthMismatch {
                    row: i,
                    len: row.len(),
                    expected: n,
                });
            }
            let mut sum = 0.0;
            for (j, &p) in row.iter().enumerate() {
                if p < 0.0 {
                    return Err(MarkovChainError::NegativeProbability {
                        row: i,
                        col: j,
                        value: p,
                    });
                }
                sum += p;
            }
            if (sum - 1.0).abs() > STOCHASTIC_TOLERANCE {
                return Err(MarkovChainError::RowNotStochastic { row: i, sum });
            }
        }
        Ok(Self { labels, transitions, initial_idx, initial_rng })
    }

    /// Number of states in the chain.
    pub fn num_states(&self) -> usize {
        self.labels.len()
    }
}

impl<S, R> StateMachine for MarkovChain<S, R>
where
    S: Clone,
    R: Rng,
{
    /// The chain is input-less; drive with `Runner::run(n)`.
    type Input = ();
    type Output = S;
    /// `(current_state_index, rng_state)`.
    type State = (usize, R);

    fn start_state(&self) -> (usize, R) {
        (self.initial_idx, self.initial_rng.clone())
    }

    fn next_values(&self, state: &(usize, R), _: &()) -> ((usize, R), S) {
        let (idx, rng) = state;
        let mut rng = rng.clone();
        let u = rng.next_f64();
        let row = &self.transitions[*idx];

        // Inverse CDF: find the first j where cumsum >= u. Using strictly
        // less-than below handles the u=1.0 edge case via the fallback to
        // the last index (which should be unreachable given f64 in [0,1)).
        let mut cum = 0.0;
        let mut next_idx = row.len() - 1;
        for (j, &p) in row.iter().enumerate() {
            cum += p;
            if u < cum {
                next_idx = j;
                break;
            }
        }
        ((next_idx, rng), self.labels[next_idx].clone())
    }
}
