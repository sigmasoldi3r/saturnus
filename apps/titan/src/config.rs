use serde::{
    Deserialize, Deserializer, Serialize,
    de::{MapAccess, Visitor},
};
use std::{collections::HashMap, fmt, marker::PhantomData, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "package")]
    pub package: Package,
    #[serde(default = "Default::default")]
    pub dependencies: HashMap<String, DependencyContainer>,
    #[serde(default = "Default::default")]
    /// Determines the project linking stage behaviour.
    pub linking: ProjectLinking,
    pub titan: Option<TitanOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLinking {
    pub no_std: bool,
    pub mode: LinkMode,
}
impl Default for ProjectLinking {
    fn default() -> Self {
        Self {
            no_std: false,
            mode: LinkMode::Collect,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkMode {
    /// Joins the output into a single file, ready to run.
    Collect,
    /// Joins the output into a single file and produces a self-contained binary.
    Binary,
    /// Tells titan to preserve the original structure of the project, may be used in some cases for interoperability.
    PreserveStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitanOverride {
    /// The repository mirror URL list, where it will fetch sequentially the packages.
    pub repositories: Vec<String>,
}

fn def_ver() -> String {
    "1.0.0".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    #[serde(default = "def_ver")]
    pub version: String,
    pub description: Option<String>,
    // Add other fields as necessary
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencyContainer {
    String(String),
    Object(Dependency),
}
impl DependencyContainer {
    pub fn unwrap(self) -> Dependency {
        match self {
            DependencyContainer::String(version) => Dependency::Version { version },
            DependencyContainer::Object(dependency) => dependency,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Version {
        version: String,
    },
    Git {
        git: String,
        #[serde(default = "Default::default")]
        version: String,
    },
    Path {
        path: String,
    },
}

#[derive(Debug)]
struct Void;
impl std::fmt::Display for Void {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("void")
    }
}
impl std::error::Error for Void {}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);
    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }
        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: serde::de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }
        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
