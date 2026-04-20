//! TSM sequencing. Emits "Hello World" one character at a time using a
//! `Sequence` of trivial single-character TSMs, then demonstrates `Repeat`
//! of a three-character sub-sequence.
//!
//! The inline `OneCharTSM` demonstrates the pattern for any
//! "emit-one-value-then-terminate" primitive — it's a 10-line
//! `impl StateMachine` with `done` flipping true after the first step.

use state_machines_rs::{
    StateMachine,
    tsm::{DynTSM, Repeat, Sequence, into_dyn},
};

struct OneCharTSM(char);

impl StateMachine for OneCharTSM {
    type Input = ();
    type Output = char;
    type State = bool;
    fn start_state(&self) -> bool { false }
    fn next_values(&self, _: &bool, _: &()) -> (bool, char) { (true, self.0) }
    fn done(&self, s: &bool) -> bool { *s }
}

fn emit_all<M: DynTSM<(), char> + ?Sized>(m: &mut M) -> Vec<char> {
    let mut out = Vec::new();
    while !m.is_done() {
        out.push(m.step(&()));
    }
    out
}

fn make_text_sequence(s: &str) -> Sequence<(), char> {
    let machines: Vec<Box<dyn DynTSM<(), char>>> =
        s.chars().map(|c| into_dyn(OneCharTSM(c))).collect();
    Sequence::new(machines)
}

fn main() {
    let mut hello = make_text_sequence("Hello World");
    let out = emit_all(&mut hello);
    println!("{}", out.iter().collect::<String>());
    assert_eq!(out.iter().collect::<String>(), "Hello World");

    let abc = Box::new(make_text_sequence("abc")) as Box<dyn DynTSM<(), char>>;
    let mut repeated = Repeat::times(abc, 3);
    let out = emit_all(&mut repeated);
    println!("{}", out.iter().collect::<String>());
    assert_eq!(out.iter().collect::<String>(), "abcabcabc");
}
