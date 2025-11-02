use std::sync::OnceLock;

use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Runtime init error: {0}")]
    RuntimeInit(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

static ASYNC_RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn init_runtime() -> Result<()> {
    if ASYNC_RUNTIME.get().is_some() {
        return Ok(());
    }

    ASYNC_RUNTIME
        .set(Runtime::new()?)
        .expect("We just checked that this value is not set");

    Ok(())
}

fn get_runtime() -> &'static Runtime {
    ASYNC_RUNTIME
        .get()
        .expect("Async runtime should have been initialized")
}

pub fn get_handle() -> &'static Handle {
    get_runtime().handle()
}

pub fn spawn<F>(f: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    get_runtime().spawn(f)
}
