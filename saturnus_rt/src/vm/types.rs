use mlua::IntoLua;
use st_macros::{IntoSaturnus, wrapper_enum};

#[derive(Debug, Clone, IntoSaturnus)]
pub struct Table {
    pub(super) value: mlua::Table,
}
impl Table {
    pub(super) fn new(value: mlua::Table) -> Self {
        Self { value }
    }
    pub(crate) fn set(&mut self, k: impl IntoLua, v: impl IntoLua) {
        self.value.set(k, v).unwrap()
    }
    pub(crate) fn get(&self, k: impl IntoLua) -> Option<Any> {
        self.value
            .get::<mlua::Value>(k)
            .ok()
            .map(|val| val.into_saturnus())
    }
}

#[macro_export]
macro_rules! table_get {
    ( $vm:expr; $tbl:expr , $k:expr ) => {{
        let key = $vm.lock().create_string($k);
        $vm.lock().get_table(&$tbl, key)
    }};
}

#[macro_export]
macro_rules! table_set {
    ( $vm:expr; $tbl:expr , $k:expr => $v:expr ) => {{
        let key = $vm.lock().create_string($k);
        let value = $v;
        $vm.lock().set_table(&mut $tbl, key, value)
    }};
}

#[macro_export]
macro_rules! table {
    ( $vm:expr; $( $k:expr => $v:expr ),* $(,)? ) => { {
        let mut tbl = $vm.create_table().unwrap();
        $( tbl.set($k, $v); )+
        tbl
    } };
}

#[derive(Debug, Clone, IntoSaturnus)]
pub struct Str {
    pub(super) value: mlua::String,
}
impl Str {
    pub(super) fn new(value: mlua::String) -> Self {
        Self { value }
    }
    pub fn to_string(&self) -> String {
        self.value.to_string_lossy()
    }
}

#[derive(Debug, Clone, IntoSaturnus)]
pub struct Function {
    pub(super) internal: mlua::Function,
}
impl Function {
    pub(super) fn new(internal: mlua::Function) -> Self {
        Self { internal }
    }
}

#[derive(Debug, Clone, IntoSaturnus)]
pub struct Coroutine {
    pub(super) internal: mlua::Thread,
}
impl Coroutine {
    pub(super) fn new(co: mlua::Thread) -> Self {
        Self { internal: co }
    }
}

#[derive(Debug, Clone)]
pub enum Other {
    LightUserData(mlua::LightUserData),
    UserData(mlua::AnyUserData),
    Pointer(mlua::Value),
    Error(Box<mlua::Error>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSaturnus)]
pub struct Unit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSaturnus)]
pub struct Nothing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSaturnus)]
pub struct Never;

#[wrapper_enum]
#[derive(Debug, Clone)]
pub enum Any {
    Table,
    Function,
    Coroutine,
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Str,
    Unit,
    Never,
    Nothing,
    Other,
}
impl Any {
    pub fn unit() -> Self {
        Self::Unit(Unit)
    }
    pub fn or(self, other: impl IntoAny) -> Self {
        match self {
            Self::Unit(_) => other.into_any(),
            value => value,
        }
    }
}
impl IntoSaturnus for Any {
    fn into_saturnus(self) -> Any {
        self
    }
}

pub trait IntoSaturnus {
    fn into_saturnus(self) -> Any;
}

pub(crate) mod conversion {
    use mlua::IntoLua;

    use super::{
        Any, Coroutine, Function, IntoAny, IntoSaturnus, Never, Nothing, Other, Str, Table, Unit,
    };

    macro_rules! def_into_lua {
        ($($t:ident => |$s:ident| $v:expr),*) => {
            $(
                impl IntoLua for $t {
                    fn into_lua($s, _: &mlua::Lua) -> mlua::Result<mlua::Value> {
                        Ok($v)
                    }
                }
            )*
        };
    }

    def_into_lua! {
        Table => |self| mlua::Value::Table(self.value),
        Function => |self| mlua::Value::Function(self.internal),
        Coroutine => |self| mlua::Value::Thread(self.internal),
        Str => |self| mlua::Value::String(self.value),
        Unit => |self| mlua::Value::Nil,
        Never => |self| mlua::Value::Nil,
        Nothing => |self| mlua::Value::Nil,
        Other => |self| match self {
            Other::LightUserData(light_user_data) => mlua::Value::LightUserData(light_user_data),
            Other::UserData(any_user_data) => mlua::Value::UserData(any_user_data),
            Other::Pointer(value) => value,
            Other::Error(error) => mlua::Value::Error(error),
        }
    }

    impl IntoLua for Any {
        fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
            match self {
                Any::Table(table) => table.into_lua(lua),
                Any::Function(function) => function.into_lua(lua),
                Any::Coroutine(coroutine) => coroutine.into_lua(lua),
                Any::Integer(val) => val.into_lua(lua),
                Any::Decimal(val) => val.into_lua(lua),
                Any::Boolean(val) => val.into_lua(lua),
                Any::Str(val) => val.into_lua(lua),
                Any::Unit(unit) => unit.into_lua(lua),
                Any::Never(never) => never.into_lua(lua),
                Any::Nothing(nothing) => nothing.into_lua(lua),
                Any::Other(other) => other.into_lua(lua),
            }
        }
    }

    macro_rules! def_into_saturnus {
        ($($t:path => |$s:ident| $v:expr),*) => {
            $(
                impl IntoSaturnus for $t {
                    fn into_saturnus($s) -> Any {
                        $v
                    }
                }
            )*
        };
    }

    def_into_saturnus! {
        mlua::Table => |self| Table::new(self).into_any(),
        mlua::String => |self| Str::new(self).into_any(),
        mlua::Function => |self| Function::new(self).into_any(),
        mlua::Thread => |self| Coroutine::new(self).into_any()
    }

    impl IntoSaturnus for mlua::Value {
        fn into_saturnus(self) -> super::Any {
            match self {
                mlua::Value::Nil => super::Unit.into_any(),
                mlua::Value::Table(table) => table.into_saturnus(),
                mlua::Value::Boolean(val) => val.into_any(),
                mlua::Value::Integer(val) => val.into_any(),
                mlua::Value::Number(val) => val.into_any(),
                mlua::Value::String(val) => val.into_saturnus(),
                mlua::Value::Function(function) => function.into_saturnus(),
                mlua::Value::Thread(thread) => thread.into_saturnus(),
                mlua::Value::LightUserData(light_user_data) => {
                    Other::LightUserData(light_user_data).into_any()
                }
                mlua::Value::UserData(any_user_data) => Other::UserData(any_user_data).into_any(),
                mlua::Value::Error(error) => Other::Error(error).into_any(),
                val => Other::Pointer(val).into_any(),
            }
        }
    }
}
