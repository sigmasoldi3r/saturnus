use saturnus::compiler::{CompilerOptions, ModuleType};

use crate::cli::Args;

pub struct OptionsAdapter;

impl OptionsAdapter {
    pub fn new() -> Self {
        Self
    }
    pub fn parse_args(&self, args: &Args) -> CompilerOptions {
        match args {
            Args::Compile {
                only_macros,
                module_resolution,
                static_is_global,
                use_std_collections,
                disable_loop_interop,
                disable_unit_interop,
                mod_path,
                ..
            } => CompilerOptions {
                use_std_collections: *use_std_collections,
                skip_loop_interop: *disable_loop_interop,
                unit_interop: !*disable_unit_interop,
                override_mod_path: mod_path.clone(),
                module_type: match module_resolution {
                    crate::cli::ModSys::Saturnus => ModuleType::Saturnus,
                    crate::cli::ModSys::Native => ModuleType::LocalModuleReturn,
                    crate::cli::ModSys::Glboals => ModuleType::PubAsGlobal,
                },
            },
            Args::Run { .. } => Default::default(),
            Args::StdOutput { .. } => Default::default(),
        }
    }
}
