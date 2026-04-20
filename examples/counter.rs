//! Counter via feedback (§4.2.3). Feeds an incrementer's output through a
//! delay and back to its input, producing the stream `init+step, init+2*step,
//! ...`. This is the first non-trivial feedback example in the chapter.

use state_machines_rs::{
    Runner, SMExt,
    primitives::{Delay, Increment},
};

fn main() {
    // makeCounter(init=3, step=2). Delay(3) produces the initial value first,
    // so the output starts at 3 then 5, 7, 9, ... matching the chapter trace
    // on p.141.
    let counter = Increment::new(Some(2))
        .cascade(Delay::new(Some(3)))
        .feedback();

    let output: Vec<_> = Runner::new(counter).run(10);
    println!("{:?}", output.iter().flatten().collect::<Vec<_>>());
    // Expected: [3, 5, 7, 9, 11, 13, 15, 17, 19, 21]
    assert_eq!(output.into_iter().flatten().collect::<Vec<_>>(), [3, 5, 7, 9, 11, 13, 15, 17, 19, 21]);
}
