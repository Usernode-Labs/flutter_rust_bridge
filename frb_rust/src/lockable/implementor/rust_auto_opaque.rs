use crate::for_generated::{BaseArc, RustAutoOpaqueInner, RustOpaqueBase};
use crate::lockable::base::Lockable;
use crate::lockable::order::LockableOrder;
use std::future::Future;
use std::pin::Pin;

impl<T: Send, A: BaseArc<RustAutoOpaqueInner<T>>> Lockable
    for RustOpaqueBase<RustAutoOpaqueInner<T>, A>
{
    type MutexGuard<'a>
        = crate::rust_async::MutexGuard<'a, T>
    where
        A: 'a;

    fn lockable_order(&self) -> LockableOrder {
        self.order
    }

    fn lockable_decode_sync_ref(&self) -> Self::MutexGuard<'_> {
        self.data.blocking_lock()
    }

    fn lockable_decode_sync_ref_mut(&self) -> Self::MutexGuard<'_> {
        self.data.blocking_lock()
    }

    fn lockable_decode_async_ref<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Self::MutexGuard<'a>> + Send + 'a>>
    where
        Self: Sync + 'a,
    {
        Box::pin(async move { self.data.lock().await })
    }

    fn lockable_decode_async_ref_mut<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Self::MutexGuard<'a>> + Send + 'a>>
    where
        Self: Sync + 'a,
    {
        Box::pin(async move { self.data.lock().await })
    }
}
