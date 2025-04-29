use std::path::PathBuf;

pub mod backends;

#[derive(Debug, Clone)]
pub enum CompilerError {
    SyntaxError(String),
    MacroError,
    SystemError,
    ParsingError(String),
}

pub struct CompilerSource {
    pub source: String,
    pub location: Option<PathBuf>,
}
impl CompilerSource {
    pub fn without_shebang(&self) -> String {
        let source = self.source.clone();
        if source.starts_with("#") {
            return source.split("\n").skip(1).collect::<Vec<_>>().join("\n");
        }
        source
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ModuleType {
    /// Default module mode, uses custom module namespacing.
    #[default]
    Saturnus,
    /// Top-level use is disabled, and everything is exported as a global.
    PubAsGlobal,
    /// Require-first modules.
    LocalModuleReturn,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub use_std_collections: bool,
    pub skip_loop_interop: bool,
    pub unit_interop: bool,
    pub module_type: ModuleType,
    pub override_mod_path: Option<PathBuf>,
}
impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            use_std_collections: Default::default(),
            skip_loop_interop: Default::default(),
            unit_interop: true,
            module_type: Default::default(),
            override_mod_path: None,
        }
    }
}

pub trait Compiler {
    fn compile(
        &mut self,
        source: CompilerSource,
        options: CompilerOptions,
    ) -> std::result::Result<String, CompilerError>;
}

pub type Result = std::result::Result<(), CompilerError>;
