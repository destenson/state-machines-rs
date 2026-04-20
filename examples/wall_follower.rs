//! Plant/controller coupling (§4.2.4). A robot drives toward a wall and stops
//! at a desired distance using proportional control. Output converges to 1.0.
//!
//! Topology: `Feedback(Cascade(controller, plant))`.

use state_machines_rs::{Runner, SMExt, StateMachine};

/// Proportional controller: velocity = k * (d_desired - distance).
struct WallController {
    k: f64,
    d_desired: f64,
}

impl StateMachine for WallController {
    type Input = Option<f64>;
    type Output = Option<f64>;
    type State = Option<f64>;

    fn start_state(&self) -> Option<f64> {
        None
    }
    fn next_values(
        &self,
        _: &Option<f64>,
        input: &Option<f64>,
    ) -> (Option<f64>, Option<f64>) {
        let v = input.map(|d| self.k * (self.d_desired - d));
        (v, v)
    }
}

/// World: next distance = current - delta_t * velocity. Has one-step delay
/// built in — new distance depends on previous velocity, not current.
struct WallWorld {
    delta_t: f64,
    initial_distance: f64,
}

impl StateMachine for WallWorld {
    /// Input: velocity command.
    type Input = Option<f64>;
    /// Output: current distance to wall.
    type Output = Option<f64>;
    type State = Option<f64>;

    fn start_state(&self) -> Option<f64> {
        Some(self.initial_distance)
    }
    fn next_values(
        &self,
        state: &Option<f64>,
        input: &Option<f64>,
    ) -> (Option<f64>, Option<f64>) {
        let new_distance = match (state, input) {
            (Some(d), Some(v)) => Some(d - self.delta_t * v),
            (Some(d), None) => Some(*d),
            _ => None,
        };
        (new_distance, *state)
    }
}

fn main() {
    let controller = WallController { k: -1.5, d_desired: 1.0 };
    let world = WallWorld { delta_t: 0.1, initial_distance: 5.0 };

    let system = controller.cascade(world).feedback();

    let out: Vec<_> = Runner::new(system).run(30);
    for (t, d) in out.iter().enumerate() {
        if let Some(dist) = d {
            println!("t={:2}  distance = {:.6}", t, dist);
        }
    }
    // Distance converges monotonically to 1.0.
}
