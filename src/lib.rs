pub mod error;
pub mod logging;

pub mod async_dispatch;
pub mod async_runtime;

pub mod buffer;

pub mod lua;
pub mod lua_plugins;

use nvim::mlua;
use nvim_oxi as nvim;

pub use error::{Error, Result};

#[cfg(feature = "nvim_tests")]
pub mod nvim_tests {
    use super::*;

    #[nvim::test(nvim_oxi = nvim)]
    fn basic_test() -> nvim::Result<()> {
        let var_key = "test_value";
        let original_value = String::from("Hello!");

        nvim::api::set_var(var_key, original_value.clone())?;
        let value = nvim::api::get_var::<String>(var_key)?;

        assert_eq!(value, original_value);

        Ok(())
    }
}
