//! Flagship chapter examples at scale: counter, fibonacci, factorial,
//! wall-follower, and TSM-based text sequencing. These exercise the full stack
//! (feedback, feedback2, cascade, parallel, DynTSM dispatch) on realistic
//! machines.

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use state_machines_rs::{
    Runner, SMExt, StateMachine,
    primitives::{Adder, Delay, Increment, Multiplier},
    tsm::{CharTSM, DynTSM, Repeat, Sequence, into_dyn},
};

const STEPS: &[usize] = &[1_000, 10_000, 100_000];

fn bench_counter(c: &mut Criterion) {
    let mut group = c.benchmark_group("counter_feedback");
    for &n in STEPS {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || {
                    let m = Increment::new(Some(1i64))
                        .cascade(Delay::new(Some(0i64)))
                        .feedback();
                    Runner::new(m)
                },
                |mut r| {
                    for _ in 0..n {
                        black_box(r.step(()));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_fibonacci(c: &mut Criterion) {
    // i64 Fibonacci overflows around step 92 — for benchmarking speed we don't
    // care about correct values past that; the machine keeps running.
    // Cap the step count at 90 so the values stay meaningful, and report
    // per-step throughput in a separate bench at higher counts using u128.
    let mut group = c.benchmark_group("fibonacci_i128");
    for &n in &[90usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || {
                    let one: Option<i128> = Some(1);
                    let zero: Option<i128> = Some(0);
                    let m = Delay::new(one)
                        .parallel(Delay::new(one).cascade(Delay::new(zero)))
                        .cascade(Adder::<Option<i128>>::new())
                        .feedback();
                    Runner::new(m)
                },
                |mut r| {
                    for _ in 0..n {
                        black_box(r.step(()));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_factorial(c: &mut Criterion) {
    // 20! overflows i64; use f64 for throughput measurement (floating-point
    // ops are representative of numeric primitive cost, and factorial values
    // quickly become irrelevant anyway).
    let mut group = c.benchmark_group("factorial_f64");
    for &n in &[100usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || {
                    let one: Option<f64> = Some(1.0);
                    let counter = Increment::new(one)
                        .cascade(Delay::new(one))
                        .feedback();
                    let product = Multiplier::<Option<f64>>::new()
                        .cascade(Delay::new(one))
                        .feedback2();
                    Runner::new(counter.cascade(product))
                },
                |mut r| {
                    for _ in 0..n {
                        black_box(r.step(()));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_wall_follower(c: &mut Criterion) {
    struct Controller {
        k: f64,
        d_desired: f64,
    }
    impl StateMachine for Controller {
        type Input = Option<f64>;
        type Output = Option<f64>;
        type State = Option<f64>;
        fn start_state(&self) -> Option<f64> { None }
        fn next_values(&self, _: &Option<f64>, i: &Option<f64>) -> (Option<f64>, Option<f64>) {
            let v = i.map(|d| self.k * (self.d_desired - d));
            (v, v)
        }
    }
    struct World {
        dt: f64,
        init: f64,
    }
    impl StateMachine for World {
        type Input = Option<f64>;
        type Output = Option<f64>;
        type State = Option<f64>;
        fn start_state(&self) -> Option<f64> { Some(self.init) }
        fn next_values(&self, s: &Option<f64>, i: &Option<f64>) -> (Option<f64>, Option<f64>) {
            let ns = match (s, i) {
                (Some(d), Some(v)) => Some(d - self.dt * v),
                (Some(d), None) => Some(*d),
                _ => None,
            };
            (ns, *s)
        }
    }

    let mut group = c.benchmark_group("wall_follower");
    for &n in STEPS {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || {
                    let sys = Controller { k: -1.5, d_desired: 1.0 }
                        .cascade(World { dt: 0.1, init: 5.0 })
                        .feedback();
                    Runner::new(sys)
                },
                |mut r| {
                    for _ in 0..n {
                        black_box(r.step(()));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

/// TSM dispatch cost: each step goes through `Box<dyn DynTSM>` indirection.
/// Compare per-step cost against a cold Runner call on a primitive.
fn bench_tsm_sequence(c: &mut Criterion) {
    let mut group = c.benchmark_group("tsm_sequence");

    // One `CharTSM` per emitted character; Sequence dispatches through its
    // boxed trait objects.
    for &n in &[100usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || {
                    let machines: Vec<Box<dyn DynTSM<(), char>>> =
                        (0..n).map(|i| into_dyn(CharTSM::new((b'a' + (i % 26) as u8) as char))).collect();
                    Sequence::new(machines)
                },
                |mut seq| {
                    while !seq.is_done() {
                        black_box(seq.step(&()));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }

    // Repeat(CharTSM) N times — lightest possible TSM dispatch loop.
    for &n in &[1_000usize, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("repeat_chartsm", n), &n, |b, &n| {
            b.iter_batched(
                || Repeat::times(into_dyn(CharTSM::new('x')), n),
                |mut r| {
                    while !r.is_done() {
                        black_box(r.step(&()));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_counter,
    bench_fibonacci,
    bench_factorial,
    bench_wall_follower,
    bench_tsm_sequence,
);
criterion_main!(benches);
