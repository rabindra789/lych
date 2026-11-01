use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
    saved_daif: Option<u64>,
    _not_send: PhantomData<*mut ()>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        self.acquire(None)
    }

    pub fn lock_irqsave(&self) -> SpinLockGuard<'_, T> {
        let saved_daif: u64;

        unsafe {
            core::arch::asm!(
                "mrs {flags}, daif",
                "msr daifset, #2",
                "isb",
                flags = out(reg) saved_daif,
                options(nostack, preserves_flags)
            );
        }

        self.acquire(Some(saved_daif))
    }

    fn acquire(&self, saved_daif: Option<u64>) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        SpinLockGuard {
            lock: self,
            saved_daif,
            _not_send: PhantomData,
        }
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);

        if let Some(flags) = self.saved_daif {
            unsafe {
                core::arch::asm!(
                    "msr daif, {flags}",
                    "isb",
                    flags = in(reg) flags,
                    options(nostack, preserves_flags)
                );
            }
        }
    }
}
