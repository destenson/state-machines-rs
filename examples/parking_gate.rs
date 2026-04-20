//! Parking gate controller (§4.1.3). Reproduces the verbose trace shown on
//! p.134 of the chapter: car arrives, gate raises, car passes through, gate
//! lowers, another car arrives.

use state_machines_rs::{
    Runner,
    primitives::{GateCommand, GatePosition, ParkingGate, ParkingGateInput},
};

fn main() {
    let mut gate = Runner::new(ParkingGate);

    let p = |pos, at_gate, just_exited| ParkingGateInput::new(pos, at_gate, just_exited);

    // Chapter's test input sequence (pp. 134–135).
    let inputs = [
        p(GatePosition::Bottom, false, false),
        p(GatePosition::Bottom, true, false),
        p(GatePosition::Bottom, true, false),
        p(GatePosition::Middle, true, false),
        p(GatePosition::Middle, true, false),
        p(GatePosition::Middle, true, false),
        p(GatePosition::Top, true, false),
        p(GatePosition::Top, true, false),
        p(GatePosition::Top, true, false),
        p(GatePosition::Top, true, true),
        p(GatePosition::Top, true, true),
        p(GatePosition::Top, true, false),
        p(GatePosition::Middle, true, false),
        p(GatePosition::Middle, true, false),
        p(GatePosition::Middle, true, false),
        p(GatePosition::Bottom, true, false),
        p(GatePosition::Bottom, true, false),
    ];

    println!("step | input                             | state       | command");
    for (t, input) in inputs.iter().enumerate() {
        let command = gate.step(*input);
        println!(
            "{:4} | pos={:?}, at_gate={}, exited={}  | {:?} -> {:?}",
            t, input.position, input.car_at_gate, input.car_just_exited,
            gate.state(), command,
        );
    }

    // Final output sequence per the chapter: nop, raise, raise, raise, raise,
    // raise, nop, nop, nop, lower, lower, lower, lower, lower, lower, nop, raise
    let _ = GateCommand::Nop; // ensure enum is in scope
}
