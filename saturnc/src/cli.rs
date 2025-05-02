use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CompileTarget {
    /// Default Lua target, 5.3
    Lua,
}
impl CompileTarget {
    pub fn ext(&self) -> String {
        match self {
            Self::Lua => "lua",
        }
        .into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ModSys {
    /// The default module resolution strategy.
    Saturnus,
    Native,
    Glboals,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub enum Args {
    Compile {
        /// The input file to compile.
        #[arg(long, short)]
        input: PathBuf,
        /// Only processes the macro code and expands it, currently disabled (ignored).
        #[arg(long)]
        only_macros: bool,
        /// The module resolution strategy to use.
        #[arg(long, default_value = "saturnus")]
        module_resolution: ModSys,
        /// Makes top-level static variables available as global variables.
        #[arg(long)]
        static_is_global: bool,
        /// Uses std library collections instead of naked ones.
        #[arg(long)]
        use_std_collections: bool,
        /// Specify the code backend used as a compilation result.
        #[arg(long, short, value_enum, default_value = "lua")]
        target: CompileTarget,
        /// Disables the platform-specific loop optimizations.
        #[arg(long)]
        disable_loop_interop: bool,
        /// Unit will be treated as it's own object, instead of translating to platform-null value.
        #[arg(long)]
        disable_unit_interop: bool,
        /// Output file target, if skipped, will match the input plus the output extension.
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// Output is ignored in favour of redirecting the output source code to the stdout.
        #[arg(long)]
        stdout: bool,
        /// Instead of inferring the module path from the input, it uses the given module path.
        #[arg(long, short)]
        mod_path: Option<PathBuf>,
        #[arg(long)]
        strip_core_types: bool,
    },
    Run {
        /// The input file to run with saturnus runtime.
        #[arg(long, short)]
        input: PathBuf,
        /// Dumps the compiled code to the console, used to debug runtime errors that may be caused by the compiler.
        #[arg(long)]
        dump_ir: bool,
    },
}
