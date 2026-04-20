//! The [`StateMachine`] trait and its [`Runner`].
//!
//! `StateMachine` is pure by construction: `next_values` takes `&self` and
//! `&State`, returning the new state and output by value. This is the property
//! the chapter relies on when defining `Feedback` — the combinator probes a
//! machine twice per step with the same state and different inputs.
//!
//! State mutation lives in [`Runner`], which owns the current state and exposes
//! `start` / `step` / `transduce` / `run`.

/// A discrete-time state machine: a pure `(state, input) -> (state, output)`
/// transition plus an initial state.
///
/// Implementors should make `next_values` genuinely pure — no interior
/// mutation, no RNG, no I/O. Feedback combinators will call it multiple
/// times per step with the same state.
pub trait StateMachine {
    type Input;
    type Output;
    type State: Clone;

    fn start_state(&self) -> Self::State;

    fn next_values(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> (Self::State, Self::Output);

    /// Whether this machine has terminated. Non-terminating machines return
    /// `false` forever (the default); TSMs override this.
    fn done(&self, _state: &Self::State) -> bool {
        false
    }
}

/// Owns a machine and its current state, providing mutating `step`/`transduce`
/// methods on top of the pure [`StateMachine`] transition.
pub struct Runner<M: StateMachine> {
    machine: M,
    state: M::State,
}

impl<M: StateMachine> Runner<M> {
    pub fn new(machine: M) -> Self {
        let state = machine.start_state();
        Self { machine, state }
    }

    /// Restart: reset `state` to the machine's `start_state`.
    pub fn start(&mut self) {
        self.state = self.machine.start_state();
    }

    pub fn step(&mut self, input: M::Input) -> M::Output {
        let (new_state, output) = self.machine.next_values(&self.state, &input);
        self.state = new_state;
        output
    }

    /// Run the machine over a sequence of inputs. Stops early if the machine
    /// reports `done` after any step — mirroring the chapter's TSM-aware
    /// `transduce`.
    pub fn transduce<I>(&mut self, inputs: I) -> Vec<M::Output>
    where
        I: IntoIterator<Item = M::Input>,
    {
        let mut out = Vec::new();
        for input in inputs {
            out.push(self.step(input));
            if self.machine.done(&self.state) {
                break;
            }
        }
        out
    }

    pub fn state(&self) -> &M::State {
        &self.state
    }

    pub fn machine(&self) -> &M {
        &self.machine
    }

    pub fn is_done(&self) -> bool {
        self.machine.done(&self.state)
    }
}

impl<M> Runner<M>
where
    M: StateMachine<Input = ()>,
{
    /// Drive an input-less machine for up to `n` steps, stopping early if it
    /// terminates.
    pub fn run(&mut self, n: usize) -> Vec<M::Output> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.step(()));
            if self.machine.done(&self.state) {
                break;
            }
        }
        out
    }
}
