pub mod error;
pub mod lua;
pub mod lua_plugins;

pub use nvim_oxi as nvim;
pub use nvim::mlua as mlua;

pub use error::{Result, Error};
