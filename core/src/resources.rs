//! Resource management.
use core::ops::{Index, IndexMut};

use linux_raw_sys::general::{RLIM_NLIMITS, RLIMIT_STACK};

/// Resource limit structure representing soft and hard limits.
///
/// Each resource limit has two components:
/// - `current`: The soft limit, which is the current value the process may consume.
///   If a process reaches its soft limit, it may receive a signal but can continue execution.
/// - `max`: The hard limit, which is the ceiling for the soft limit.
///   A process may only raise its soft limit up to the hard limit, and only privileged
///   processes may raise the hard limit.
#[derive(Default)]
pub struct Rlimit {
    /// The current (soft) limit
    pub current: u64,
    /// The maximum (hard) limit
    pub max: u64,
}

impl Rlimit {
    /// Creates a new resource limit with specified soft and hard limits.
    ///
    /// # Arguments
    ///
    /// * `soft` - The soft limit value
    /// * `hard` - The hard limit value
    pub fn new(soft: u64, hard: u64) -> Self {
        Self {
            current: soft,
            max: hard,
        }
    }
}

impl From<u64> for Rlimit {
    /// Creates a resource limit where both soft and hard limits are set to the same value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to set for both limits
    fn from(value: u64) -> Self {
        Self {
            current: value,
            max: value,
        }
    }
}

/// Process resource limits collection.
///
/// This structure maintains all resource limits for a process as defined
/// in the POSIX standard and Linux. It supports access by resource ID
/// (e.g., RLIMIT_STACK, RLIMIT_CPU) to get or set specific limits.
pub struct Rlimits([Rlimit; RLIM_NLIMITS as usize]);

impl Default for Rlimits {
    /// Creates a default set of resource limits.
    ///
    /// Currently only initializes the stack size limit to the configured
    /// user stack size, leaving other limits at their default values.
    fn default() -> Self {
        let mut result = Self(Default::default());
        // Set the default stack size limit
        result[RLIMIT_STACK] = (axconfig::plat::USER_STACK_SIZE as u64).into();
        result
    }
}

impl Index<u32> for Rlimits {
    type Output = Rlimit;

    /// Gets a reference to the resource limit for the specified resource ID.
    ///
    /// # Arguments
    ///
    /// * `index` - The resource ID (e.g., RLIMIT_STACK)
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u32> for Rlimits {
    /// Gets a mutable reference to the resource limit for the specified resource ID.
    ///
    /// # Arguments
    ///
    /// * `index` - The resource ID (e.g., RLIMIT_STACK)
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}
