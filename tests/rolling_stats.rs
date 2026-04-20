//! Integration tests for `SumLastN` and `MovingAverageN`.

use state_machines_rs::{
    Runner,
    primitives::{MovingAverageN, MovingAverageNError, SumLastN, SumLastNError},
};

#[test]
fn sum_last_3_matches_chapter_trace() {
    // SumLastN(3) is the generalized form of the chapter's SumLast3 toy.
    // Input [2, 1, 3, 4, 10, 1, 2, 1, 5] — same sequence as the chapter
    // test. Note: chapter's SumLast3 seeds state with (0, 0) so its first
    // two outputs fold in implicit zeros: 2, 3, 6, 8, 17, 15, 13, 4, 8.
    // SumLastN's partial-sum warm-up gives the same values, because sum
    // over "whatever we've seen so far" equals sum with zero-padded
    // prefix — the trailing zeros don't change the sum.
    let m = SumLastN::<i64>::new_with(3).unwrap();
    let out = Runner::new(m).transduce([2, 1, 3, 4, 10, 1, 2, 1, 5]);
    assert_eq!(out, [2, 3, 6, 8, 17, 15, 13, 4, 8]);
}

#[test]
fn sum_last_n_rolling_behavior() {
    let m = SumLastN::<i64>::new_with(4).unwrap();
    let out = Runner::new(m).transduce([1, 2, 3, 4, 5, 6, 7, 8]);
    // Expected (partial during warm-up, then strict rolling 4):
    //   1, 3, 6, 10, 14, 18, 22, 26
    assert_eq!(out, [1, 3, 6, 10, 14, 18, 22, 26]);
}

#[test]
fn sum_last_n_window_of_one_is_passthrough() {
    let m = SumLastN::<i64>::new_with(1).unwrap();
    let out = Runner::new(m).transduce([5, -3, 10, 42]);
    assert_eq!(out, [5, -3, 10, 42]);
}

#[test]
fn sum_last_n_rejects_zero_window() {
    let err = SumLastN::<i64>::new_with(0).err().unwrap();
    assert_eq!(err, SumLastNError::ZeroWindow);
}

#[test]
fn moving_average_n_matches_chapter_trace_after_warmup() {
    // MovingAverageN(2) over [10, 5, 2, 10]:
    //   t=0: partial average = 10/1 = 10.0
    //   t=1: (10+5)/2        = 7.5
    //   t=2: (5+2)/2         = 3.5
    //   t=3: (2+10)/2        = 6.0
    // The chapter's Average2 seeds state with 0, producing 5.0 at t=0
    // instead; MovingAverageN emits the partial average at warm-up by
    // design (see module doc).
    let m = MovingAverageN::new_with(2).unwrap();
    let out = Runner::new(m).transduce([10.0, 5.0, 2.0, 10.0]);
    assert_eq!(out, [10.0, 7.5, 3.5, 6.0]);
}

#[test]
fn moving_average_n_five_sample_window() {
    let m = MovingAverageN::new_with(5).unwrap();
    let out = Runner::new(m).transduce([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    // After step 4, the window is full; sliding over [1..5], [2..6], [3..7]
    // gives means 3, 4, 5. Warm-up means are 1, 1.5, 2, 2.5.
    assert_eq!(out, [1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0]);
}

#[test]
fn moving_average_n_rejects_zero_window() {
    let err = MovingAverageN::new_with(0).err().unwrap();
    assert_eq!(err, MovingAverageNError::ZeroWindow);
}
