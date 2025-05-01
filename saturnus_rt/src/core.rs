use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Arc};

use serde::{Serialize, ser::SerializeMap};

pub trait IntoSaturnus {
    fn into_saturnus(self) -> Any;
}

impl IntoSaturnus for std::string::String {
    fn into_saturnus(self) -> Any {
        Any::String(self)
    }
}
impl IntoSaturnus for &str {
    fn into_saturnus(self) -> Any {
        self.to_owned().into_saturnus()
    }
}
impl IntoSaturnus for f64 {
    fn into_saturnus(self) -> Any {
        Any::Decimal(Decimal::new(self))
    }
}
impl IntoSaturnus for i64 {
    fn into_saturnus(self) -> Any {
        Any::Integer(self)
    }
}
impl IntoSaturnus for bool {
    fn into_saturnus(self) -> Any {
        Any::Boolean(self)
    }
}
impl IntoSaturnus for Any {
    fn into_saturnus(self) -> Any {
        self
    }
}
impl IntoSaturnus for &Any {
    fn into_saturnus(self) -> Any {
        self.clone()
    }
}
impl IntoSaturnus for Table {
    fn into_saturnus(self) -> Any {
        Any::Object(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    data: HashMap<Any, Any>,
}
impl IntoIterator for Table {
    type Item = (Any, Any);
    type IntoIter = std::collections::hash_map::IntoIter<Any, Any>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
impl Table {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
        }
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Any, Any> {
        self.data.iter()
    }
    pub fn set(&mut self, key: impl IntoSaturnus, value: impl IntoSaturnus) {
        self.data.insert(key.into_saturnus(), value.into_saturnus());
    }
    pub fn get(&self, key: impl IntoSaturnus) -> Option<Any> {
        self.data.get(&key.into_saturnus()).cloned()
    }
}
impl Hash for Table {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (k, v) in self.data.iter() {
            k.hash(state);
            v.hash(state);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decimal(f64);
impl Decimal {
    pub fn new(value: f64) -> Self {
        Self(value)
    }
    pub fn into_inner(self) -> f64 {
        self.0
    }
}
impl Hash for Decimal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_be_bytes().hash(state);
    }
}
impl Eq for Decimal {}
impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

type FnDef = dyn Fn(Vec<Any>) -> Any + Send + Sync;

#[derive(Clone)]
pub struct Callable(Arc<FnDef>);
impl Callable {
    pub fn new(func: impl Fn(Vec<Any>) -> Any + 'static + Send + Sync) -> Self {
        Self(Arc::new(func))
    }
    pub fn into_inner(self) -> Arc<FnDef> {
        self.0
    }
}
impl Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn({:?})", &*self.0 as *const FnDef)
    }
}
impl Hash for Callable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = &*self.0 as *const FnDef;
        ptr.hash(state);
    }
}
impl Eq for Callable {}
impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        let left = &*self.0 as *const FnDef;
        let right = &*other.0 as *const FnDef;
        std::ptr::addr_eq(left, right)
    }
}
impl IntoSaturnus for Callable {
    fn into_saturnus(self) -> Any {
        Any::Function(self)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Any {
    Integer(i64),
    Decimal(Decimal),
    Boolean(bool),
    String(String),
    Object(Table),
    Function(Callable),
    Unit,
    Future(Box<impl Future<Any>>),
}

unsafe impl Send for Any {}
unsafe impl Sync for Any {}

impl Serialize for Any {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Any::Integer(val) => serializer.serialize_i64(*val),
            Any::Decimal(decimal) => serializer.serialize_f64(decimal.0),
            Any::Boolean(val) => serializer.serialize_bool(*val),
            Any::String(val) => serializer.serialize_str(val.as_str()),
            Any::Object(table) => table.serialize(serializer),
            Any::Function(_) => unimplemented!(),
            Any::Unit => serializer.serialize_unit(),
            _ => unimplemented!(),
        }
    }
}

impl Serialize for Table {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let map = serializer.serialize_map(Some(self.data.len()));
        match map {
            Ok(mut sm) => {
                for (k, v) in self.iter() {
                    if let Err(err) = sm.serialize_key(k) {
                        return Err(err);
                    }
                    if let Err(err) = sm.serialize_value(v) {
                        return Err(err);
                    }
                }
                sm.end()
            }
            Err(err) => Err(err),
        }
    }
}

#[macro_export]
macro_rules! table {
    ( $( $key:expr => $value:expr ),* $(,)* ) => {
        {
            let mut table = Table::new();
            $(
                table.set($key, $value);
            )*
            table
        }
    };
}
