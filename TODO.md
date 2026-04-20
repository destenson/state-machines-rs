# TODO

## `lang` feature — common-pattern language acceptors

Build on top of `DfaAcceptor` with convenience constructors for the handful
of acceptor patterns people actually reach for, without pulling in a regex
crate.

Target API (all gated behind a `lang` cargo feature):

- `DfaAcceptor::contains_substring(needle)` — Knuth-Morris-Pratt-style DFA
  over any `Eq + Hash + Clone` alphabet. Outputs `true` on every step once
  `needle` has appeared anywhere in the stream.
- `DfaAcceptor::starts_with(prefix)` — accept once the stream has produced
  `prefix`, then stay accepting forever. No-match sink on deviation.
- `DfaAcceptor::any_of(patterns)` — alternation. Accept when any element
  of `patterns` has been seen as a prefix.
- `DfaAcceptor::prefix_of(pattern)` — current prefix is itself a prefix of
  `pattern`. Rejects as a sink on deviation (the generalized chapter ABC).
- `DfaAcceptor::cyclic_prefix(pattern)` — prefix of `pattern^ω`. Exact
  generalization of the chapter's toy to arbitrary alphabets and lengths.

`CharClass` helpers for `char` alphabets:
- `digit`, `alpha`, `alphanumeric`, `whitespace`, `xdigit`, `custom(&[char])`,
  complement via `not`. Compose into acceptor patterns.

Optional: a tiny regex-lite recognizer for `foo|bar|a*b`-shaped patterns,
implemented as NFA → DFA compile at construction time. Skip if the
substring / prefix / alternation set already covers the real use cases.

Explicit non-goals: Unicode character classes (`\w` semantics across
locales), capture groups, lookaround. Users who need those should reach
for a real regex crate.

## Generalize other chapter toys out of the `toy` feature

Currently gated: `ParkingGate`, `SumLast3`, `Average2`, `ABC`. Each one is
a specific instance of something more useful. Before un-gating, replace
with a generic primitive:

### `ParkingGate` → generic `TableFsm<S, I, O>` — DONE

Shipped as `primitives::TableFsm`. Users supply a `Fn(&S, &I) -> (S, O)`
at construction time; the closure is a generating function for the
transition table, which subsumes any matrix-style encoding. See the
traffic-light / vending-machine tests in `tests/table_fsm.rs` and the
doctest on `TableFsm`.

For non-deterministic (probabilistic) FSMs, see `MarkovChain` —
separately shipped since stochastic transitions are mathematically a
different object.

### `SumLast3` → `SumLastN<T>` — DONE

Shipped as `primitives::SumLastN<T>`. Runtime-sized window, O(1) updates
via a running sum maintained in state (subtract oldest, add newest once
the buffer fills). Generic over any `SafeAdd + SafeSub + Clone + Default`
— works for integers, floats, and `Option<T>` in feedback pipelines.
`new_with(n) -> Result<Self, SumLastNError>` rejects zero-width windows.

### `WindowedFold<T, S, Add, Remove, Finish>` — deferred

Hypothetical generalization of all `*LastN` primitives: a ring-buffer-
backed state machine parameterized by user closures `add: Fn(&S, &T) ->
S`, `remove: Fn(&S, &T) -> S`, and `finish: Fn(&S, usize) -> O`. Every
existing rolling-window primitive is an instance:

- SumLastN    : S=T,         add=+,  remove=−, finish=s.clone()
- MovingAvgN  : S=sum,       add=+,  remove=−, finish=s/n
- VarianceN   : S=(sum,sum²), add on both, remove on both, finish via (s2-s1²/n)/n
- StdDevN     : same, finish=sqrt(variance)

Deferred because: (a) the generic signature is closure-heavy and
clunkier than the concrete form for the common cases; (b) numerical
concerns (Welford-style variance, int vs float semantics, sample vs
population divisors) don't cleanly fold into a single `finish`; (c) we
only have four concrete consumers, so the generic would add indirection
rather than remove it. Revisit if/when someone wants rolling min/max,
median, skewness, kurtosis, or a monoidal metric we don't already ship —
at ~3 more consumers the generalization starts to pay.

DRY is already taken care of at the implementation level: all the
rolling-window primitives share `primitives::RingBuffer<T>` for their
ring-buffer state. Only the aggregate update + output formula differs.

### `Average2` → `MovingAverageN` — DONE

Shipped as `primitives::MovingAverageN` (f64-specialized). Maintains a
running sum exactly and divides once per step on output; during warm-up
it emits the partial average over samples seen so far rather than
pretending about zero-padding. `new_with(n) -> Result<Self,
MovingAverageNError>` rejects zero-width windows. For integer or other
typed SMAs, users can cascade `SumLastN<T>` with `Gain(1/n)` manually.

### `CharTSM`, `ConsumeFiveValues` → just write a small impl

Both are chapter-pedagogical TSMs gated behind `toy`. The "emit a fixed
value once, terminate" and "read N inputs, emit a fold, terminate"
patterns are tiny — typically 10 lines including `done`. No general
primitive seems to beat writing the impl directly. `examples/
hello_world_tsm.rs` ships an inline `OneCharTSM` demonstrating the
pattern for the first case.

If a compelling general form emerges (e.g. `OnceTSM<T>` used in enough
places to deduplicate), it can be added later without churning the
feature gate.

### `UpDown` → already generalized by `Accumulator<i64>`

`UpDown` is just `Accumulator<i64>` with a trivial `{Up → +1, Down → -1}`
input mapping. Gated as a toy; no new primitive needed — users who want
it can write a one-line enum mapping and feed the deltas straight into
`Accumulator`. Consider adding a short example showing this migration so
the pattern is discoverable.

### General `ABC` → already done (`DfaAcceptor` exists)

Keep `ABC` gated as the pedagogical reference; steer everyone toward
`DfaAcceptor` for real use. The `lang` feature above will make the
migration story even clearer.
