use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait RwLockExt<T> {
    fn read_unwrap(&self) -> RwLockReadGuard<'_, T>;
    fn write_unwrap(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn read_unwrap(&self) -> RwLockReadGuard<'_, T> {
        self.read()
            .expect("RwLock should not be poisoned (writer panicked while holding lock?)")
    }
    fn write_unwrap(&self) -> RwLockWriteGuard<'_, T> {
        self.write()
            .expect("RwLock should not be poisoned (writer panicked while holding lock?)")
    }
}
