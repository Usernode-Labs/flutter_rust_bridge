use crate::generalized_arc::base_arc::BaseArc;
use crate::rust_async::{MutexGuard};
use crate::rust_auto_opaque::inner::RustAutoOpaqueInner;
use crate::rust_auto_opaque::RustAutoOpaqueBase;
use crate::rust_opaque::RustOpaqueBase;
use tokio::sync::{Mutex, TryLockError};

impl<T, A: BaseArc<RustAutoOpaqueInner<T>>> RustAutoOpaqueBase<T, A> {
    pub fn new(value: T) -> Self {
        Self(RustOpaqueBase::new(RustAutoOpaqueInner::new(Mutex::new(
            value,
        ))))
    }

    pub fn blocking_read(&self) -> MutexGuard<'_, T> {
        self.0.data.blocking_lock()
    }

    pub fn blocking_write(&self) -> MutexGuard<'_, T> {
        self.0.data.blocking_lock()
    }

    pub async fn read(&self) -> MutexGuard<'_, T> {
        self.0.data.lock().await
    }

    pub async fn write(&self) -> MutexGuard<'_, T> {
        self.0.data.lock().await
    }

    pub fn try_read(&self) -> Result<MutexGuard<'_, T>, TryLockError> {
        self.0.data.try_lock()
    }

    pub fn try_write(&self) -> Result<MutexGuard<'_, T>, TryLockError> {
        self.0.data.try_lock()
    }
}

#[cfg(test)]
mod tests {
    use crate::RustAutoOpaqueNom;

    #[test]
    fn test_api_sync() {
        let opaque = RustAutoOpaqueNom::new(42);
        assert_eq!(*opaque.blocking_read(), 42);
        assert_eq!(*opaque.blocking_write(), 42);
        assert_eq!(*opaque.try_read().unwrap(), 42);
        assert_eq!(*opaque.try_write().unwrap(), 42);
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_api_async() {
        let opaque = RustAutoOpaqueNom::new(42);
        assert_eq!(*opaque.read().await, 42);
        assert_eq!(*opaque.write().await, 42);
    }

    #[test]
    fn test_clone() {
        let a = RustAutoOpaqueNom::new(42);
        let b = a.clone();
        *a.blocking_write() = 200;
        assert_eq!(*b.blocking_read(), 200);
    }
}
