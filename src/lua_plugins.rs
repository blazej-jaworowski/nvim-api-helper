use crate::mlua::{FromLuaMulti, Function, IntoLuaMulti, Table};
use crate::{Result, lua::lua_get_global_path};

pub fn require_plugin(plugin_name: &str) -> Result<Table> {
    let require_func: Function = lua_get_global_path("require")?;
    Ok(require_func.call(plugin_name)?)
}

pub fn require_call_setup_val<A, R>(plugin_name: &str, args: A) -> Result<R>
where
    A: IntoLuaMulti,
    R: FromLuaMulti,
{
    Ok(require_plugin(plugin_name)?
        .get::<Function>("setup")?
        .call(args)?)
}

pub fn require_call_setup<A>(plugin_name: &str, args: A) -> Result<()>
where
    A: IntoLuaMulti,
{
    require_call_setup_val(plugin_name, args)
}
