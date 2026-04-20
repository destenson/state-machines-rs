# state-machines-rs

Compositional discrete-time state machines in Rust, after MIT 6.01 chapter 4
([source PDF](docs/6d24bc51571a1a945a63ffa8343a5b55_MIT6_01SCS11_chap04.pdf)).

Every machine implements one small trait:

```rust
pub trait StateMachine {
    type Input;
    type Output;
    type State: Clone;
    fn start_state(&self) -> Self::State;
    fn next_values(&self, state: &Self::State, input: &Self::Input)
        -> (Self::State, Self::Output);
    fn done(&self, _: &Self::State) -> bool { false }
}
```

`next_values` is pure (`&self`, `&State`, returns the new state and output
by value). Mutation lives in `Runner<M>`. That split is what makes the
feedback combinator well-defined — it probes a machine twice per step with
the same state and different inputs.

## Quick start

```rust
use state_machines_rs::{Runner, SMExt, primitives::{Delay, Increment}};

// A counter: output[t+1] = output[t] + 1, starting at 3.
// Python: sm.Feedback(sm.Cascade(Increment(1), sm.Delay(3)))
let counter = Increment::new(Some(1i64))
    .cascade(Delay::new(Some(3i64)))
    .feedback();

let out: Vec<i64> = Runner::new(counter).run(10)
    .into_iter().flatten().collect();
assert_eq!(out, [3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
```

The `Option<T>` wrapping around the integer comes from the `Defined` trait:
feedback combinators need a sentinel to probe the inner machine, and
`None` works naturally as that sentinel. Non-feedback code can use bare
`T` directly.

## What's in the box

**Primitives** (`primitives::`):
`Accumulator`, `Gain`, `Delay` (aliased `R`), `Increment`, `Wire`,
`Negation`, `Adder`, `Multiplier`, `Select`, `SumLastN`,
`MovingAverageN`, `DfaAcceptor`, `TableFsm`, `MarkovChain`.

Chapter-specific pedagogical machines (`ABC`, `ParkingGate`, `SumLast3`,
`Average2`, `UpDown`, `CharTSM`, `ConsumeFiveValues`) are gated behind
the `toy` cargo feature — enable it with `--features toy` to build the
chapter trace tests and the original examples. Each toy now has a
generalized counterpart on the default surface: `TableFsm` replaces
`ParkingGate`, `SumLastN` replaces `SumLast3`, `MovingAverageN`
replaces `Average2`, and `Accumulator<i64>` with a delta mapping
covers `UpDown`. See [`TODO.md`](TODO.md) for notes on `CharTSM` /
`ConsumeFiveValues` (shape is small enough to inline).

**Randomness** (`rng`):
`Rng` trait with `next_u64` / `next_f64`, plus the reference
[`SplitMix64`] PRNG. `MarkovChain<S, R>` carries any `R: Rng` in its
state. With `--features rand`, you get a blanket impl for
`rand::RngCore + Clone`, so `rand::rngs::SmallRng` and friends drop in
without a newtype.

**Combinators** (`combinators::`):
- Dataflow: `Cascade`, `Parallel`, `Parallel2`, `ParallelAdd`
- Feedback: `Feedback`, `Feedback2`, `FeedbackAdd`, `FeedbackSubtract`
- Conditional: `Switch`, `Mux`, `If`

**Terminating state machines** (`tsm::`):
`Repeat`, `Sequence`, `Until`, `RepeatUntil` over `Box<dyn DynTSM<I, O>>`,
plus the `Stateful` adapter that erases a static `StateMachine`'s state
type so you can drop it into a heterogeneous sequence. Writing a TSM
primitive is usually a ~10-line `impl StateMachine` with a custom `done`
predicate; see `examples/hello_world_tsm.rs` for the canonical shape.

**Fluent builder** (`SMExt`):
`.cascade()`, `.parallel()`, `.parallel_add()`, `.feedback()`,
`.feedback2()`, `.feedback_add()`, `.feedback_subtract()`, `.switch()`,
`.mux()`, `.if_else()`.

## Examples

Run any with `cargo run --example <name>`. Each one finishes with internal
assertions, so a successful run means the output matched expectations.

### From the chapter

