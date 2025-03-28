use thiserror::Error;
use std::{
    result::Result as StdResult,
    io::Error as IOError,
};
use crate::{
    nvim::{
        Error as NvimError,
        api::Error as NvimApiError,
        libuv::Error as LibUVError,
    },
    mlua::prelude::LuaError,
    async_utils::AsyncError,
    buffer::BufferError,
};


#[derive(Error, Debug)]
pub enum Error {
    #[error("InvalidType error")]
    InvalidType,

    #[error("Nvim error: {0}")]
    Nvim(#[from] NvimError),

    #[error("NvimApi error: {0}")]
    NvimApi(#[from] NvimApiError),

    #[error("Lua error: {0}")]
    Lua(#[from] LuaError),

    #[error("IO error: {0}")]
    IO(#[from] IOError),

    #[error("LibUV error: {0}")]
    LibUV(#[from] LibUVError),

    #[error("Async error: {0}")]
    Async(#[from] AsyncError),

    #[error("Buffer error: {0}")]
    Buffer(#[from] BufferError),
}

pub type Result<R> = StdResult<R, Error>;
