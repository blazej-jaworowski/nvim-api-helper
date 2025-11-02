use crate::{async_dispatch, async_runtime, buffer, nvim};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("InvalidType error")]
    InvalidType,

    #[error("Nvim error: {0}")]
    Nvim(#[from] nvim::Error),

    #[error("NvimApi error: {0}")]
    NvimApi(#[from] nvim::api::Error),

    #[error("Lua error: {0}")]
    Lua(#[from] nvim::mlua::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("LibUV error: {0}")]
    LibUV(#[from] nvim::libuv::Error),

    #[error("AsyncDispatch error: {0}")]
    AsyncDispatch(#[from] async_dispatch::Error),

    #[error("AsyncRuntime error: {0}")]
    AsyncRuntime(#[from] async_runtime::Error),

    #[error("Buffer error: {0}")]
    Buffer(#[from] buffer::BufferError),

    #[error("Error: {0}")]
    Custom(String),
}

pub type Result<R> = std::result::Result<R, Error>;
