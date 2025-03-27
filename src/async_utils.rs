use std::{
    sync::{Mutex, Arc, OnceLock},
    cell::{RefCell, OnceCell},
};

use crate::{
    Result,
    nvim::libuv::AsyncHandle,
};


#[derive(thiserror::Error, Debug)]
pub enum AsyncError {
    #[error("Dispatcher lock poisoned")]
    LockPoisoned,

    #[error("Static dispatcher not initialized")]
    NoDispatcher,
}


struct InnerDispatcher {
    async_handle: OnceCell<AsyncHandle>,
    func: RefCell<Option<Box<dyn Fn() + Send>>>,
}

impl InnerDispatcher {

    // Must be called from nvim thread
    fn call_func(&self) {
        self.func.borrow().as_ref().expect("Dispatcher called with no func")()
    }

    // Can be called from anywhere
    pub fn dispatch(&self, func: Box<dyn Fn() + Send>) -> Result<()> {
        *self.func.borrow_mut() = Some(func);
        Ok(
            self.async_handle.get().expect("Async handle not set")
                .send()?
        )
    }

}

#[derive(Clone)]
pub struct Dispatcher {
    inner: Arc<Mutex<InnerDispatcher>>,
}

impl Dispatcher {

    pub fn new() -> Result<Dispatcher> {
        let inner = Arc::new(Mutex::new(InnerDispatcher {
            async_handle: OnceCell::new(),
            func: RefCell::new(None),
        }));

        let cloned_inner = inner.clone();
        let async_handle = AsyncHandle::new(move || {
            cloned_inner.lock()
                .map_err(|_| AsyncError::LockPoisoned).unwrap()
                .call_func();
        })?;

        _ = inner.lock()
            .map_err(|_| AsyncError::LockPoisoned)?
            .async_handle.set(async_handle);

        Ok(Dispatcher { inner })
    }

    pub fn dispatch(&self, func: Box<dyn Fn() + Send>) -> Result<()> {
        self.inner.lock()
            .map_err(|_| AsyncError::LockPoisoned)?
            .dispatch(func)
    }

}

static STATIC_DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();

pub fn init_static_dispatcher() -> Result<()> {
    let dispatcher = Dispatcher::new()?;
    _ = STATIC_DISPATCHER.set(dispatcher);

    Ok(())
}

pub fn static_dispatch(func: Box<dyn Fn() + Send>) -> Result<()> {
    let dispatcher = STATIC_DISPATCHER.get()
        .ok_or(AsyncError::NoDispatcher)?;
    dispatcher.dispatch(func)
}

#[macro_export]
macro_rules! run_async {
    ($( $tt:tt )*) => {
        $crate::async_utils::static_dispatch(Box::new(move || {
            $( $tt )*
        }))
    };
}
