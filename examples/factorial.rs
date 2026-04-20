//! Factorial sequence `1, 1, 2, 6, 24, 120, ...` via `Cascade(counter,
//! Feedback2(Cascade(Multiplier, Delay(1))))`. §4.2.3.5.
//!
//! The counter emits `1, 2, 3, ...`; the inner feedback2 loop multiplies the
//! counter value by the previous running product (carried through the delay),
//! yielding factorials.

use state_machines_rs::{
    Runner, SMExt,
    primitives::{Delay, Increment, Multiplier},
};

fn main() {
    let one: Option<i64> = Some(1);

    // makeCounter(1, 1): outputs 1, 2, 3, 4, ...
    let counter = Increment::new(one).cascade(Delay::new(one)).feedback();

    // Inner: takes a pair (counter_value, feedback_product), multiplies, delays.
    let running_product = Multiplier::<Option<i64>>::new()
        .cascade(Delay::new(one))
        .feedback2();

    let fact = counter.cascade(running_product);

    let out: Vec<_> = Runner::new(fact).run(10);
    println!("{:?}", out.iter().flatten().collect::<Vec<_>>());
    // Expected: [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880]
    assert_eq!(out.into_iter().flatten().collect::<Vec<_>>(), [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880]);
}
