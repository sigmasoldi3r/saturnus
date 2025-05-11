pub mod internal;
pub mod program;
pub mod types;

use std::{error::Error, fmt::Display, ops::Deref, sync::Arc};

use lazy_static::lazy_static;
use mlua::{ExternalError, IntoLua as _, Lua, MaybeSend};
use program::Program;
use ststd::Ir;
use tokio::{sync::Mutex, task::JoinHandle};
use types::{Any, Function, IntoSaturnus, Str, Table};

use crate::mem::*;

#[derive(Debug)]
pub enum StError {
    TypeError { message: String },
    Internal(Box<dyn Error>),
}
impl StError {
    pub fn internal(err: impl Error + 'static) -> Self {
        Self::Internal(Box::new(err))
    }
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }
}
unsafe impl Send for StError {}
unsafe impl Sync for StError {}
impl Error for StError {}
impl Display for StError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(error) => (error.deref() as &dyn Display).fmt(f),
            Self::TypeError { message } => format!("Type error: {message}").fmt(f),
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

struct TaskQueue {
    handles: Mutex<Vec<JoinHandle<()>>>,
}
impl TaskQueue {
    fn new() -> Self {
        Self {
            handles: Default::default(),
        }
    }
    async fn push_task(&self, handle: JoinHandle<()>) {
        self.handles.lock().await.push(handle);
    }
    async fn pending(&self) {
        let handles = std::mem::take(&mut *self.handles.lock().await);
        for handle in handles {
            handle.await.unwrap();
        }
    }
}

lazy_static! {
    static ref TASK_QUEUE: Arc<TaskQueue> = Arc::new(TaskQueue::new());
}

/// # Saturnus Virtual Machine
///
/// This is the backend that runs saturnus.
///
/// Hosts creation and management of the native runtime.
#[deprecated(note = "Moving to thin-wrappers. See `runtime::internal::Vm;`")]
#[derive(Clone)]
pub struct StVm {
    rt: Lua,
}
impl StVm {
    pub fn new() -> Self {
        Self::from_rt(Lua::new())
    }
    fn from_rt(rt: Lua) -> Self {
        Self { rt }
    }

    pub async fn process_pending(&self) {
        TASK_QUEUE.pending().await;
    }

    pub async fn spawn(&self, block: impl Future<Output = ()> + Send + 'static) {
        TASK_QUEUE.push_task(tokio::spawn(block)).await;
    }

    pub fn create_table(&self) -> Result<Table> {
        Ok(Table {
            value: self.rt.create_table().wrap()?,
        })
    }

    pub fn create_fn<F, FR>(&self, func: F) -> Result<Function>
    where
        F: FnMut(StVm, Vec<Any>) -> FR + 'static,
        F: MaybeSend,
        FR: Future<Output = Result<Any>> + Send,
    {
        let func = func.into_rc();
        let out = self
            .rt
            .create_async_function(move |lua: Lua, args: mlua::Variadic<mlua::Value>| {
                let func = func.clone();
                async move {
                    let func = func.clone();
                    let vm = Self::from_rt(lua.clone());
                    let argv: Vec<Any> =
                        args.into_iter().map(IntoSaturnus::into_saturnus).collect();
                    let result = {
                        let mut func = func.lock().await;
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

    pub fn invoke(&self, target: Function, args: Vec<Any>) -> Result<Any> {
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

    pub fn create_string(&self, value: impl AsRef<[u8]>) -> Str {
        Str::new(self.rt.create_string(value).unwrap())
    }

    pub fn load_program(&self, program: Box<dyn Ir>) -> Program<'_> {
        Program(self.rt.load(program.to_string()))
    }

    pub fn get_globals(&self) -> Table {
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
