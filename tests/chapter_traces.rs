//! Pin the library's behavior to the exact I/O traces given in chapter 4 of
//! MIT 6.01. Each test cites the page of the source PDF.

use state_machines_rs::{
    Runner, SMExt, StateMachine,
    combinators::{Feedback, Mux, Switch},
    primitives::{
        ABC, Accumulator, AbcOutput, Adder, Average2, Delay, Gain, Increment, Multiplier,
        Negation, SumLast3, UpDown, UpDownInput, Wire,
    },
    tsm::{CharTSM, ConsumeFiveValues, DynTSM, Repeat, RepeatUntil, Sequence, Until, into_dyn},
};

// ---------- primitives ----------

#[test]
fn accumulator_matches_chapter_p123() {
    // Input  [100, -3, 4, -123, 10]; output [100, 97, 101, -22, -12].
    let mut m = Runner::new(Accumulator::<i64>::new(0));
    let out = m.transduce([100, -3, 4, -123, 10]);
    assert_eq!(out, [100, 97, 101, -22, -12]);
}

#[test]
fn abc_acceptor_p121() {
    // Input  'a','b','c','a','c','a','b' -> T,T,T,T,F,F,F
    let mut m = Runner::new(ABC);
    let out = m.transduce(['a', 'b', 'c', 'a', 'c', 'a', 'b']);
    let bools: Vec<bool> = out.into_iter().map(|o| matches!(o, AbcOutput::Accept)).collect();
    assert_eq!(bools, [true, true, true, true, false, false, false]);
}

#[test]
fn updown_counter_p122() {
    use UpDownInput::*;
    let mut m = Runner::new(UpDown);
    let out = m.transduce([Up, Up, Up, Down, Down, Up]);
    assert_eq!(out, [1, 2, 3, 2, 1, 2]);
}

#[test]
fn delay_p122() {
    // Delay(7) on [3, 1, 2, 5, 9] -> [7, 3, 1, 2, 5].
    let mut m = Runner::new(Delay::<i64>::new(7));
    let out = m.transduce([3, 1, 2, 5, 9]);
    assert_eq!(out, [7, 3, 1, 2, 5]);
}

#[test]
fn average2_p131() {
    // Input [10, 5, 2, 10]; output [5.0, 7.5, 3.5, 6.0].
    let mut m = Runner::new(Average2::default());
    let out = m.transduce([10.0, 5.0, 2.0, 10.0]);
    assert_eq!(out, [5.0, 7.5, 3.5, 6.0]);
}

#[test]
fn sum_last3_p132() {
    let mut m = Runner::new(SumLast3::<i64>::default());
    let out = m.transduce([2, 1, 3, 4, 10, 1, 2, 1, 5]);
    assert_eq!(out, [2, 3, 6, 8, 17, 15, 13, 4, 8]);
}

#[test]
fn gain_p128() {
    let mut m = Runner::new(Gain::new(3.0));
    let out = m.transduce([1.1, -2.0, 100.0, 5.0]);
    assert_eq!(out, [3.3000000000000003, -6.0, 300.0, 15.0]);
}

// ---------- combinators ----------

#[test]
fn cascade_two_delays_p136() {
    // Cascade(Delay(99), Delay(22)) on [3, 8, 2, 4, 6, 5]
    //   -> [22, 99, 3, 8, 2, 4].
    let m = Delay::<i64>::new(99).cascade(Delay::new(22));
    let mut r = Runner::new(m);
    let out = r.transduce([3, 8, 2, 4, 6, 5]);
    assert_eq!(out, [22, 99, 3, 8, 2, 4]);
}

#[test]
fn counter_via_feedback_p141() {
    // makeCounter(3, 2): Cascade(Increment(2), Delay(3)) under feedback.
    // Per the chapter's verbose trace (p.141) the first emitted value is the
    // delay's initial state, 3, then 5, 7, ...
    let m = Increment::new(Some(2i64))
        .cascade(Delay::new(Some(3i64)))
        .feedback();
    let out: Vec<i64> = Runner::new(m).run(10).into_iter().flatten().collect();
    assert_eq!(out, [3, 5, 7, 9, 11, 13, 15, 17, 19, 21]);
}

