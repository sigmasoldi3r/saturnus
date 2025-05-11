use macros::generate_bindings;
use mlua::{FromLua, IntoLua};

#[cfg(test)]
mod test;

pub mod code;
pub mod compiling;
pub mod parsing;
pub mod processing;
pub mod typecheck;

use core::ffi::c_int;

macro_rules! wrap_fields {
    ( $parent:ty where $( $name:ident ( $( $params:ident ),* ) -> $out:ty ; )* )=> {
        impl $parent {
            $(
                pub fn $name(&self, $( $params: impl IntoSaturnus ),*) -> $out {
                    match self.0.$name($( $params ),*) {
                        Ok(val) => Ok(val),
                        Err(err) => Err(RuntimeError {
                            caused_by: Some(Box::new(err)),
                            message: (concat!("Error while calling ", stringify!($parent), "::", stringify!($name), "(..)")).into(),
                        }),
                    }
                }
            )*
        }
        impl std::fmt::Debug for $parent {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

pub struct Program<'a>(mlua::Chunk<'a>);
impl<'a> Program<'a> {
    pub fn exec(self) -> Result<()> {
        self.0
            .exec()
            .wrap_err("Failed to execute this program chunk")
    }
    pub fn eval(self) -> Result<Value> {
        self.0
            .eval::<mlua::Value>()
            .wrap_err("Failed to eval this program chunk")
            .map(Value)
    }
}

/// # Saturnus runtime
///
/// Compiles and executes saturnus code.
///
/// You can use it as a runtime by evaluating the code, or as a compiler,
/// using the `Saturnus::compile(..)` method.
#[generate_bindings(target = "Lua")]
pub struct Saturnus {}
impl Saturnus {
    pub fn create_table(&self) -> Result<Table> {
        Ok(Table(
            self.runtime
                .create_table()
                .wrap_err("Failed to create a table")?,
        ))
    }
    pub fn load<'a>(&self, chunk: impl mlua::AsChunk<'a>) -> Program<'a> {
        todo!("This must use it's own type of chunk! Sources must be compiled before loading!!");
        Program(self.runtime.load(chunk))
    }
}

trait WrapRtError<T> {
    fn wrap_err(self, message: impl Into<String>) -> Result<T>;
}
impl<T> WrapRtError<T> for mlua::Result<T> {
    fn wrap_err(self, message: impl Into<String>) -> Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(RuntimeError {
                caused_by: Some(Box::new(err)),
                message: message.into(),
            }),
        }
    }
}

macro_rules! is_delegate {
    ( $( $from:ident -> $to:ident ),* ) => {
        $(
            pub fn $to(&self) -> bool {
                self.0.$from()
            }
        )*
    };
}

pub struct Value(mlua::Value);
impl Value {
    pub const UNIT: Self = Self(mlua::Value::NULL);
    pub const NOTHING: Self = Self(mlua::Value::Nil);
    is_delegate! {
        is_null -> is_unit,
        is_nil -> is_nothing,
        is_boolean -> is_boolean,
        is_error -> is_error,
        is_function -> is_function,
        is_integer -> is_integer,
        is_light_userdata -> is_pointer,
        is_string -> is_string,
        is_thread -> is_suspend,
        is_userdata -> is_custom_type
    }
}
impl Eq for Value {}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
wrap_fields! { Value where }
impl FromLua for Value {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        Ok(Self(value))
    }
}
impl IntoLua for Value {
    fn into_lua(self, _: &mlua::Lua) -> mlua::Result<mlua::Value> {
        Ok(self.0)
    }
}
pub trait IntoSaturnus: mlua::IntoLua {}
impl<T: mlua::IntoLua> IntoSaturnus for T {}

pub type Integer = mlua::Integer;

pub struct Table(mlua::Table);
wrap_fields! {
    Table where
    set(key, value) -> Result<()>;
    get(key) -> Result<Value>;
    contains_key(key) -> Result<bool>;
    push(value) -> Result<()>;
    pop() -> Result<Value>;
    raw_set(key, value) -> Result<()>;
    raw_get(key) -> Result<Value>;
    raw_push(value) -> Result<()>;
    raw_pop() -> Result<Value>;
    raw_remove(key) -> Result<()>;
    clear() -> Result<()>;
    len() -> Result<Integer>;
}
impl Table {
    fn into_inner(self) -> mlua::Table {
        self.0
    }
    pub fn raw_insert(&self, idx: Integer, value: impl IntoSaturnus) -> Result<()> {
        self.0
            .raw_insert(idx, value)
            .wrap_err("Error while attempting to Table::raw_insert(..)")
    }
    pub fn equals(&self, other: &Table) -> Result<bool> {
        self.0
            .equals(&other.0)
            .wrap_err("Failed to compare two tables")
    }
    pub fn raw_len(&self) -> usize {
        self.0.raw_len()
    }
    pub fn metatable(&self) -> Option<Table> {
        self.0.metatable().map(Self)
    }
    pub fn set_metatable(&self, metatable: Option<Table>) {
        self.0.set_metatable(metatable.map(Self::into_inner));
    }
}
