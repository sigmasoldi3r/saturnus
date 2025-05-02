pub mod internal;
pub mod program;
pub mod types;

use std::{error::Error, fmt::Display, ops::Deref};

use mlua::{ExternalError, IntoLua as _, Lua};
use program::Program;
use saturnstd::Ir;
use types::{Any, Function, IntoSaturnus, Str, Table};

use crate::mem::*;

/// # Saturnus Virtual Machine
///
/// This is the backend that runs saturnus.
///
/// Hosts creation and management of the native runtime.
pub struct StVm {
    rt: Lua,
}

impl IntoRefCount for StVm {}

#[derive(Debug)]
pub enum StError {
    Internal(Box<dyn Error>),
}
unsafe impl Send for StError {}
unsafe impl Sync for StError {}
impl Error for StError {}
impl Display for StError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StError::Internal(error) => (error.deref() as &dyn Display).fmt(f),
        }
    }
}

pub type Result<T> = core::result::Result<T, StError>;

trait IntoStResult<T> {
    fn wrap(self) -> Result<T>;
}

impl<T> IntoStResult<T> for mlua::Result<T> {
    fn wrap(self) -> Result<T> {
        self.map_err(|error| StError::Internal(Box::new(error)))
    }
}

impl StVm {
    pub fn new() -> St<Self> {
        let rt = Lua::new();
        Self::from_rt(rt)
    }

    fn from_rt(rt: Lua) -> St<Self> {
        Self { rt }.into_rc()
    }

    pub fn create_table(&mut self) -> Result<Table> {
        Ok(Table {
            value: self.rt.create_table().wrap()?,
        })
    }

    pub fn create_fn<F, FR>(&mut self, func: F) -> Result<Function>
    where
        F: FnMut(St<StVm>, Vec<Any>) -> FR + Send + Sync + 'static,
        FR: Future<Output = Result<Any>> + Send + 'static,
    {
        let func = St::new(func);
        let out = self
            .rt
            .create_async_function(move |lua, args: mlua::Variadic<mlua::Value>| {
                let func = func.clone();
                let vm = Self::from_rt(lua.clone());
                async move {
                    let argv: Vec<Any> =
                        args.into_iter().map(IntoSaturnus::into_saturnus).collect();
                    let result = {
                        let mut func = func.lock();
                        func(vm, argv)
                    }
                    .await
                    .map_err(ExternalError::into_lua_err)?;
                    Ok(result.into_lua(&lua))
                }
            })
            .map_err(|err| StError::Internal(Box::new(err)))?;
        Ok(Function::new(out))
    }

    pub fn invoke(&mut self, target: Function, args: Vec<Any>) -> Result<Any> {
        let mut argv = mlua::Variadic::<mlua::Value>::new();
        for arg in args.into_iter() {
            argv.push(
                arg.into_lua(&self.rt)
                    .map_err(|err| crate::vm::StError::Internal(Box::new(err)))?,
            );
        }
        let result = target
            .internal
            .call::<mlua::Value>(argv)
            .map_err(|err| StError::Internal(Box::new(err)))?;
        Ok(result.into_saturnus())
    }

    pub fn create_string(&mut self, value: impl AsRef<[u8]>) -> Str {
        Str::new(self.rt.create_string(value).unwrap())
    }

    pub fn load_program(&mut self, program: Box<dyn Ir>) -> Program<'_> {
        Program(self.rt.load(program.to_string()))
    }

    pub fn get_globals(&mut self) -> Table {
        let globals = self.rt.globals();
        Table { value: globals }
    }

    pub fn get_table(&self, table: &Table, key: impl IntoSaturnus) -> Any {
        let k = key.into_saturnus();
        let Some(k) = k.into_lua(&self.rt).ok() else {
            return Any::unit();
        };
        table.get(k).unwrap_or(Any::unit())
    }

    pub fn set_table(&self, table: &mut Table, key: impl IntoSaturnus, value: impl IntoSaturnus) {
        let k = key.into_saturnus();
        let v = value.into_saturnus();
        let k = k.into_lua(&self.rt).unwrap();
        let v = v.into_lua(&self.rt).unwrap();
        table.set(k, v);
    }
}
