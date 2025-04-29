use mlua::IntoLua;

use crate::core::{Any, Callable, IntoSaturnus, Table};

use super::{RtEnv, Runtime, RuntimeError};

fn table_to_saturnus(table: mlua::Table) -> Any {
    let mut out = Table::new();
    for pair in table.pairs() {
        let (k, v): (mlua::Value, mlua::Value) = pair.unwrap();
        out.set(k.into_saturnus(), v.into_saturnus());
    }
    out.into_saturnus()
}

impl IntoSaturnus for mlua::Value {
    fn into_saturnus(self) -> Any {
        match self {
            mlua::Value::Nil => Any::Unit,
            mlua::Value::Boolean(v) => Any::Boolean(v),
            mlua::Value::LightUserData(light_user_data) => todo!(),
            mlua::Value::Integer(v) => Any::Integer(v),
            mlua::Value::Number(v) => v.into_saturnus(),
            mlua::Value::String(v) => Any::String(v.to_string_lossy()),
            mlua::Value::Table(table) => table_to_saturnus(table),
            mlua::Value::Function(function) => Any::Unit,
            mlua::Value::Thread(thread) => todo!(),
            mlua::Value::UserData(any_user_data) => todo!(),
            mlua::Value::Error(error) => todo!(),
            mlua::Value::Other(value_ref) => todo!(),
        }
    }
}

impl IntoLua for Table {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = lua.create_table_from(self.into_iter())?;
        mlua::Result::Ok(mlua::Value::Table(tbl))
    }
}
impl IntoLua for Callable {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let inner = self.into_inner();
        mlua::Result::Ok(mlua::Value::Function(lua.create_function(
            move |lua, args: mlua::Variadic<mlua::Value>| {
                let result = inner(args.into_iter().map(IntoSaturnus::into_saturnus).collect());
                Ok(result.into_lua(lua))
            },
        )?))
    }
}
impl IntoLua for Any {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        match self {
            Any::Integer(value) => mlua::Result::Ok(mlua::Value::Integer(value)),
            Any::Decimal(value) => mlua::Result::Ok(mlua::Value::Number(value.into_inner())),
            Any::Boolean(value) => mlua::Result::Ok(mlua::Value::Boolean(value)),
            Any::String(value) => mlua::Result::Ok(mlua::Value::String(lua.create_string(value)?)),
            Any::Object(table) => table.into_lua(lua),
            Any::Function(callable) => callable.into_lua(lua),
            Any::Unit => mlua::Result::Ok(mlua::Value::Nil),
        }
    }
}

pub struct LuaRt {
    config: RtEnv,
}
impl LuaRt {
    pub fn default(config: RtEnv) -> Self {
        Self { config }
    }
    fn init_globals(&self, lua: &mlua::Lua) -> Result<(), RuntimeError> {
        let globals = lua.globals();
        for (k, v) in self.config.globals.iter() {
            globals
                .set(
                    k.clone().into_lua(&lua).map_err(|err| RuntimeError {
                        message: format!("Panic! Initialization of environment failed! Cannot set key:\n{err}"),
                        source_name: "".into(),
                    })?,
                    v.clone().into_lua(&lua).map_err(|err| RuntimeError {
                        message: format!("Panic! Initialization of environment failed! Cannot set value:\n{err}"),
                        source_name: "".into(),
                    })?,
                )
                .map_err(|err| RuntimeError {
                    message: format!("Panic! Initialization of environment failed! Cannot mutate global environment:\n{err}"),
                    source_name: "".into(),
                })?;
        }
        Ok(())
    }
}
impl Runtime for LuaRt {
    fn run(&mut self, chunks: Vec<(String, String)>) -> Result<(), RuntimeError> {
        let lua = mlua::Lua::new();
        self.init_globals(&lua)?;
        for (code, source_name) in chunks.into_iter() {
            lua.load(format!("do\n\n{code}\n\nend"))
                .exec()
                .map_err(|err| RuntimeError {
                    message: format!("{err}"),
                    source_name,
                })?;
        }
        Ok(())
    }
}
