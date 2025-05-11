use std::{
    collections::HashMap,
    error::Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

pub type DynErr = Box<dyn Error>;
pub trait ToDynErr<T> {
    fn to_dyn(self) -> Result<T, DynErr>;
}
impl<T, E: Error + Sized + 'static> ToDynErr<T> for Result<T, E> {
    fn to_dyn(self) -> Result<T, DynErr> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Box::new(err)),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TableSpace {
    tables: HashMap<String, Vec<toml::Value>>,
}
impl TableSpace {
    pub fn get_table(&mut self, name: impl Into<String>) -> &'_ mut Vec<toml::Value> {
        let name: String = name.into();
        if self.tables.get(&name).is_some() {
            return self.tables.get_mut(&name).unwrap();
        }
        self.tables.insert(name.clone(), vec![]);
        self.tables.get_mut(&name).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Timestamp {
    value: u64,
}
impl Timestamp {
    pub fn now() -> Self {
        let current = SystemTime::now();
        let value = current.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        Self { value }
    }
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis(self.value)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Db {
    created_at: Timestamp,
    modified_at: Timestamp,
    table_spaces: HashMap<String, TableSpace>,
}
impl Default for Db {
    fn default() -> Self {
        Self {
            modified_at: Timestamp::now(),
            created_at: Timestamp::now(),
            table_spaces: Default::default(),
        }
    }
}
impl Db {
    pub const DB_FILE: &'static str = ".build_db";
    pub fn load() -> Result<Self, DynErr> {
        if !std::fs::exists(Self::DB_FILE).to_dyn()? {
            let mut db = Db::default();
            db.save()?;
            return Ok(db);
        }
        let raw = std::fs::read_to_string(Self::DB_FILE).to_dyn()?;
        let db = toml::from_str(raw.as_str()).to_dyn()?;
        Ok(db)
    }
    pub fn save(&mut self) -> Result<(), DynErr> {
        self.modified_at = Timestamp::now();
        let raw = toml::to_string_pretty(self).to_dyn()?;
        std::fs::write(Self::DB_FILE, raw).to_dyn()?;
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.table_spaces.is_empty()
    }
    pub fn get_table_space(&mut self, name: impl Into<String>) -> &'_ mut TableSpace {
        let name: String = name.into();
        if self.table_spaces.get(&name).is_some() {
            return self.table_spaces.get_mut(&name).unwrap();
        }
        let empty = TableSpace {
            tables: Default::default(),
        };
        self.table_spaces.insert(name.clone(), empty);
        self.table_spaces.get_mut(&name).unwrap()
    }
}
