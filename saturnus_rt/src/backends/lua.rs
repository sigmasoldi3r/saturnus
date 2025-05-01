use std::{clone, sync::Arc};

use minijinja::value;
use mlua::{IntoLua, Variadic};

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
            mlua::Value::Integer(v) => Any::Integer(v),
            mlua::Value::Number(v) => v.into_saturnus(),
            mlua::Value::String(v) => Any::String(v.to_string_lossy()),
            mlua::Value::Table(table) => table_to_saturnus(table),
            _ => unimplemented!(
                "Can't convert them contextless or does not make sense, use other conversion methods."
            ),
        }
    }
}

fn convert_any(lua: Arc<mlua::Lua>) -> impl Fn(mlua::Value) -> Any {
    move |value| match value {
        mlua::Value::Function(function) => Callable::new({
            let lua = lua.clone();
            move |args| {
                let mut argv = Variadic::<mlua::Value>::new();
                for arg in args.into_iter() {
                    argv.push(arg.into_lua(lua.clone()).unwrap());
                }
                let result = function
                    .call::<mlua::Value>(argv)
                    .map(convert_any(lua.clone()));
                match result {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!("{err:?}");
                        Any::Unit
                    }
                }
            }
        })
        .into_saturnus(),
        mlua::Value::LightUserData(light_user_data) => todo!(),
        mlua::Value::Thread(thread) => todo!(),
        mlua::Value::UserData(any_user_data) => todo!(),
        mlua::Value::Error(error) => todo!(),
        mlua::Value::Other(value_ref) => todo!(),
        other => other.into_saturnus(),
    }
}

trait IntoLuaArc {
    fn into_lua(self, lua: Arc<mlua::Lua>) -> mlua::Result<mlua::Value>;
}
impl IntoLuaArc for Table {
    fn into_lua(self, lua: Arc<mlua::Lua>) -> mlua::Result<mlua::Value> {
        let tbl = lua.create_table()?;
        for (k, v) in self.into_iter() {
            tbl.set(k.into_lua(lua.clone())?, v.into_lua(lua.clone())?)?;
        }
        mlua::Result::Ok(mlua::Value::Table(tbl))
    }
}
impl IntoLuaArc for Callable {
    fn into_lua(self, lua: Arc<mlua::Lua>) -> mlua::Result<mlua::Value> {
        let inner = self.into_inner();
        mlua::Result::Ok(mlua::Value::Function(lua.create_async_function({
            let lua = lua.clone();
            move |_, args: mlua::Variadic<mlua::Value>| {
                let inner = inner.clone();
                let lua = lua.clone();
                async move {
                    let result = inner(args.into_iter().map(convert_any(lua.clone())).collect());

                    Ok(result.into_lua(lua.clone()))
                }
            }
        })?))
    }
}
impl IntoLuaArc for Any {
    fn into_lua(self, lua: Arc<mlua::Lua>) -> mlua::Result<mlua::Value> {
        match self {
            Any::Integer(value) => mlua::Result::Ok(mlua::Value::Integer(value)),
            Any::Decimal(value) => mlua::Result::Ok(mlua::Value::Number(value.into_inner())),
            Any::Boolean(value) => mlua::Result::Ok(mlua::Value::Boolean(value)),
            Any::String(value) => mlua::Result::Ok(mlua::Value::String(lua.create_string(value)?)),
            Any::Object(table) => table.into_lua(lua),
            Any::Function(callable) => callable.into_lua(lua),
            Any::Unit => mlua::Result::Ok(mlua::Value::Nil),
            _ => unimplemented!(),
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
    fn init_globals(&self, lua: Arc<mlua::Lua>) -> Result<(), RuntimeError> {
        let globals = lua.globals();
        for (k, v) in self.config.globals.iter() {
            globals
                .set(
                    k.clone().into_lua(lua.clone()).map_err(|err| RuntimeError {
                        message: format!("Panic! Initialization of environment failed! Cannot set key:\n{err}"),
                        source_name: "".into(),
                    })?,
                    v.clone().into_lua(lua.clone()).map_err(|err| RuntimeError {
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
        let lua = Arc::new(mlua::Lua::new());
        self.init_globals(lua.clone())?;
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
