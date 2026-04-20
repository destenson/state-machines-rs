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

### `SumLast3` → `SumLastN<T>` (rolling window sum)

A fixed-window rolling sum over arbitrary numeric types. Effectively a
length-N FIR filter with unity coefficients. Useful in streaming metrics
(5-minute request totals), signal processing (integer moving averages),
and as a building block for `MovingAverage`.

Implementation: ring buffer in state, O(1) per step (subtract oldest, add
newest). Parameterize by window size; decide whether to expose a
const-generic `SumLastN<T, const N: usize>` or a runtime-sized variant
(or both).

### `Average2` → `MovingAverageN<T>` (simple moving average)

Cascade of `SumLastN` with a divide-by-N gain. Trivial once `SumLastN`
exists — mostly just a named convenience. Include both integer and
floating-point variants (integer SMA is useful when you want predictable
arithmetic).

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