#[test]
fn fibonacci_p143() {
    let one: Option<i64> = Some(1);
    let zero: Option<i64> = Some(0);
    let fib = Delay::new(one)
        .parallel(Delay::new(one).cascade(Delay::new(zero)))
        .cascade(Adder::new())
        .feedback();
    let out: Vec<i64> = Runner::new(fib).run(10).into_iter().flatten().collect();
    assert_eq!(out, [1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);
}

#[test]
fn factorial_p146() {
    let one: Option<i64> = Some(1);
    let counter = Increment::new(one).cascade(Delay::new(one)).feedback();
    let product = Multiplier::<Option<i64>>::new()
        .cascade(Delay::new(one))
        .feedback2();
    let fact = counter.cascade(product);
    let out: Vec<i64> = Runner::new(fact).run(10).into_iter().flatten().collect();
    assert_eq!(out, [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880]);
}

#[test]
fn feedback_add_makes_accumulator_p145() {
    // FeedbackAdd(R(0), Wire) acts as a running-sum accumulator.
    let m = Delay::new(Some(0i64)).feedback_add(Wire::<Option<i64>>::new());
    let out: Vec<i64> = Runner::new(m)
        .transduce((0..10).map(Some))
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(out, [0, 0, 1, 3, 6, 10, 15, 21, 28, 36]);
}

#[test]
fn alternate_true_false_via_negation_feedback_exercise_4_4() {
    // Negation has a direct input-output dependence, so it needs a Delay in
    // the loop to be valid in `Feedback`.
    let m = Negation.cascade(Delay::new(Some(false))).feedback();
    let out: Vec<bool> = Runner::new(m).run(6).into_iter().flatten().collect();
    // Starting with Delay(false), output alternates: false, true, false, ...
    assert_eq!(out, [false, true, false, true, false, true]);
}

#[test]
fn switch_vs_mux_exercise_4_11() {
    // Switch with two Accumulators, threshold > 100: only one accumulates at a time.
    let switch = Switch::new(
        |x: &i64| *x > 100,
        Accumulator::<i64>::new(0),
        Accumulator::<i64>::new(0),
    );
    let out_switch = Runner::new(switch).transduce([2, 3, 4, 200, 300, 400, 1, 2, 3]);
    assert_eq!(out_switch, [2, 5, 9, 200, 500, 900, 10, 12, 15]);

    // Mux: both accumulate always; output selects which to emit.
    let mux = Mux::new(
        |x: &i64| *x > 100,
        Accumulator::<i64>::new(0),
        Accumulator::<i64>::new(0),
    );
    let out_mux = Runner::new(mux).transduce([2, 3, 4, 200, 300, 400, 1, 2, 3]);
    // Both accumulators track the same running sum; output is m1 when the
    // input exceeds 100, m2 otherwise.
    assert_eq!(out_mux, [2, 5, 9, 209, 509, 909, 910, 912, 915]);
}

// ---------- TSMs ----------

#[test]
fn consume_five_values_p153() {
    let m = ConsumeFiveValues::<i64>::new();
    let mut r = Runner::new(m);
    let out = r.transduce([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(out, [None, None, None, None, Some(15)]);
    assert!(r.is_done());
}

#[test]
fn repeat_chars_p154() {
    let mut r = Repeat::times(into_dyn(CharTSM::new('a')), 4);
    let mut out = Vec::new();
    while !r.is_done() {
        out.push(r.step(&()));
    }
    assert_eq!(out, ['a', 'a', 'a', 'a']);
}

#[test]
fn sequence_abc_p156() {
    let abc: Vec<Box<dyn DynTSM<(), char>>> = ['a', 'b', 'c']
        .into_iter()
        .map(|c| into_dyn(CharTSM::new(c)))
        .collect();
    let mut seq = Sequence::new(abc);
    let mut out = Vec::new();
    while !seq.is_done() {
        out.push(seq.step(&()));
    }
    assert_eq!(out, ['a', 'b', 'c']);
}

#[test]
fn sequence_inside_repeat_p156() {
    let make_abc = || {
        let v: Vec<Box<dyn DynTSM<(), char>>> = ['a', 'b', 'c']
            .into_iter()
            .map(|c| into_dyn(CharTSM::new(c)))
            .collect();
        Sequence::new(v)
    };
    let mut r = Repeat::times(Box::new(make_abc()), 3);
    let mut out = Vec::new();
    while !r.is_done() {
        out.push(r.step(&()));
    }
    assert_eq!(out, ['a', 'b', 'c', 'a', 'b', 'c', 'a', 'b', 'c']);
}

#[test]
fn repeat_until_and_until_p158() {
    // Until(greaterThan10, ConsumeFiveValues) on 0..20: runs CFV once,
    // terminates when it produces a value > 10 on its fifth step (value 10).
    // Chapter says output = [None, None, None, None, 10].
    let mut m = Until::new(
        |x: &i64| *x > 10,
        into_dyn(ConsumeFiveValues::<i64>::new()),
    );
    let mut out = Vec::new();
    for i in 0..20 {
        if m.is_done() { break; }
        out.push(m.step(&i));
    }
    assert_eq!(out, [None, None, None, None, Some(10)]);

    // RepeatUntil runs CFV multiple times, terminates only when done AND cond true.
    // Chapter's output is longer; the terminating condition becomes true on
    // the step producing Some(60) (the 15th).
    let mut m = RepeatUntil::new(
        |x: &i64| *x > 10,
        into_dyn(ConsumeFiveValues::<i64>::new()),
    );
    let mut out = Vec::new();
    for i in 0..20 {
        if m.is_done() { break; }
        out.push(m.step(&i));
    }
    // The last element must be Some(60), reached after three completed runs.
    assert_eq!(out.last(), Some(&Some(60)));
}

// ---------- ensure StateMachine trait is object-usable via Runner ----------

#[test]
fn runner_state_inspection() {
    let mut r = Runner::new(Accumulator::<i64>::new(10));
    r.step(5);
    r.step(-2);
    assert_eq!(*r.state(), 13);
}

// keep trait in scope
#[allow(dead_code)]
fn _trait_in_scope<M: StateMachine>(_: M) -> () {
    let _ = Feedback::<M>::new;
}
