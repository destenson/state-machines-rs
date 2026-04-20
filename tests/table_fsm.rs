//! Integration tests for `TableFsm`.

use state_machines_rs::{Runner, StateMachine, primitives::TableFsm};

#[test]
fn traffic_light_cycles() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Light {
        Red,
        Green,
        Yellow,
    }

    let light = TableFsm::new(Light::Red, |s: &Light, _: &()| {
        let next = match s {
            Light::Red => Light::Green,
            Light::Green => Light::Yellow,
            Light::Yellow => Light::Red,
        };
        (next, *s)
    });

    let out = Runner::new(light).run(7);
    assert_eq!(
        out,
        [
            Light::Red,
            Light::Green,
            Light::Yellow,
            Light::Red,
            Light::Green,
            Light::Yellow,
            Light::Red,
        ]
    );
}

#[test]
fn vending_machine_with_structured_output() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Input {
        Coin(u32),
        Refund,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Output {
        Accept,
        Vend { change: u32 },
        Refund { amount: u32 },
    }

    const PRICE: u32 = 25;
    let vm = TableFsm::new(0u32, |balance: &u32, inp: &Input| match inp {
        Input::Coin(c) => {
            let total = balance + c;
            if total >= PRICE {
                (0, Output::Vend { change: total - PRICE })
            } else {
                (total, Output::Accept)
            }
        }
        Input::Refund => (0, Output::Refund { amount: *balance }),
    });

    let trace = Runner::new(vm).transduce([
        Input::Coin(10),
        Input::Coin(10),
        Input::Coin(5),
        Input::Coin(25),
        Input::Coin(10),
        Input::Refund,
    ]);

    assert_eq!(
        trace,
        [
            Output::Accept,                    // 10
            Output::Accept,                    // 20
            Output::Vend { change: 0 },        // 25, vend, reset
            Output::Vend { change: 0 },        // 25 exact
            Output::Accept,                    // 10
            Output::Refund { amount: 10 },     // refund 10
        ]
    );
}

#[test]
fn state_type_can_be_complex() {
    // State is a tuple; input and output are distinct types. Sanity check
    // that the generic parameters don't fight each other.
    let m = TableFsm::new(
        (0i64, false),
        |&(count, flipped): &(i64, bool), x: &i64| {
            let next = (count + x, !flipped);
            (next, (count, flipped))
        },
    );
    let out = Runner::new(m).transduce([10, 20, 30]);
    assert_eq!(out, [(0, false), (10, true), (30, false)]);
}
