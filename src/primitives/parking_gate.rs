//! Parking-gate controller from §4.1.3 — the canonical finite-state controller
//! example. Sensors report arm position, whether a car is waiting, and whether
//! a car just exited; the gate motor is driven with `Raise`, `Lower`, or `Nop`.

use crate::core::StateMachine;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GatePosition {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateState {
    Waiting,
    Raising,
    Raised,
    Lowering,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateCommand {
    Raise,
    Lower,
    Nop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParkingGateInput {
    pub position: GatePosition,
    pub car_at_gate: bool,
    pub car_just_exited: bool,
}

impl ParkingGateInput {
    pub fn new(position: GatePosition, car_at_gate: bool, car_just_exited: bool) -> Self {
        Self { position, car_at_gate, car_just_exited }
    }
}

pub struct ParkingGate;

impl ParkingGate {
    fn command_for(state: GateState) -> GateCommand {
        match state {
            GateState::Raising => GateCommand::Raise,
            GateState::Lowering => GateCommand::Lower,
            GateState::Waiting | GateState::Raised => GateCommand::Nop,
        }
    }
}

impl StateMachine for ParkingGate {
    type Input = ParkingGateInput;
    type Output = GateCommand;
    type State = GateState;

    fn start_state(&self) -> GateState {
        GateState::Waiting
    }

    fn next_values(
        &self,
        state: &GateState,
        input: &ParkingGateInput,
    ) -> (GateState, GateCommand) {
        let next = match (state, input) {
            (GateState::Waiting, i) if i.car_at_gate => GateState::Raising,
            (GateState::Raising, i) if i.position == GatePosition::Top => GateState::Raised,
            (GateState::Raised, i) if i.car_just_exited => GateState::Lowering,
            (GateState::Lowering, i) if i.position == GatePosition::Bottom => GateState::Waiting,
            (s, _) => *s,
        };
        (next, Self::command_for(next))
    }
}
