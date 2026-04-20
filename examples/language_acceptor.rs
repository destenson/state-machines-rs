//! Language acceptor (§4.1.1.1). Accepts prefixes of the infinite pattern
//! `a, b, c, a, b, c, ...`; once the stream deviates, the machine enters a
//! sink state and rejects forever.

use state_machines_rs::{
    Runner,
    primitives::{ABC, AbcOutput},
};

fn main() {
    let mut acceptor = Runner::new(ABC);
    let chars: Vec<char> = "abcacab".chars().collect();
    let outs = acceptor.transduce(chars.iter().copied());

    for (c, o) in chars.iter().zip(outs.iter()) {
        let verdict = if matches!(o, AbcOutput::Accept) { "accept" } else { "REJECT" };
        println!("{} -> {}", c, verdict);
    }
    // Expected: accept, accept, accept, accept, REJECT, REJECT, REJECT
}
