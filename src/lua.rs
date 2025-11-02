use crate::mlua::{self, FromLua, Value};
use crate::{Error, Result};

#[macro_export]
macro_rules! lua_tuple {
    () => {
        ()
    };

    ($( $value:tt ),* $(,)? ) => {{
        use $crate::lua_value;
        ( $( lua_value!($value) ),* )
    }};
}

#[macro_export]
macro_rules! lua_table {
    () => {{
        use $crate::mlua::{lua, Value};
        Value::Table(lua().create_table()?)
    }};

    ($( $key:expr => $value:tt ),* $(,)? ) => {{
        use $crate::{mlua::{lua, Value}, lua_value};
        Value::Table(lua().create_table_from(
            [ $( (lua_value!($key), lua_value!($value)) ),* ]
        )?)
    }};
}

#[macro_export]
macro_rules! lua_array {
    () => {{
        use $crate::mlua::{lua, Value};
        Value::Table(lua().create_table()?)
    }};

    ($( $value:tt ),* $(,)? ) => {{
        use $crate::{mlua::{lua, Value}, lua_value};
        Value::Table(lua().create_sequence_from(
            [ $( lua_value!($value) ),* ]
        )?)
    }};
}

#[macro_export]
macro_rules! lua_value {
    () => {{
        use $crate::mlua::Value;
        Value::NULL
    }};

    ({ $( $key:expr => $value:tt ),* $(,)? }) => {{
        use $crate::lua_table;
        lua_table!{$( $key => $value ),*}
    }};

    ([ $( $value:tt ),* $(,)? ]) => {{
        use $crate::lua_array;
        lua_array![$( $value ),*]
    }};

    (( $( $value:tt ),* $(,)? )) => {{
        use $crate::lua_tuple;
        lua_tuple!($( $value ),*)
    }};

    ($value:tt) => {{
        use $crate::mlua::{IntoLua, lua};
        $value.into_lua(&lua())?
    }};
}

pub fn lua_get_value_path<T: FromLua>(obj: Value, path: &str) -> Result<T> {
    let mut table = if let Value::Table(t) = obj {
        t
    } else {
        return Err(Error::InvalidType);
    };

    let parts: Vec<&str> = path.split(".").collect();

    for part in &parts[0..(parts.len() - 1)] {
        table = table.get(*part)?;
    }

    let part = *parts.last().unwrap();

    Ok(table.get(part)?)
}

pub fn lua_get_global_path<T: FromLua>(path: &str) -> Result<T> {
    let globals = Value::Table(mlua::lua().globals());
    lua_get_value_path(globals, path)
}
