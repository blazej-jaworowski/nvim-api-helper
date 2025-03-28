use nvim_api_helper::{
    Result,
    nvim,
};

#[nvim::plugin(nvim_oxi = nvim)]
fn nvim_api_helper_tests() -> nvim::Result<()> {
    Ok(())
}

#[nvim::test(nvim_oxi = nvim)]
fn basic_test() -> Result<()> {
    let var_key = "test_value";
    let original_value = String::from("Hello!");

    nvim::api::set_var(var_key, original_value.clone())?;
    let value = nvim::api::get_var::<String>(var_key)?;

    assert_eq!(value, original_value);

    Ok(())
}

mod buffer;
mod async_utils;
