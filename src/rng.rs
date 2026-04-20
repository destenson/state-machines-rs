//! Minimal RNG trait and a reference implementation.
//!
//! We avoid a hard dependency on `rand`: library consumers already using
//! `rand` can implement [`Rng`] for their `rand::RngCore` type with a
//! five-line newtype (see the module docs of
//! [`MarkovChain`](crate::primitives::MarkovChain)). The trait needs only
//! `next_u64` and is otherwise minimal.
//!
//! The reference implementation [`SplitMix64`] is a well-known
//! high-quality small-state PRNG, suitable for simulations and tests. It
//! is not cryptographically secure.

/// A uniform random number generator the library can sample from.
///
/// `Clone` is required because state machines carry the RNG as part of
/// their [`State`](crate::StateMachine::State), and the state is cloned
/// when feedback combinators probe the inner machine.
pub trait Rng: Clone {
    /// Next uniform `u64`.
    fn next_u64(&mut self) -> u64;

    /// Uniform `f64` in `[0.0, 1.0)`. Default implementation uses the top
    /// 53 bits of `next_u64`.
    #[inline]
    fn next_f64(&mut self) -> f64 {
        // 53-bit mantissa, standard technique.
        (self.next_u64() >> 11) as f64 * (1.0 / ((1u64 << 53) as f64))
    }
}

/// SplitMix64 — a small-state deterministic PRNG. State is a single
/// `u64`; `Clone` and seeding are trivial.
///
/// Reference: Steele, Lea, Flood (2014) "Fast Splittable Pseudorandom
/// Number Generators". Also the warm-up RNG inside xoshiro/xoroshiro.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl Rng for SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

/// Blanket impl for any `rand::RngCore + Clone`. Enable with `--features
/// rand` to use RNGs from the [`rand`](https://docs.rs/rand) crate (e.g.
/// `SmallRng`, `ChaCha8Rng`) directly, without writing a newtype.
#[cfg(feature = "rand")]
impl<T: rand::RngCore + Clone> Rng for T {
    #[inline]
    fn next_u64(&mut self) -> u64 {
        rand::RngCore::next_u64(self)
    }
}
