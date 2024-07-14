use std::sync::{Mutex, MutexGuard};

pub trait IgnoreLockPoison<T> {
    fn lock_or_panic(&self) -> MutexGuard<T>;
}
impl<T> IgnoreLockPoison<T> for Mutex<T> {
    /// 如果发生了lock poison，则直接panic
    #[allow(clippy::unwrap_used)]
    fn lock_or_panic(&self) -> MutexGuard<T> {
        self.lock().unwrap()
    }
}
