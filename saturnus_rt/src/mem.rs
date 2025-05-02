// Some memory management utilitites.

use std::{
    fmt::{Debug, Display, Pointer},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

pub struct StGuard<'a, T> {
    guard: MutexGuard<'a, T>,
}
impl<T> Deref for StGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}
impl<T> DerefMut for StGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

pub struct St<T> {
    handle: Arc<Mutex<T>>,
}
impl<T> St<T> {
    pub fn new(value: T) -> Self {
        Self {
            handle: Arc::new(Mutex::new(value)),
        }
    }
    pub fn lock(&self) -> StGuard<T> {
        StGuard {
            guard: self.handle.lock().expect("Poisoned handle!"),
        }
    }
}
impl<T> Clone for St<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}
impl<T: Debug> Debug for St<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("St").field("handle", &self.handle).finish()
    }
}
impl<T: Display> Display for St<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

pub trait IntoRefCount
where
    Self: Sized,
{
    fn into_rc(self) -> St<Self> {
        St::new(self)
    }
}
