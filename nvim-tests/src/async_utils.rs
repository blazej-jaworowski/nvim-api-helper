use nvim_api_helper::{
    Result,
    nvim,
    async_utils::{
        AsyncError,
        init_static_dispatcher,
        static_dispatch,
    },
    run_async,
};

macro_rules! assert_async_error {
    ($value:expr, $error:expr) => {
        if let Err(nvim_api_helper::Error::Async(e)) = $value {
            assert_eq!(e, $error)
        } else {
            assert!(false, "Expected async error, got: {:?}", $value)
        }
    }
}
