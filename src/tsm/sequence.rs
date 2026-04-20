use crate::tsm::DynTSM;

/// Runs a list of TSMs in order: once the first finishes, control passes to
/// the second, and so on. Done when the last machine terminates. §4.3.2.
pub struct Sequence<I, O> {
    machines: Vec<Box<dyn DynTSM<I, O>>>,
    index: usize,
}

impl<I, O> Sequence<I, O> {
    pub fn new(machines: Vec<Box<dyn DynTSM<I, O>>>) -> Self {
        assert!(!machines.is_empty(), "Sequence requires at least one machine");
        Self { machines, index: 0 }
    }
}

impl<I, O> DynTSM<I, O> for Sequence<I, O> {
    fn reset(&mut self) {
        for m in &mut self.machines {
            m.reset();
        }
        self.index = 0;
    }

    fn step(&mut self, input: &I) -> O {
        let output = self.machines[self.index].step(input);
        // Advance to the next non-done machine. Matches the chapter's
        // `advanceIfDone` on Sequence.
        while self.machines[self.index].is_done() && self.index + 1 < self.machines.len() {
            self.index += 1;
        }
        output
    }

    fn is_done(&self) -> bool {
        self.index + 1 == self.machines.len() && self.machines[self.index].is_done()
    }
}
