//! TSM sequencing (§4.3.2). Emits "Hello World" one character at a time using
//! a `Sequence` of `CharTSM`s, then demonstrates `Repeat` of a three-character
//! sub-sequence.

use state_machines_rs::tsm::{CharTSM, DynTSM, Repeat, Sequence, into_dyn};

fn emit_all<M: DynTSM<(), char> + ?Sized>(m: &mut M) -> Vec<char> {
    let mut out = Vec::new();
    while !m.is_done() {
        out.push(m.step(&()));
    }
    out
}

fn make_text_sequence(s: &str) -> Sequence<(), char> {
    let machines: Vec<Box<dyn DynTSM<(), char>>> =
        s.chars().map(|c| into_dyn(CharTSM::new(c))).collect();
    Sequence::new(machines)
}

fn main() {
    let mut hello = make_text_sequence("Hello World");
    let out = emit_all(&mut hello);
    println!("{}", out.iter().collect::<String>());

    // Repeat "abc" three times -> "abcabcabc"
    let abc = Box::new(make_text_sequence("abc")) as Box<dyn DynTSM<(), char>>;
    let mut repeated = Repeat::times(abc, 3);
    let out = emit_all(&mut repeated);
    println!("{}", out.iter().collect::<String>());
}
