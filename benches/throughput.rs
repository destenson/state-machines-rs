//! Per-step throughput of primitives and basic combinators. Each bench drives
//! a machine for `N` steps and reports throughput in elements.
//!
//! Reading the numbers: `time / N` gives ns-per-step; compare primitives
//! against each other to see the cost of stateful work, and against
//! combinators to see composition overhead.

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use state_machines_rs::{
    Runner, SMExt,
    primitives::{Accumulator, Adder, Average2, Delay, Gain, Increment, SumLast3, Wire},
};

const SIZES: &[usize] = &[1_000, 100_000, 1_000_000];

fn bench_accumulator(c: &mut Criterion) {
    let mut group = c.benchmark_group("accumulator");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || (Runner::new(Accumulator::<i64>::new(0)), (0..n as i64).collect::<Vec<_>>()),
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_delay(c: &mut Criterion) {
    let mut group = c.benchmark_group("delay");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || (Runner::new(Delay::<i64>::new(0)), (0..n as i64).collect::<Vec<_>>()),
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_gain(c: &mut Criterion) {
    let mut group = c.benchmark_group("gain");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || (Runner::new(Gain::new(2.5f64)), (0..n).map(|i| i as f64).collect::<Vec<_>>()),
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_sum_last3(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_last3");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || (Runner::new(SumLast3::<i64>::default()), (0..n as i64).collect::<Vec<_>>()),
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_average2(c: &mut Criterion) {
    let mut group = c.benchmark_group("average2");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || (Runner::new(Average2::default()), (0..n).map(|i| i as f64).collect::<Vec<_>>()),
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

/// Compare a single `Accumulator` against a 3-machine `Cascade` of
/// `Accumulator -> Gain -> Accumulator`. Same per-step work should have
/// ~linear cost in depth if composition is free.
fn bench_cascade_depth(c: &mut Criterion) {
    const N: usize = 100_000;
    let mut group = c.benchmark_group("cascade_depth");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function("depth_1_accumulator", |b| {
        b.iter_batched(
            || (Runner::new(Accumulator::<i64>::new(0)), (0..N as i64).collect::<Vec<_>>()),
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("depth_3_accum_gain_accum", |b| {
        b.iter_batched(
            || {
                let m = Accumulator::<i64>::new(0)
                    .cascade(Gain::new(1i64))
                    .cascade(Accumulator::<i64>::new(0));
                (Runner::new(m), (0..N as i64).collect::<Vec<_>>())
            },
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("depth_5_delays", |b| {
        b.iter_batched(
            || {
                let m = Delay::<i64>::new(0)
                    .cascade(Delay::new(0))
                    .cascade(Delay::new(0))
                    .cascade(Delay::new(0))
                    .cascade(Delay::new(0));
                (Runner::new(m), (0..N as i64).collect::<Vec<_>>())
            },
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Feedback adds two inner-machine probes per step. Measure the tax against
/// the same inner machine run open-loop.
fn bench_feedback_overhead(c: &mut Criterion) {
    const N: usize = 100_000;
    let mut group = c.benchmark_group("feedback_overhead");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function("open_loop_incr_delay", |b| {
        b.iter_batched(
            || {
                let m = Increment::new(Some(1i64)).cascade(Delay::new(Some(0i64)));
                (Runner::new(m), (0..N).map(|i| Some(i as i64)).collect::<Vec<_>>())
            },
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("feedback_counter", |b| {
        b.iter_batched(
            || {
                let m = Increment::new(Some(1i64))
                    .cascade(Delay::new(Some(0i64)))
                    .feedback();
                Runner::new(m)
            },
            |mut r| {
                for _ in 0..N {
                    black_box(r.step(()));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("feedback_add_wire", |b| {
        // Running-sum via FeedbackAdd(Delay, Wire).
        b.iter_batched(
            || {
                let m = Delay::new(Some(0i64)).feedback_add(Wire::<Option<i64>>::new());
                (Runner::new(m), (0..N).map(|i| Some(i as i64)).collect::<Vec<_>>())
            },
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Parallel composition runs both branches every step; output is a pair.
fn bench_parallel(c: &mut Criterion) {
    const N: usize = 100_000;
    let mut group = c.benchmark_group("parallel");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function("parallel_two_accumulators", |b| {
        b.iter_batched(
            || {
                let m = Accumulator::<i64>::new(0).parallel(Accumulator::<i64>::new(0));
                (Runner::new(m), (0..N as i64).collect::<Vec<_>>())
            },
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("parallel_add_two_accumulators", |b| {
        b.iter_batched(
            || {
                let m = Accumulator::<i64>::new(0).parallel_add(Accumulator::<i64>::new(0));
                (Runner::new(m), (0..N as i64).collect::<Vec<_>>())
            },
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Adder under feedback2 — the inner kernel for factorial and similar.
fn bench_adder_feedback2(c: &mut Criterion) {
    const N: usize = 100_000;
    let mut group = c.benchmark_group("feedback2");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function("running_sum_via_feedback2", |b| {
        b.iter_batched(
            || {
                // (external, feedback) -> add -> delay -> feedback. A Kahan-free
                // running sum driven by Feedback2.
                let inner = Adder::<Option<i64>>::new().cascade(Delay::new(Some(0i64)));
                let m = inner.feedback2();
                (Runner::new(m), (0..N).map(|i| Some(i as i64)).collect::<Vec<_>>())
            },
            |(mut r, input)| {
                for x in input {
                    black_box(r.step(x));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_accumulator,
    bench_delay,
    bench_gain,
    bench_sum_last3,
    bench_average2,
    bench_cascade_depth,
    bench_feedback_overhead,
    bench_parallel,
    bench_adder_feedback2,
);
criterion_main!(benches);
