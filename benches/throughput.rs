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
    primitives::{
        Accumulator, Adder, Delay, DfaAcceptor, Gain, Increment, MovingAverageN, StdDevLastN,
        SumLastN, TableFsm, VarianceLastN, Wire,
    },
};

#[cfg(feature = "toy")]
use state_machines_rs::primitives::{Average2, SumLast3};

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

#[cfg(feature = "toy")]
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

#[cfg(feature = "toy")]
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

/// Rolling-window primitives at a few window sizes. The ring buffer is
/// shared across all four, so differences reflect aggregate-update cost:
/// SumLastN is one add+sub, Moving adds a divide, Variance adds a
/// square+sub+square, StdDev adds a sqrt on top of Variance.
fn bench_rolling_window(c: &mut Criterion) {
    const N: usize = 100_000;
    let windows: &[usize] = &[4, 20, 100];
    let mut group = c.benchmark_group("rolling_window");
    group.throughput(Throughput::Elements(N as u64));

    for &w in windows {
        group.bench_with_input(BenchmarkId::new("sum_last_n", w), &w, |b, &w| {
            b.iter_batched(
                || {
                    (
                        Runner::new(SumLastN::<f64>::new_with(w).unwrap()),
                        (0..N).map(|i| i as f64 * 0.5).collect::<Vec<_>>(),
                    )
                },
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("moving_average_n", w), &w, |b, &w| {
            b.iter_batched(
                || {
                    (
                        Runner::new(MovingAverageN::new_with(w).unwrap()),
                        (0..N).map(|i| i as f64 * 0.5).collect::<Vec<_>>(),
                    )
                },
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("variance_last_n_pop", w), &w, |b, &w| {
            b.iter_batched(
                || {
                    (
                        Runner::new(VarianceLastN::new_population_with(w).unwrap()),
                        (0..N).map(|i| i as f64 * 0.5).collect::<Vec<_>>(),
                    )
                },
                |(mut r, input)| {
                    for x in input {
                        black_box(r.step(x));
                    }
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("std_dev_last_n_pop", w), &w, |b, &w| {
            b.iter_batched(
                || {
                    (
                        Runner::new(StdDevLastN::new_population_with(w).unwrap()),
                        (0..N).map(|i| i as f64 * 0.5).collect::<Vec<_>>(),
                    )
                },
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

/// Deterministic FSMs configured at runtime: TableFsm (arbitrary output)
/// and DfaAcceptor (bool output via an acceptance predicate).
fn bench_declarative_fsm(c: &mut Criterion) {
    const N: usize = 1_000_000;
    let mut group = c.benchmark_group("declarative_fsm");
    group.throughput(Throughput::Elements(N as u64));

    // Traffic-light cycle; state is a u8 index, output is the state.
    group.bench_function("table_fsm_3_state_cycle", |b| {
        b.iter_batched(
            || {
                let m = TableFsm::new(0u8, |s: &u8, _: &()| {
                    let next = (s + 1) % 3;
                    (next, *s)
                });
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

    // Substring-match DFA for pattern "ab"; state 0 = none, 1 = saw 'a',
    // 2 = matched (sink). Input is a repeating stream mostly of 'x'.
    group.bench_function("dfa_contains_substring", |b| {
        b.iter_batched(
            || {
                let dfa = DfaAcceptor::new(
                    0u8,
                    |s: &u8, c: &char| match (*s, *c) {
                        (2, _) => 2,
                        (_, 'a') => 1,
                        (1, 'b') => 2,
                        _ => 0,
                    },
                    |s: &u8| *s == 2,
                );
                let input: Vec<char> = (0..N)
                    .map(|i| match i % 13 { 5 => 'a', 6 => 'b', _ => 'x' })
                    .collect();
                (Runner::new(dfa), input)
            },
            |(mut r, input)| {
                for c in input {
                    black_box(r.step(c));
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

#[cfg(not(feature = "toy"))]
criterion_group!(
    benches,
    bench_accumulator,
    bench_delay,
    bench_gain,
    bench_cascade_depth,
    bench_feedback_overhead,
    bench_parallel,
    bench_adder_feedback2,
    bench_rolling_window,
    bench_declarative_fsm,
);

#[cfg(feature = "toy")]
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
    bench_rolling_window,
    bench_declarative_fsm,
);

criterion_main!(benches);
