use std::sync::{OnceLock, mpsc};

use tokio::sync::oneshot;

use crate::nvim::{
    self,
    libuv::AsyncHandle,
};


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Oneshot recv error: {0}")]
    OneshotRecvError(#[from] oneshot::error::RecvError),

    #[error("Dispatcher lock poisoned")]
    LockPoisoned,

    #[error("Static dispatcher not initialized")]
    NoDispatcher,

    #[error("Dispatcher async handle not set")]
    NoAsyncHandle,

    #[error("Nvim error: {0}")]
    Nvim(#[from] nvim::Error),

    #[error("Nvim LibUV error: {0}")]
    NvimLibUV(#[from] nvim::libuv::Error),

    #[error("Channel send error")]
    ChannelSendError,
}

type Result<T> = std::result::Result<T, Error>;


pub struct Dispatcher {
    async_handle: AsyncHandle,
    func_tx: mpsc::Sender<Box<dyn FnOnce() + Send>>,
}

impl Dispatcher {

    pub fn new() -> Result<Dispatcher> {
        let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();

        let async_handle = AsyncHandle::new(move || {
            let func = rx.recv().expect("Receive failed");
            func()
        })?;

        Ok(Dispatcher {
            async_handle,
            func_tx: tx,
        })
    }

    pub async fn dispatch<R: Send + 'static>(&self, func: Box<dyn FnOnce() -> R + Send>) -> Result<R> {
        let (result_tx, result_rx) = oneshot::channel::<R>();

        let dispatch_func = Box::new(|| {
            _ = result_tx.send(func());
        });

        self.func_tx.send(dispatch_func).map_err(|_| Error::ChannelSendError)?;
        self.async_handle.send()?;

        Ok(result_rx.await?)
    }

}

static STATIC_DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();

pub fn init_static_dispatcher() -> Result<()> {
    let dispatcher = Dispatcher::new()?;

    _ = STATIC_DISPATCHER.set(dispatcher).inspect_err(|_| {
        nvim::print!("Static dispatcher initialized already");
    });

    Ok(())
}

pub async fn static_dispatch<R: Send + 'static>(func: Box<dyn FnOnce() -> R + Send>) -> Result<R> {
    let dispatcher = STATIC_DISPATCHER.get()
        .ok_or(Error::NoDispatcher)?;

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
