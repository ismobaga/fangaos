//! SMP-safe Spinlock
//!
//! This module provides a spinlock implementation for SMP systems.

use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// A spinlock for SMP synchronization
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Create a new spinlock
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Try to acquire the lock without blocking
    pub fn try_lock(&self) -> Option<SpinLockGuard<T>> {
        if self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }
    
    /// Acquire the lock, spinning until it's available
    pub fn lock(&self) -> SpinLockGuard<T> {
        // Spin until we acquire the lock
        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            // Hint to CPU that we're spinning
            core::hint::spin_loop();
        }
        
        SpinLockGuard { lock: self }
    }
    
    /// Check if the lock is currently held
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }
    
    /// Unlock the spinlock (internal use)
    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// RAII guard for SpinLock
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spinlock_basic() {
        let lock = SpinLock::new(42);
        
        assert!(!lock.is_locked());
        
        {
            let guard = lock.lock();
            assert_eq!(*guard, 42);
            assert!(lock.is_locked());
        }
        
        assert!(!lock.is_locked());
    }
    
    #[test]
    fn test_spinlock_try_lock() {
        let lock = SpinLock::new(100);
        
        let guard1 = lock.try_lock();
        assert!(guard1.is_some());
        
        // Can't acquire while already locked
        let guard2 = lock.try_lock();
        assert!(guard2.is_none());
        
        // Drop guard1 to release lock
        drop(guard1);
        
        // Now we should be able to acquire it
        let guard3 = lock.try_lock();
        assert!(guard3.is_some());
    }
    
    #[test]
    fn test_spinlock_mutation() {
        let lock = SpinLock::new(0);
        
        {
            let mut guard = lock.lock();
            *guard = 42;
        }
        
        {
            let guard = lock.lock();
            assert_eq!(*guard, 42);
        }
    }
}