| Example | What it shows |
| --- | --- |
| `parking_gate` | FSM controller with enum states and inputs (§4.1.3) — requires `--features toy` |
| `language_acceptor` | FSM accepting prefixes of `a,b,c,a,b,c,…` (§4.1.1.1) — requires `--features toy` |
| `counter` | Feedback counter via `Feedback(Cascade(Increment, Delay))` (§4.2.3) |
| `fibonacci` | Feedback + Parallel + Adder: outputs 1, 2, 3, 5, 8, 13, … (§4.2.3.2) |
| `factorial` | Feedback2 + Multiplier + Delay driven by a counter (§4.2.3.5) |
| `wall_follower` | Plant/controller coupling via cascade-plus-feedback (§4.2.4) |
| `hello_world_tsm` | `Sequence` of `CharTSM`s, plus `Repeat` (§4.3.1–4.3.2) |
| `accumulator_variants` | Same running sum via two topologies; shows the unit-of-delay difference |

### Signal processing / DSP

| Example | What it shows |
| --- | --- |
| `biquad_filter` | Transposed-DF-II IIR low-pass with RBJ coefficients; filters a noisy sine |
| `one_pole_filters` | EMA (low-pass) and DC blocker (high-pass) side by side |

### Control

| Example | What it shows |
| --- | --- |
| `pid_controller` | P+I+D composed as `ParallelAdd(Gain, Accumulator·Gain, …)` wrapped in `Feedback` around a first-order plant |
| `kalman_filter` | Scalar Kalman filter (953× MSE reduction on constant truth) |

### Low-latency decisions and reliability

| Example | What it shows |
| --- | --- |
| `schmitt_trigger` | Hysteresis: 10 clean edges vs 257 naive-comparator chattery edges |
| `debouncer` | N-consecutive-sample confirmation for digital inputs |
| `circuit_breaker` | Closed/Open/HalfOpen reliability FSM with failure counting and cooldown |
| `token_bucket` | API rate limiter with fractional refill and scripted workload |

### Trading / time-series

| Example | What it shows |
| --- | --- |
| `ma_crossover` | Two EMAs + crossover detector = momentum buy/sell signal |
| `bollinger_breakout` | Rolling mean ± Kσ with ring-buffer state; fires on volatility shocks |

### Parsing

| Example | What it shows |
| --- | --- |
| `json_lexer` | Char-stream to `Vec<Token>` lexer with multi-state accumulation and flush-on-delimiter |
| `tcp_state_machine` | RFC 793 TCP connection FSM via `TableFsm`; active/passive/simultaneous close scenarios |

### Games / sensation-driven FSMs

| Example | What it shows |
| --- | --- |
| `game_ai` | Enemy behavior (idle/chase/attack/flee) via `TableFsm`; transitions on distance + health |

### Stochastic / simulation

| Example | What it shows |
| --- | --- |
| `weather_markov` | `MarkovChain` sampling a 3-state weather model; empirical distribution converges to the stationary distribution |
| `sir_epidemic` | 1000 independent `MarkovChain` agents modeling S/I/R disease progression; infection curve peaks and decays |
| `text_markov` | Character-level bigram model built from a small corpus; samples "English-adjacent" text |

## Performance

Static composition (everything except the `DynTSM` TSMs) is genuinely
zero-cost. Numbers from `cargo bench` on a modern x86 box with thin LTO:

| Machine | ns/step |
| --- | --- |
| Gain (f64 multiply) | 0.28 |
| Delay / Average2 | 0.36 |
| Accumulator | 0.53 |
| Cascade depth 5 (five Delays) | 0.53 |
| Feedback counter (vs open-loop cascade) | indistinguishable |
| Parallel (two Accumulators) | 0.32 |
| Fibonacci i128 | 1.04 |
| Wall follower (Option<f64>) | 3.16 |
| TSM Repeat / Sequence (dyn-dispatch) | 5–20 |

Composition tax for monomorphized pipelines is in the noise: a five-deep
cascade costs the same as a single primitive because LLVM inlines the
whole chain. The `Box<dyn DynTSM>` path is 10–50× slower than inlined
primitives but still >50M steps/sec.

Reproduce with `cargo bench --bench throughput` and
`cargo bench --bench compositions`.

## Tests

```
cargo test
```

`tests/chapter_traces.rs` pins every primitive and combinator to the exact
I/O traces published in the chapter, with page citations.

## License

This library is [MIT-licensed](LICENSE). The MIT 6.01 chapter PDF under
`docs/` is OpenCourseWare material and retains its original CC BY-NC-SA
4.0 license — see `LICENSE` for details. The Rust code here is a new
implementation inspired by the chapter's formalism.
