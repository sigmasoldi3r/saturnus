use std::path::PathBuf;

use crate::source::{SaturnusIR, SourceCode};

#[derive(Debug, Clone)]
pub enum CompilerError {
    SyntaxError(String),
    MacroError,
    SystemError,
    ParsingError(String),
}
impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::SyntaxError(cause) => write!(f, "Syntax error: {cause}"),
            CompilerError::MacroError => write!(f, "Macro expansion error: <not available>"),
            CompilerError::SystemError => write!(f, "System error: <not available>"),
            CompilerError::ParsingError(cause) => write!(f, "Parsing error: {cause}"),
        }
    }
}
impl std::error::Error for CompilerError {}

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
        source: impl SourceCode,
        options: CompilerOptions,
    ) -> std::result::Result<SaturnusIR, CompilerError>;
}

pub type Result = std::result::Result<(), CompilerError>;
