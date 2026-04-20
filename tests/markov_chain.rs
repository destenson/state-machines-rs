use state_machines_rs::{
    Runner, SplitMix64,
    primitives::{MarkovChain, MarkovChainError},
};

#[test]
fn rejects_nonsquare_matrix() {
    let err = MarkovChain::new_with(
        vec!["a", "b"],
        vec![vec![0.5, 0.5]], // one row for two labels
        0,
        SplitMix64::new(0),
    )
    .err()
    .expect("should have errored");
    assert_eq!(err, MarkovChainError::ShapeMismatch { labels: 2, rows: 1 });
}

#[test]
fn rejects_row_length_mismatch() {
    let err = MarkovChain::new_with(
        vec!["a", "b"],
        vec![vec![0.5, 0.5], vec![1.0]], // second row too short
        0,
        SplitMix64::new(0),
    )
    .err()
    .expect("should have errored");
    assert_eq!(
        err,
        MarkovChainError::RowLengthMismatch { row: 1, len: 1, expected: 2 }
    );
}

#[test]
fn rejects_non_stochastic_row() {
    let err = MarkovChain::new_with(
        vec!["a", "b"],
        vec![vec![0.4, 0.5], vec![0.5, 0.5]], // first row sums to 0.9
        0,
        SplitMix64::new(0),
    )
    .err()
    .expect("should have errored");
    match err {
        MarkovChainError::RowNotStochastic { row: 0, sum } => {
            assert!((sum - 0.9).abs() < 1e-12);
        }
        other => panic!("unexpected error {:?}", other),
    }
}

#[test]
fn rejects_negative_probability() {
    let err = MarkovChain::new_with(
        vec!["a", "b"],
        vec![vec![-0.1, 1.1], vec![0.5, 0.5]],
        0,
        SplitMix64::new(0),
    )
    .err()
    .expect("should have errored");
    assert!(matches!(
        err,
        MarkovChainError::NegativeProbability { row: 0, col: 0, .. }
    ));
}

#[test]
fn rejects_out_of_range_initial() {
    let err = MarkovChain::new_with(
        vec!["a", "b"],
        vec![vec![0.5, 0.5], vec![0.5, 0.5]],
        5,
        SplitMix64::new(0),
    )
    .err()
    .expect("should have errored");
    assert_eq!(
        err,
        MarkovChainError::InitialIdxOutOfRange { idx: 5, n_states: 2 }
    );
}

#[test]
fn absorbing_state_stays_absorbing() {
    // Two states; state 1 is absorbing (self-loop with prob 1).
    let mc = MarkovChain::new_with(
        vec![0u8, 1u8],
        vec![vec![0.5, 0.5], vec![0.0, 1.0]],
        0,
        SplitMix64::new(0xDEADBEEF),
    )
    .unwrap();

    let trajectory: Vec<u8> = Runner::new(mc).run(200);
    // Once we hit state 1 we stay there forever.
    let first_one = trajectory.iter().position(|&x| x == 1).expect("must enter state 1");
    assert!(trajectory[first_one..].iter().all(|&x| x == 1));
}

#[test]
fn same_seed_same_trajectory() {
    // Determinism property: two runs with the same seed give the same output.
    let make = || {
        MarkovChain::new_with(
            vec!["A", "B", "C"],
            vec![
                vec![0.3, 0.4, 0.3],
                vec![0.2, 0.5, 0.3],
                vec![0.4, 0.3, 0.3],
            ],
            0,
            SplitMix64::new(1234),
        )
        .unwrap()
    };
    let a: Vec<_> = Runner::new(make()).run(50);
    let b: Vec<_> = Runner::new(make()).run(50);
    assert_eq!(a, b);
}
