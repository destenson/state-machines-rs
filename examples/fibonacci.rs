//! Fibonacci via feedback, cascade, and parallel (§4.2.3.2). Output:
//! `1, 2, 3, 5, 8, 13, 21, 34, 55, 89`. The first element is taken as `F(2)=1`
//! rather than `F(1)=1` — an artifact of the initial delay values.
//!
//! Topology (p. 143 figure 4.8):
//! ```text
//!          ┌─── Delay(1) ───────────────┐
//!  input ──┤                            +── output
//!          └── Delay(1) ── Delay(0) ────┘
//! ```
//! (wrapped in Feedback; input-less)

use state_machines_rs::{
    Runner, SMExt,
    primitives::{Adder, Delay},
};

fn main() {
    let one: Option<i64> = Some(1);
    let zero: Option<i64> = Some(0);

    // Parallel branch: top is Delay(1), bottom is Cascade(Delay(1), Delay(0)).
    // The output pair feeds into an Adder, which sums the two lagged values.
    let fib = Delay::new(one)
        .parallel(Delay::new(one).cascade(Delay::new(zero)))
        .cascade(Adder::new())
        .feedback();

    let out: Vec<_> = Runner::new(fib).run(10);
    println!("{:?}", out.iter().flatten().collect::<Vec<_>>());
    // Expected: [1, 2, 3, 5, 8, 13, 21, 34, 55, 89]
    assert_eq!(out.into_iter().flatten().collect::<Vec<_>>(), [1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);
}
