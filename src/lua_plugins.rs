use crate::{
    Result,
    lua::lua_get_global_path,
};
use crate::mlua::{
    Function, Table,
    IntoLuaMulti, FromLuaMulti,
};

pub fn require_plugin<'lua>(plugin_name: &str) -> Result<Table<'lua>> {
    let require_func: Function = lua_get_global_path("require")?;
    Ok(require_func.call(plugin_name)?)
}

pub fn require_call_setup_val<'lua, A, R>(plugin_name: &str, args: A) -> Result<R>
where
    A: IntoLuaMulti<'lua>,
    R: FromLuaMulti<'lua>,
{
    Ok(require_plugin(plugin_name)?
        .get::<&str, Function>("setup")?
        .call(args)?)
}

pub fn require_call_setup<'lua, A>(plugin_name: &str, args: A) -> Result<()>
where
    A: IntoLuaMulti<'lua>,
{
    require_call_setup_val(plugin_name, args)
}
