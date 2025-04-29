use crate::{
    core::{Any, Callable, Table},
    table,
};

/// The main source code of the Saturnus Standard Library.
///
/// Here are included the cross-platform declarations.
///
/// Native implementations are included in other modules of the
/// Saturnus standard library, depending on the target.
///
/// (For example, the runtime comes with builtin native functions).
pub const SOURCE: &'static str = include_str!("./main.st");

// Below are the stdlib extensions only available to the runtime.

pub trait ArgsHelper {
    fn get_arg(&self, nth: usize) -> Any;
}
impl ArgsHelper for Vec<Any> {
    fn get_arg(&self, nth: usize) -> Any {
        self.get(nth).cloned().unwrap_or(Any::Unit)
    }
}

pub enum Number {
    Int(i64),
    Dec(f64),
}
pub trait AnyAssertions {
    fn assert_int(self) -> Option<i64>;
    fn assert_optional_int(self) -> Option<Any>;
    fn assert_decimal(self) -> Option<f64>;
    fn assert_optional_decimal(self) -> Option<Any>;
    fn assert_number(self) -> Option<Number>;
    fn assert_optional_number(self) -> Option<Any>;
    fn assert_bool(self) -> Option<bool>;
    fn assert_optional_bool(self) -> Option<Any>;
    fn assert_string(self) -> Option<String>;
    fn assert_optional_string(self) -> Option<Any>;
    fn assert_table(self) -> Option<Table>;
    fn assert_optional_table(self) -> Option<Any>;
}
impl AnyAssertions for Any {
    fn assert_int(self) -> Option<i64> {
        match self {
            Any::Integer(val) => Some(val),
            _ => None,
        }
    }
    fn assert_optional_int(self) -> Option<Any> {
        match self {
            Any::Integer(val) => Some(Any::Integer(val)),
            Any::Unit => Some(Any::Unit),
            _ => None,
        }
    }
    fn assert_decimal(self) -> Option<f64> {
        match self {
            Any::Decimal(val) => Some(val.into_inner()),
            _ => None,
        }
    }
    fn assert_optional_decimal(self) -> Option<Any> {
        match self {
            Any::Decimal(val) => Some(Any::Decimal(val)),
            Any::Unit => Some(Any::Unit),
            _ => None,
        }
    }
    fn assert_number(self) -> Option<Number> {
        match self {
            Any::Integer(val) => Some(Number::Int(val)),
            Any::Decimal(val) => Some(Number::Dec(val.into_inner())),
            _ => None,
        }
    }
    fn assert_optional_number(self) -> Option<Any> {
        match self {
            Any::Integer(val) => Some(Any::Integer(val)),
            Any::Decimal(val) => Some(Any::Decimal(val)),
            Any::Unit => Some(Any::Unit),
            _ => None,
        }
    }
    fn assert_bool(self) -> Option<bool> {
        match self {
            Any::Boolean(val) => Some(val),
            _ => None,
        }
    }
    fn assert_optional_bool(self) -> Option<Any> {
        match self {
            Any::Boolean(val) => Some(Any::Boolean(val)),
            Any::Unit => Some(Any::Unit),
            _ => None,
        }
    }
    fn assert_string(self) -> Option<String> {
        match self {
            Any::String(val) => Some(val),
            _ => None,
        }
    }
    fn assert_optional_string(self) -> Option<Any> {
        match self {
            Any::String(val) => Some(Any::String(val)),
            Any::Unit => Some(Any::Unit),
            _ => None,
        }
    }
    fn assert_table(self) -> Option<Table> {
        match self {
            Any::Object(val) => Some(val),
            _ => None,
        }
    }
    fn assert_optional_table(self) -> Option<Any> {
        match self {
            Any::Object(val) => Some(Any::Object(val)),
            Any::Unit => Some(Any::Unit),
            _ => None,
        }
    }
}

pub trait AnyCasts {
    fn as_table_or(self, value: Table) -> Table;
}
impl AnyCasts for Any {
    fn as_table_or(self, value: Table) -> Table {
        match self {
            Any::Object(tbl) => tbl,
            _ => value,
        }
    }
}

pub struct MiniJinjaCompilerFn;
impl MiniJinjaCompilerFn {
    pub fn new() -> Callable {
        Callable::new(|args| {
            let Any::String(tpl) = args.get_arg(0) else {
                eprintln!("First argument must be a string.");
                return Any::Unit;
            };
            let Some(tbl) = args.get_arg(1).assert_optional_table() else {
                eprintln!("Second argument must be a table, or nothing.");
                return Any::Unit;
            };
            let mut env = minijinja::Environment::new();
            env.add_template("template", tpl.as_str()).unwrap();
            let tpl = env.get_template("template").unwrap();
            let ctx = minijinja::Value::from_serialize(&tbl);
            return Any::String(tpl.render(ctx).unwrap());
        })
    }
}

pub struct Net;
impl Net {
    pub fn raw_get() -> Callable {
        use std::io::Read;
        Callable::new(|args| {
            let Any::String(url) = args.get_arg(0) else {
                eprintln!("First argument must be a string!");
                return Any::Unit;
            };
            let mut body = String::new();
            match reqwest::blocking::get(url.clone()) {
                Ok(mut res) => match res.read_to_string(&mut body) {
                    Ok(body_len) => Any::Object(table! {
                        "body" => body,
                        "body_len" => body_len as i64,
                        "status" => res.status().to_string(),

                    }),
                    Err(err) => {
                        eprintln!("Error while reading {url:?}: {err}");
                        Any::Unit
                    }
                },
                Err(err) => {
                    eprintln!("Could not get {url:?}: {err}");
                    Any::Unit
                }
            }
        })
    }
}

pub fn init_native_modules() -> Table {
    table! {
        "__modules__" => table!{
            "std" => table! {
                "template" => MiniJinjaCompilerFn::new(),
                "net" => table!{
                    "get" => Net::raw_get()
                }
            }
        }
    }
}
