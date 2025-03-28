use std::{
    sync::{Mutex, Arc, OnceLock},
    cell::RefCell,
};
use futures::channel::oneshot;

use crate::{
    Result,
    nvim::libuv::AsyncHandle,
};


#[derive(thiserror::Error, Debug)]
pub enum AsyncError {
    #[error("OneshotCanceled error: {0}")]
    OneshotCanceled(#[from] oneshot::Canceled),

    #[error("Dispatcher lock poisoned")]
    LockPoisoned,

    #[error("Static dispatcher not initialized")]
    NoDispatcher,

    #[error("Dispatcher async handle not set")]
    NoAsyncHandle,
}

type DispatcherFunc = Option<Box<dyn FnOnce() + Send>>;

pub struct Dispatcher {
    async_handle: AsyncHandle,
    func: Arc<Mutex<RefCell<DispatcherFunc>>>,
    dispatch_lock: futures::lock::Mutex<()>,
}

impl Dispatcher {

    pub fn new() -> Result<Dispatcher> {
        let func: Arc<Mutex<RefCell<DispatcherFunc>>> = Default::default();

        let func_clone = func.clone();
        let async_handle = AsyncHandle::new(move || {
            let func_guard = func_clone.lock()
                .expect("Dispatcher lock poisoned");
            let func = func_guard.take();

            func.expect("Dispatcher called with no function")();
        })?;

        Ok(Dispatcher {
            async_handle, func,
            dispatch_lock: Default::default(),
        })
    }

    pub async fn dispatch<R: Send + 'static>(&self, func: Box<dyn FnOnce() -> R + Send>) -> Result<R> {
        let (tx, rx) = oneshot::channel::<R>();

        let _dispatch_guard = self.dispatch_lock.lock().await;

        {
            let func_guard = self.func.lock()
                .map_err(|_| AsyncError::LockPoisoned)?;

            *func_guard.borrow_mut() = Some(Box::new(move || {
                _ = tx.send(func());
            }));
        }

        self.async_handle.send()?;

        let result = rx.await.map_err(AsyncError::OneshotCanceled)?;

        Ok(result)
    }

}

static STATIC_DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();

pub fn init_static_dispatcher() -> Result<()> {
    let dispatcher = Dispatcher::new()?;
    _ = STATIC_DISPATCHER.set(dispatcher);

    Ok(())
}

pub async fn static_dispatch<R: Send + 'static>(func: Box<dyn FnOnce() -> R + Send>) -> Result<R> {
    let dispatcher = STATIC_DISPATCHER.get()
        .ok_or(AsyncError::NoDispatcher)?;

    dispatcher.dispatch(func).await
}

#[macro_export]
macro_rules! run_async {
    ($( $tt:tt )*) => {
        $crate::async_utils::static_dispatch(Box::new(move || {
            $( $tt )*
        }))
    };
}
