use compiler::CompilerOptions;
use macros::generate_bindings;
use mlua::{FromLua, IntoLua};
use source::{SaturnusIR, SourceCode};

mod backends;
pub mod code;
pub mod compiler;
pub mod parsing;
pub mod processing;
pub mod source;
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
pub struct Saturnus {
    pub options: CompilerOptions,
}
impl Saturnus {
    pub fn create_table(&self) -> Result<Table> {
        Ok(Table(
            self.runtime
                .create_table()
                .wrap_err("Failed to create a table")?,
        ))
    }
    pub fn compile(&self, source: impl SourceCode) -> Result<SaturnusIR> {
        use compiler::Compiler as _;
        let mut cp = backends::LuaCompiler::new();
        cp.compile(source, self.options.clone())
            .map_err(|err| RuntimeError {
                caused_by: Some(Box::new(err)),
                message: "Compilation failed!".into(),
            })
    }
    pub fn load<'a>(&self, source: impl SourceCode) -> Result<Program<'a>> {
        let ir = self.compile(source)?;
        self.load_ir(ir)
    }
    pub fn load_ir<'a>(&self, ir: SaturnusIR) -> Result<Program<'a>> {
        Ok(Program(self.runtime.load(ir)))
    }
    pub fn globals(&self) -> Table {
        Table(self.runtime.globals())
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

macro_rules! integers {
    ( $( $kind:ty ),* ) => {
        $(
            impl Into<$kind> for Value {
                fn into(self) -> $kind {
                    match self.0 {
                        mlua::Value::Integer(val) => val as $kind,
                        _ => panic!("Invalid value contained!")
                    }
                }
            }
        )*
    };
}
integers!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl Into<Table> for Value {
    fn into(self) -> Table {
        match self.0 {
            mlua::Value::Table(val) => Table(val),
            _ => panic!("Invalid value contained!"),
        }
    }
}

macro_rules! impl_into_lua {
    ( for $id:ty ) => {
        impl mlua::IntoLua for $id {
            fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
                self.0.into_lua(lua)
            }
        }
    };
}

pub trait IntoSaturnus: mlua::IntoLua {}
impl<T: mlua::IntoLua> IntoSaturnus for T {}

pub type Integer = mlua::Integer;

pub struct Table(mlua::Table);
impl_into_lua! { for Table }
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

#[cfg(test)]
mod test {
    use crate::Saturnus;

    #[test]
    fn simple_hello_world() {
        let sat = Saturnus::new();
        let g = sat.globals();
        g.set("the_wild_ones", 5).unwrap();
        let prog = sat
            .load(
                r#"
            // A fair simple hello world:
            let hya = 1;
            let hey = 1 + the_wild_ones;
            return hey;"#,
            )
            .unwrap();
        let out = prog.eval().unwrap();
        assert!(out.is_integer());
        let i: i32 = out.into();
        assert_eq!(i, 6i32);
    }
}
