use std::{
    fmt::Display,
    hash::{BuildHasher, Hash, Hasher},
    time::SystemTime,
};

use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

/// # Lock database
///
/// Used to keep track of the packages installed.
#[derive(Debug)]
pub enum LockDbError {
    Io(std::io::Error),
    Read(ron::error::SpannedError),
    Write(ron::Error),
}
impl Display for LockDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockDbError::Io(error) => write!(f, "Failed to perform I/O on the database: {error}"),
            LockDbError::Read(spanned_error) => {
                write!(f, "The database file seems corrupted: {spanned_error}")
            }
            LockDbError::Write(error) => write!(f, "Could not encode the database: {error}"),
        }
    }
}
impl std::error::Error for LockDbError {}

fn now() -> usize {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as usize
}

pub struct LockDb {
    data: internal::LockDatabase,
}
impl LockDb {
    pub const DB_PATH: &'static str = "titan.lock";
    pub const DB_VERSION: usize = 1usize;

    /// # Load the database
    ///
    /// Reads the data base (if exists), and loads it in memory.
    /// If it does not exist, it creates it and makes it ready to write.
    pub fn load() -> Result<Self, LockDbError> {
        if std::fs::exists(Self::DB_PATH).map_err(LockDbError::Io)? {
            let raw = std::fs::read_to_string(Self::DB_PATH).map_err(LockDbError::Io)?;
            let raw = raw.as_str();
            let out = ron::from_str(raw).map_err(LockDbError::Read)?;
            Ok(Self { data: out })
        } else {
            let mut db = Self {
                data: Default::default(),
            };
            db.save()?;
            Ok(db)
        }
    }

    /// # Synchronize the database
    ///
    /// Stores the memory information onto the disk to later use.
    pub fn save(&mut self) -> Result<(), LockDbError> {
        self.data.last_modified = now();
        let config = PrettyConfig::new().struct_names(true);
        let raw = ron::ser::to_string_pretty(&self.data, config).map_err(LockDbError::Write)?;
        std::fs::write(Self::DB_PATH, raw).map_err(LockDbError::Io)?;
        Ok(())
    }

    /// Saves the record, does not store the database!
    pub fn store(&mut self, record: &Package) -> PackageId {
        let id = PackageId::from_package(record);
        self.data.entries.insert(id.0.clone(), record.clone());
        id
    }

    /// Tries to get the package.
    pub fn get_package(&self, id: &PackageId) -> Option<Package> {
        self.data.entries.get(&id.0).cloned()
    }

    /// Looks up the package in the database.
    pub fn contains(&self, pkg: &Package) -> bool {
        let id = PackageId::from_package(pkg);
        self.data.entries.contains_key(&id.0)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Package {
    pub url: String,
    pub version: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageId(String);
impl PackageId {
    pub fn from_package(package: &Package) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        package.url.hash(&mut hasher);
        let url_hash = hasher.finish();
        let mut hasher = std::hash::DefaultHasher::new();
        package.version.hash(&mut hasher);
        let version_hash = hasher.finish();
        Self(format!("{url_hash:#x}-{version_hash:#x}"))
    }
}
impl ToString for PackageId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

mod internal {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use super::{LockDb, Package, now};

    #[derive(Serialize, Deserialize)]
    pub struct LockDatabase {
        pub version: usize,
        pub last_modified: usize,
        pub created_on: usize,
        pub entries: HashMap<String, Package>,
    }
    impl Default for LockDatabase {
        fn default() -> Self {
            Self {
                version: LockDb::DB_VERSION,
                last_modified: now(),
                created_on: now(),
                entries: Default::default(),
            }
        }
    }
}
