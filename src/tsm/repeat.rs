use crate::tsm::DynTSM;

/// Run an inner TSM to completion `n` times (or forever if `n` is `None`).
/// §4.3.1.
pub struct Repeat<I, O> {
    inner: Box<dyn DynTSM<I, O>>,
    limit: Option<usize>,
    count: usize,
}

impl<I, O> Repeat<I, O> {
    pub fn new(inner: Box<dyn DynTSM<I, O>>, limit: Option<usize>) -> Self {
        Self { inner, limit, count: 0 }
    }

    /// Finite repetition: runs `inner` to completion exactly `n` times.
    pub fn times(inner: Box<dyn DynTSM<I, O>>, n: usize) -> Self {
        Self::new(inner, Some(n))
    }

    /// Infinite repetition.
    pub fn forever(inner: Box<dyn DynTSM<I, O>>) -> Self {
        Self::new(inner, None)
    }
}

impl<I, O> DynTSM<I, O> for Repeat<I, O> {
    fn reset(&mut self) {
        self.count = 0;
        self.inner.reset();
    }

    fn step(&mut self, input: &I) -> O {
        let output = self.inner.step(input);
        // Advance past any completed iterations. Uses `while` in case a fresh
        // inner machine is already `done` on the first step after reset —
        // matches the chapter's `advanceIfDone`.
        while self.inner.is_done() && !self.is_done() {
            self.count += 1;
            if !self.is_done() {
                self.inner.reset();
            }
        }
        output
    }

    fn is_done(&self) -> bool {
        match self.limit {
            Some(n) => self.count >= n,
            None => false,
        }
    }
}
