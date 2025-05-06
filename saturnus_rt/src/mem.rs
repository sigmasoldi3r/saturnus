// Some memory management utilitites.

use std::{fmt::Debug, ops::Deref, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct St<T> {
    data: Arc<Mutex<T>>,
}
impl<T> St<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }
}
impl<T> Deref for St<T> {
    type Target = Mutex<T>;
    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}
impl<T> Clone for St<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
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
impl<T> IntoRefCount for T {}
