//! Enemy AI behavior tree as a `TableFsm`.
//!
//! The ubiquitous "idle / patrol / chase / attack / flee" pattern seen in
//! almost every action game. The transition is driven by perception data
//! (distance to the player, own health); outputs are motor commands the
//! engine's animation/locomotion layer would consume.
//!
//! We script a scenario in which the player approaches, gets into attack
//! range, the enemy trades hits until its health dips below a threshold,
//! then it flees until it feels safe again.

use state_machines_rs::{Runner, primitives::TableFsm};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Behavior {
    Idle,
    Chase,
    Attack,
    Flee,
}

/// Per-tick perception snapshot.
#[derive(Clone, Copy, Debug)]
pub struct Sense {
    pub player_distance: f32,
    pub health: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Hold position, scan.
    Wait,
    /// Run toward the player.
    MoveToward,
    /// Swing / shoot / whatever does damage.
    Strike,
    /// Break line of sight, run to safety.
    RunAway,
}

const SIGHT_RANGE: f32 = 15.0;
const ATTACK_RANGE: f32 = 2.5;
const LOW_HEALTH: u8 = 25;
const SAFE_HEALTH: u8 = 70;
const SAFE_DISTANCE: f32 = 25.0;

fn enemy() -> TableFsm<Behavior, Sense, (Behavior, Action), impl Fn(&Behavior, &Sense) -> (Behavior, (Behavior, Action))> {
    TableFsm::new(Behavior::Idle, |b: &Behavior, sense: &Sense| {
        // Low health always pre-empts — retreat takes priority.
        let next = if sense.health < LOW_HEALTH && !matches!(b, Behavior::Flee) {
            Behavior::Flee
        } else {
            match (b, sense.player_distance) {
                (Behavior::Flee, d) if sense.health >= SAFE_HEALTH && d >= SAFE_DISTANCE => {
                    Behavior::Idle
                }
                (Behavior::Flee, _) => Behavior::Flee,
                (_, d) if d <= ATTACK_RANGE => Behavior::Attack,
                (_, d) if d <= SIGHT_RANGE => Behavior::Chase,
                _ => Behavior::Idle,
            }
        };
        let action = match next {
            Behavior::Idle => Action::Wait,
            Behavior::Chase => Action::MoveToward,
            Behavior::Attack => Action::Strike,
            Behavior::Flee => Action::RunAway,
        };
        (next, (next, action))
    })
}

fn main() {
    let mut r = Runner::new(enemy());

    // Script: player approaches from far, closes to attack range; enemy
    // takes a few hits; health drops; enemy flees; recovers in the wild.
    let script = [
        Sense { player_distance: 30.0, health: 100 }, // idle
        Sense { player_distance: 20.0, health: 100 }, // still idle (> sight)
        Sense { player_distance: 12.0, health: 100 }, // chase
        Sense { player_distance: 8.0, health: 100 },  // chase
        Sense { player_distance: 2.0, health: 100 },  // attack
        Sense { player_distance: 2.0, health: 80 },   // attack, got hit
        Sense { player_distance: 2.0, health: 60 },   // attack, more damage
        Sense { player_distance: 2.0, health: 30 },   // attack, barely alive
        Sense { player_distance: 2.0, health: 20 },   // flee now!
        Sense { player_distance: 8.0, health: 25 },   // flee (regen slow)
        Sense { player_distance: 18.0, health: 40 },  // flee
        Sense { player_distance: 30.0, health: 70 },  // flee (safe health, but not yet safe distance… flip conditions)
        Sense { player_distance: 40.0, health: 80 },  // idle (recovered + far)
    ];

    println!("tick | dist  | hp  | behavior   | action");
    let mut saw_chase = false;
    let mut saw_attack = false;
    let mut saw_flee = false;
    let mut saw_recovery = false;

    for (t, sense) in script.iter().enumerate() {
        let (behavior, action) = r.step(*sense);
        println!(
            "{:4} | {:>5.1} | {:>3} | {:?} | {:?}",
            t, sense.player_distance, sense.health, behavior, action
        );
        match behavior {
            Behavior::Chase => saw_chase = true,
            Behavior::Attack => saw_attack = true,
            Behavior::Flee => saw_flee = true,
            Behavior::Idle if saw_flee => saw_recovery = true,
            _ => {}
        }
    }

    assert!(saw_chase, "enemy should have engaged chase");
    assert!(saw_attack, "enemy should have attacked at close range");
    assert!(saw_flee, "enemy should have fled on low HP");
    assert!(saw_recovery, "enemy should have returned to idle after recovering");
}
