//! Fixed-capacity ring buffer used as the state backing of the rolling-window
//! primitives (`SumLastN`, `MovingAverageN`, `VarianceLastN`, etc.).
//!
//! Exposed as a public type because `StateMachine::State` is an associated
//! type on the primitives that use it — users don't need to touch it
//! directly, but they will see it if they inspect those state types. It's
//! a normal data structure; feel free to use it elsewhere if it's useful.

/// Fixed-capacity ring buffer. `push` overwrites the oldest entry once the
/// buffer is full, returning it so rolling-stat primitives can subtract it
/// from their running aggregate.
#[derive(Clone, Debug)]
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    write_idx: usize,
    filled: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    /// Create an empty ring buffer with capacity `n`. Slots start as
    /// `T::default()` but are treated as uninitialized until `filled()`
    /// reaches them.
    pub fn new(n: usize) -> Self {
        Self {
            buffer: vec![T::default(); n],
            write_idx: 0,
            filled: 0,
        }
    }
}

impl<T: Clone> RingBuffer<T> {
    /// Window size.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Number of real samples seen so far, capped at `capacity()`.
    pub fn filled(&self) -> usize {
        self.filled
    }

    /// Whether the buffer has received at least `capacity()` samples.
    pub fn is_full(&self) -> bool {
        self.filled == self.buffer.len()
    }

    /// Push a value. Returns the displaced (oldest) value if the buffer
    /// was already full; returns `None` while still warming up.
    pub fn push(&mut self, value: T) -> Option<T> {
        let evicted = if self.is_full() {
            Some(core::mem::replace(&mut self.buffer[self.write_idx], value))
        } else {
            self.buffer[self.write_idx] = value;
            None
        };
        self.write_idx = (self.write_idx + 1) % self.buffer.len();
        self.filled = (self.filled + 1).min(self.buffer.len());
        evicted
    }
}
