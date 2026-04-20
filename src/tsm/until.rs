use crate::tsm::DynTSM;

/// Runs the inner TSM; after each completion checks the condition on the most
/// recent input. If the condition is true, terminates; otherwise restarts the
/// inner machine. §4.3.3.
pub struct RepeatUntil<I, O, F>
where
    F: Fn(&I) -> bool,
{
    inner: Box<dyn DynTSM<I, O>>,
    cond: F,
    cond_satisfied: bool,
}

impl<I, O, F> RepeatUntil<I, O, F>
where
    F: Fn(&I) -> bool,
{
    pub fn new(cond: F, inner: Box<dyn DynTSM<I, O>>) -> Self {
        Self { inner, cond, cond_satisfied: false }
    }
}

impl<I, O, F> DynTSM<I, O> for RepeatUntil<I, O, F>
where
    F: Fn(&I) -> bool,
{
    fn reset(&mut self) {
        self.inner.reset();
        self.cond_satisfied = false;
    }

    fn step(&mut self, input: &I) -> O {
        let output = self.inner.step(input);
        self.cond_satisfied = (self.cond)(input);
        if self.inner.is_done() && !self.cond_satisfied {
            self.inner.reset();
        }
        output
    }

    fn is_done(&self) -> bool {
        self.inner.is_done() && self.cond_satisfied
    }
}

/// Runs the inner TSM until either the condition becomes true on any step
/// (early exit) or the inner terminates normally. Never restarts the inner.
/// §4.3.3.
pub struct Until<I, O, F>
where
    F: Fn(&I) -> bool,
{
    inner: Box<dyn DynTSM<I, O>>,
    cond: F,
    cond_satisfied: bool,
}

impl<I, O, F> Until<I, O, F>
where
    F: Fn(&I) -> bool,
{
    pub fn new(cond: F, inner: Box<dyn DynTSM<I, O>>) -> Self {
        Self { inner, cond, cond_satisfied: false }
    }
}

impl<I, O, F> DynTSM<I, O> for Until<I, O, F>
where
    F: Fn(&I) -> bool,
{
    fn reset(&mut self) {
        self.inner.reset();
        self.cond_satisfied = false;
    }

    fn step(&mut self, input: &I) -> O {
        let output = self.inner.step(input);
        if (self.cond)(input) {
            self.cond_satisfied = true;
        }
        output
    }

    fn is_done(&self) -> bool {
        self.cond_satisfied || self.inner.is_done()
    }
}
