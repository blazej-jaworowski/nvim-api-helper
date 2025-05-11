use thiserror::Error;
use crate::{
    nvim,
    async_utils,
    buffer,
};


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

    #[error("Async error: {0}")]
    Async(#[from] async_utils::Error),

    #[error("Buffer error: {0}")]
    Buffer(#[from] buffer::BufferError),

    #[error("Error: {0}")]
    Custom(String),
}

pub type Result<R> = std::result::Result<R, Error>;
