//! Integration tests for `SumLastN` and `MovingAverageN`.

use state_machines_rs::{
    Runner,
    primitives::{
        MovingAverageN, MovingAverageNError, StdDevLastN, SumLastN, SumLastNError,
        VarianceLastN, VarianceLastNError,
    },
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

// ---------- VarianceLastN / StdDevLastN ----------

fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

#[test]
fn population_variance_constant_input_is_zero() {
    let m = VarianceLastN::new_population_with(4).unwrap();
    let out = Runner::new(m).transduce([3.0, 3.0, 3.0, 3.0, 3.0, 3.0]);
    for v in out {
        assert_eq!(v, 0.0);
    }
}

#[test]
fn population_variance_matches_hand_computation() {
    // After the window saturates at n=3 over [2, 4, 4, 4, 5, 5, 7, 9]:
    //   window [2,4,4]     mean 10/3 ≈ 3.333  var = ((2-3.333)² + 2·(4-3.333)²)/3 ≈ 0.8889
    //   window [4,4,4]     var = 0
    //   window [4,4,5]     mean 13/3 ≈ 4.333  var = 2·(4-4.333)² + (5-4.333)² / 3 ≈ 0.2222
    //   window [4,5,5]     same geometry, var ≈ 0.2222
    //   window [5,5,7]     mean 17/3 ≈ 5.667  var ≈ 0.8889
    //   window [5,7,9]     mean 7, var ≈ 2.6667
    let m = VarianceLastN::new_population_with(3).unwrap();
    let out = Runner::new(m).transduce([2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    let expected_full_window = [0.888_888_9, 0.0, 0.222_222_2, 0.222_222_2, 0.888_888_9, 2.666_666_7];
    for (got, want) in out[2..].iter().zip(expected_full_window.iter()) {
        assert!(
            approx_eq(*got, *want, 1e-6),
            "variance {:?} did not match {:?}",
            got, want
        );
    }
}

#[test]
fn sample_variance_emits_nan_until_two_samples() {
    let m = VarianceLastN::new_sample_with(4).unwrap();
    let out = Runner::new(m).transduce([5.0, 5.0, 5.0]);
    assert!(out[0].is_nan());
    // Two samples both equal to 5 → variance exactly zero.
    assert_eq!(out[1], 0.0);
    assert_eq!(out[2], 0.0);
}

#[test]
fn sample_variance_applies_bessel_correction() {
    // Full window [1, 2, 3, 4] at n=4: mean 2.5, sum of squared deviations
    // = 1.5² + 0.5² + 0.5² + 1.5² = 5. Sample var = 5/3 ≈ 1.6666…
    let m = VarianceLastN::new_sample_with(4).unwrap();
    let out = Runner::new(m).transduce([1.0, 2.0, 3.0, 4.0]);
    assert!(approx_eq(*out.last().unwrap(), 5.0 / 3.0, 1e-9));
}

#[test]
fn std_dev_is_sqrt_of_variance() {
    let var = VarianceLastN::new_population_with(5).unwrap();
    let std = StdDevLastN::new_population_with(5).unwrap();
    let data = [1.0, 3.0, 7.0, 2.0, 4.0, 6.0, 5.0, 8.0, 2.0, 1.0];
    let vout = Runner::new(var).transduce(data);
    let sout = Runner::new(std).transduce(data);
    for (v, s) in vout.iter().zip(sout.iter()) {
        assert!(approx_eq(v.sqrt(), *s, 1e-12));
    }
}

#[test]
fn variance_last_n_rejects_zero_window() {
    assert_eq!(
        VarianceLastN::new_population_with(0).err().unwrap(),
        VarianceLastNError::ZeroWindow
    );
}

#[test]
fn sample_variance_rejects_single_slot_window() {
    assert_eq!(
        VarianceLastN::new_sample_with(1).err().unwrap(),
        VarianceLastNError::SampleWindowTooSmall
    );
}
