//! TCP connection state machine (RFC 793), expressed as a `TableFsm`.
//!
//! Every entry in the transition closure mirrors one arrow in the standard
//! TCP state diagram. Inputs are the union of user events (`PassiveOpen`,
//! `ActiveOpen`, `Close`) and arriving-segment events (`Syn`, `SynAck`,
//! `Ack`, `Fin`). Outputs pair the new state with the segment, if any,
//! that the stack would transmit on that transition.
//!
//! We exercise three scenarios: active open → graceful close, passive open
//! → peer-initiated close, and simultaneous close (both sides send FIN).
//! Each finishes with the connection in `Closed`.

use state_machines_rs::{Runner, primitives::TableFsm};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    // User-initiated
    PassiveOpen,
    ActiveOpen,
    Close,
    // Incoming segment
    Syn,
    SynAck,
    Ack,
    Fin,
    // 2·MSL timer expired in TimeWait
    Timeout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Segment {
    Syn,
    SynAck,
    Ack,
    Fin,
}

fn tcp() -> TableFsm<State, Event, (State, Option<Segment>), impl Fn(&State, &Event) -> (State, (State, Option<Segment>))> {
    TableFsm::new(State::Closed, |s: &State, e: &Event| {
        // Segment names (`Syn`, `Ack`, `Fin`) collide with Event names, so
        // qualify Segment::* on the right-hand side and use Event::* in
        // patterns via explicit paths.
        use State::*;
        let (next, send) = match (*s, *e) {
            // Opening the connection.
            (Closed, Event::PassiveOpen) => (Listen, None),
            (Closed, Event::ActiveOpen) => (SynSent, Some(Segment::Syn)),
            (Listen, Event::Syn) => (SynReceived, Some(Segment::SynAck)),
            (SynSent, Event::SynAck) => (Established, Some(Segment::Ack)),
            (SynSent, Event::Syn) => (SynReceived, Some(Segment::SynAck)), // simultaneous open
            (SynReceived, Event::Ack) => (Established, None),

            // Local close.
            (Established, Event::Close) => (FinWait1, Some(Segment::Fin)),
            (FinWait1, Event::Ack) => (FinWait2, None),
            (FinWait1, Event::Fin) => (Closing, Some(Segment::Ack)), // simultaneous close
            (FinWait2, Event::Fin) => (TimeWait, Some(Segment::Ack)),
            (Closing, Event::Ack) => (TimeWait, None),
            (TimeWait, Event::Timeout) => (Closed, None),

            // Peer-initiated close.
            (Established, Event::Fin) => (CloseWait, Some(Segment::Ack)),
            (CloseWait, Event::Close) => (LastAck, Some(Segment::Fin)),
            (LastAck, Event::Ack) => (Closed, None),

            // Otherwise not modeled — stay put.
            (state, _) => (state, None),
        };
        (next, (next, send))
    })
}

fn run_scenario(name: &str, events: &[Event]) -> Vec<(State, Option<Segment>)> {
    println!("\n--- {} ---", name);
    let mut r = Runner::new(tcp());
    let mut trace = Vec::new();
    println!("initial state: {:?}", r.state());
    for &e in events {
        let out = r.step(e);
        println!("  event {:?}  -> {:?}  send={:?}", e, out.0, out.1);
        trace.push(out);
    }
    trace
}

fn main() {
    let active = run_scenario(
        "active open, graceful close",
        &[
            Event::ActiveOpen,
            Event::SynAck,
            Event::Close,
            Event::Ack,
            Event::Fin,
            Event::Timeout,
        ],
    );
    assert_eq!(active.last().unwrap().0, State::Closed);
    assert_eq!(active[0].1, Some(Segment::Syn));

    let passive = run_scenario(
        "passive open, peer-initiated close",
        &[
            Event::PassiveOpen,
            Event::Syn,
            Event::Ack,
            Event::Fin,
            Event::Close,
            Event::Ack,
        ],
    );
    assert_eq!(passive.last().unwrap().0, State::Closed);
    assert!(passive.iter().any(|(_, s)| *s == Some(Segment::SynAck)));

    let simultaneous = run_scenario(
        "simultaneous close (both sides send FIN)",
        &[
            Event::ActiveOpen,
            Event::SynAck,
            Event::Close,
            Event::Fin,
            Event::Ack,
            Event::Timeout,
        ],
    );
    assert_eq!(simultaneous.last().unwrap().0, State::Closed);
    assert!(simultaneous.iter().any(|(s, _)| *s == State::Closing));
}
