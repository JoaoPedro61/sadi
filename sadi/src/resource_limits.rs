//! Resource limits for controlling concurrent service creation.
//!
//! This module provides mechanisms to limit the concurrency of service creation
//! per binding, preventing resource exhaustion and controlling creation bursts.

/// Policy for handling limit violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Fail immediately when limit is reached.
    Deny,
    /// Block and wait until a slot becomes available.
    Block,
}

/// Configuration for resource limits on a binding.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Maximum number of concurrent creations allowed.
    pub max_concurrent_creations: Option<usize>,
    /// Policy to apply when limit is reached.
    pub policy: Policy,
}

impl Limits {
    /// Create limits with the given concurrency and policy.
    pub fn with_concurrency(max: usize, policy: Policy) -> Self {
        Self {
            max_concurrent_creations: Some(max),
            policy,
        }
    }

    /// Create limits that deny when exceeded.
    pub fn deny(max: usize) -> Self {
        Self::with_concurrency(max, Policy::Deny)
    }

    /// Create limits that block when exceeded.
    pub fn block(max: usize) -> Self {
        Self::with_concurrency(max, Policy::Block)
    }
}

/// Thread-safe resource limiter using atomics and condition variables.
#[cfg(feature = "thread-safe")]
pub struct ResourceLimiter {
    max: usize,
    counter: std::sync::atomic::AtomicUsize,
    condvar: std::sync::Condvar,
    mutex: std::sync::Mutex<()>,
}

#[cfg(feature = "thread-safe")]
impl ResourceLimiter {
    /// Create a new resource limiter with the given maximum.
    pub fn new(max: usize) -> Self {
        Self {
            max,
            counter: std::sync::atomic::AtomicUsize::new(0),
            condvar: std::sync::Condvar::new(),
            mutex: std::sync::Mutex::new(()),
        }
    }

    /// Try to acquire a slot without blocking.
    /// Returns Some(Guard) if successful, None if limit reached.
    pub fn try_acquire(&self) -> Option<AcquireGuard<'_>> {
        use std::sync::atomic::Ordering;

        loop {
            let current = self.counter.load(Ordering::Acquire);
            if current >= self.max {
                return None;
            }

            if self
                .counter
                .compare_exchange(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Some(AcquireGuard { limiter: self });
            }
        }
    }

    /// Acquire a slot, blocking if necessary until one becomes available.
    pub fn acquire_blocking(&self) -> AcquireGuard<'_> {
        use std::sync::atomic::Ordering;

        loop {
            // Try non-blocking first
            if let Some(guard) = self.try_acquire() {
                return guard;
            }

            // Block and wait for notification
            let _lock = self.mutex.lock().unwrap();
            let _guard = self
                .condvar
                .wait_while(_lock, |_| {
                    self.counter.load(Ordering::Acquire) >= self.max
                })
                .unwrap();
        }
    }

    /// Release a slot and notify waiting threads.
    fn release(&self) {
        use std::sync::atomic::Ordering;

        self.counter.fetch_sub(1, Ordering::Release);
        self.condvar.notify_one();
    }
}

/// Non-thread-safe resource limiter using simple counter.
#[cfg(not(feature = "thread-safe"))]
pub struct ResourceLimiter {
    max: usize,
    counter: std::cell::Cell<usize>,
}

#[cfg(not(feature = "thread-safe"))]
impl ResourceLimiter {
    /// Create a new resource limiter with the given maximum.
    pub fn new(max: usize) -> Self {
        Self {
            max,
            counter: std::cell::Cell::new(0),
        }
    }

    /// Try to acquire a slot without blocking.
    /// Returns Some(Guard) if successful, None if limit reached.
    pub fn try_acquire(&self) -> Option<AcquireGuard<'_>> {
        let current = self.counter.get();
        if current >= self.max {
            return None;
        }

        self.counter.set(current + 1);
        Some(AcquireGuard { limiter: self })
    }

    /// In non-thread-safe mode, blocking is not supported.
    /// This will panic if called.
    pub fn acquire_blocking(&self) -> AcquireGuard<'_> {
        panic!("Blocking acquire not supported in non-thread-safe mode")
    }

    /// Release a slot.
    fn release(&self) {
        let current = self.counter.get();
        self.counter.set(current.saturating_sub(1));
    }
}

/// RAII guard that releases the limiter slot on drop.
pub struct AcquireGuard<'a> {
    limiter: &'a ResourceLimiter,
}

impl<'a> Drop for AcquireGuard<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_construction() {
        let limits = Limits::deny(5);
        assert_eq!(limits.max_concurrent_creations, Some(5));
        assert_eq!(limits.policy, Policy::Deny);

        let limits = Limits::block(10);
        assert_eq!(limits.max_concurrent_creations, Some(10));
        assert_eq!(limits.policy, Policy::Block);
    }

    #[test]
    fn limiter_basic_acquire() {
        let limiter = ResourceLimiter::new(2);

        let _g1 = limiter.try_acquire().expect("First acquire should succeed");
        let _g2 = limiter.try_acquire().expect("Second acquire should succeed");
        assert!(
            limiter.try_acquire().is_none(),
            "Third acquire should fail"
        );
    }

    #[test]
    fn limiter_release_on_drop() {
        let limiter = ResourceLimiter::new(1);

        {
            let _g1 = limiter.try_acquire().expect("First acquire should succeed");
            assert!(limiter.try_acquire().is_none(), "Should be at limit");
        } // g1 dropped here

        let _g2 = limiter
            .try_acquire()
            .expect("After drop, acquire should succeed");
    }

    #[cfg(feature = "thread-safe")]
    #[test]
    fn limiter_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let limiter = Arc::new(ResourceLimiter::new(2));
        let mut handles = vec![];

        for _ in 0..5 {
            let limiter = Arc::clone(&limiter);
            handles.push(thread::spawn(move || {
                if let Some(_guard) = limiter.try_acquire() {
                    thread::sleep(std::time::Duration::from_millis(10));
                    true
                } else {
                    false
                }
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let successes = results.iter().filter(|&&r| r).count();

        // At most 2 should succeed simultaneously
        assert!(successes <= 2);
    }
}
