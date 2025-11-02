use std::sync::{OnceLock, mpsc};

use tokio::sync::oneshot;

use crate::error;
use crate::nvim::{self, libuv::AsyncHandle};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Nvim LibUV error: {0}")]
    NvimLibUV(#[from] nvim::libuv::Error),

    #[error("Dispatch function send error")]
    FuncSend,

    #[error("Result receive error: {0}")]
    ResultRecv(#[from] oneshot::error::RecvError),
}

type Result<T> = std::result::Result<T, Error>;

struct Dispatcher {
    async_handle: AsyncHandle,
    func_tx: mpsc::Sender<Box<dyn FnOnce() + Send>>,
}

impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher")
            .field("func_tx", &self.func_tx)
            .finish()
    }
}

impl Dispatcher {
    pub fn new() -> Result<Dispatcher> {
        let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();

        let async_handle = AsyncHandle::new(move || match rx.recv() {
            Ok(f) => f(),
            Err(e) => error!("Error while receiving dispatch func: {e}"),
        })?;

        Ok(Dispatcher {
            async_handle,
            func_tx: tx,
        })
    }

    pub async fn dispatch<F, R>(&self, func: F) -> Result<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (result_tx, result_rx) = oneshot::channel::<R>();

        let dispatch_func = Box::new(|| {
            if result_tx.send(func()).is_err() {
                error!("Error while sending dispatch result");
            }
        });

        self.func_tx
            .send(dispatch_func)
            .map_err(|_| Error::FuncSend)?;
        self.async_handle.send()?;

        Ok(result_rx.await?)
    }
}

static DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();

pub fn init_dispatcher() -> Result<()> {
    if DISPATCHER.get().is_some() {
        return Ok(());
    }

    DISPATCHER
        .set(Dispatcher::new()?)
        .expect("We just checked that this value is not set");

    Ok(())
}

fn get_dispatcher() -> &'static Dispatcher {
    DISPATCHER
        .get()
        .expect("Dispatcher should have been initialized")
}

pub async fn async_dispatch<F, R>(func: F) -> Result<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    get_dispatcher().dispatch(func).await
}

#[macro_export]
macro_rules! async_dispatch {
    ($( $tt:tt )*) => {
        $crate::async_dispatch::async_dispatch(move || {
            $( $tt )*
        })
    };
}
