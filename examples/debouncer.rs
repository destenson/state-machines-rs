//! N-of-N debouncer — accept a new input value only after it has been stable
//! for `n` consecutive samples. Standard firmware pattern for pushbuttons,
//! limit switches, and glitchy optical sensors.
//!
//! State is the currently committed output plus a "how many times have we
//! seen the candidate in a row" counter. The candidate itself isn't stored
//! separately: if it changes mid-sequence, the counter resets.

use state_machines_rs::{Runner, StateMachine};

pub struct Debouncer<T> {
    n: u32,
    initial: T,
}

impl<T: Clone> Debouncer<T> {
    pub fn new(n: u32, initial: T) -> Self {
        assert!(n >= 1);
        Self { n, initial }
    }
}

/// State: `(committed_value, candidate_run_length)`.
impl<T: Clone + PartialEq> StateMachine for Debouncer<T> {
    type Input = T;
    type Output = T;
    type State = (T, u32);

    fn start_state(&self) -> (T, u32) {
        (self.initial.clone(), 0)
    }

    fn next_values(&self, state: &(T, u32), input: &T) -> ((T, u32), T) {
        let (committed, run_len) = state;
        if *input == *committed {
            // Input matches the held value — nothing changes; reset counter.
            ((committed.clone(), 0), committed.clone())
        } else {
            let new_run = run_len + 1;
            if new_run >= self.n {
                // Candidate has been stable long enough — commit it.
                ((input.clone(), 0), input.clone())
            } else {
                ((committed.clone(), new_run), committed.clone())
            }
        }
    }
}

fn main() {
    // Simulated noisy pushbutton: genuine press at t=10, held through t=60,
    // with a handful of single-sample glitches scattered elsewhere. '_'=open,
    // 'X'=closed.
    let press_window = 10..60;
    let glitches = [3, 5, 65, 70, 72, 95];
    let n = 100;

    let raw: Vec<bool> = (0..n)
        .map(|t| {
            let real = press_window.contains(&t);
            let glitch = glitches.contains(&t);
            real ^ glitch
        })
        .collect();

    // Require 3 consecutive identical samples before accepting a change.
    let clean = Runner::new(Debouncer::new(3, false)).transduce(raw.iter().copied());

    let raw_transitions = raw.windows(2).filter(|w| w[0] != w[1]).count();
    let clean_transitions = clean.windows(2).filter(|w| w[0] != w[1]).count();

    println!("raw transitions:    {}", raw_transitions);
    println!("debounced transitions: {}", clean_transitions);

    // Print a diff row for visual inspection.
    print!("raw:     ");
    for b in &raw { print!("{}", if *b { 'X' } else { '_' }); }
    println!();
    print!("clean:   ");
    for b in &clean { print!("{}", if *b { 'X' } else { '_' }); }
    println!();

    // The real press and release are the only transitions that should
    // survive debouncing.
    assert_eq!(clean_transitions, 2);
    // And the raw stream must actually *have* glitches for this to be a
    // meaningful test.
    assert!(raw_transitions > clean_transitions + 4);
}
