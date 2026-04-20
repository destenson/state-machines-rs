//! Two running-sum accumulators, showing they are not quite identical.
//!
//! * `Accumulator` is a pure primitive: `output[t] = sum(input[0..=t])`.
//! * `FeedbackAdd(Delay(0), Wire)` (from §4.2.3.4 of the chapter) produces
//!   `output[t] = sum(input[0..=t-1])` — one step of delay is baked in because
//!   the feedback branch uses the *previous* output.

use state_machines_rs::{
    Runner, SMExt,
    primitives::{Accumulator, Delay, Wire},
};

fn main() {
    let input: Vec<i64> = (0..10).collect();

    let direct = Runner::new(Accumulator::<i64>::new(0)).transduce(input.clone());
    println!("Accumulator:               {:?}", direct);
    assert_eq!(direct, [0, 1, 3, 6, 10, 15, 21, 28, 36, 45]);

    let feedback_sum: Vec<i64> =
        Runner::new(Delay::new(Some(0i64)).feedback_add(Wire::<Option<i64>>::new()))
            .transduce(input.iter().map(|x| Some(*x)))
            .into_iter()
            .flatten()
            .collect();
    println!("FeedbackAdd(Delay, Wire):  {:?}", feedback_sum);
    assert_eq!(feedback_sum, vec![0, 0, 1, 3, 6, 10, 15, 21, 28, 36]);

    // They differ by one step of delay: direct[t] == feedback_sum[t+1].
    assert_eq!(&direct[..direct.len() - 1], &feedback_sum[1..]);
}
