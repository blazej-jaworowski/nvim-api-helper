use thiserror::Error;
use std::result::Result as StdResult;
use std::io::Error as IOError;
use crate::nvim::Error as NvimError;
use crate::nvim::api::Error as NvimApiError;
use crate::mlua::prelude::LuaError;
use crate::nvim::libuv::Error as LibUVError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("InvalidType error")]
    InvalidType,

    #[error("Nvim error: {0}")]
    NvimError(#[from] NvimError),

    #[error("NvimApi error: {0}")]
    NvimApiError(#[from] NvimApiError),

    #[error("Lua error: {0}")]
    LuaError(#[from] LuaError),

    #[error("IO error: {0}")]
    IOError(#[from] IOError),

    #[error("LibUV error: {0}")]
    LibUVError(#[from] LibUVError),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<R> = StdResult<R, Error>;
