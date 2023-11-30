use std::path::PathBuf;

use console::style;

use crate::janusfile::JanusBuild;

fn get_output_folder(output: Option<PathBuf>) -> PathBuf {
    const TAB: &'static str = "     ";
    if let Some(output) = output {
        println!(
            "{} {}",
            TAB,
            style("Dist folder not specified, will default to dist.")
                .color256(8_u8)
                .italic()
                .dim()
        );
        output
    } else {
        "dist".into()
    }
}

pub enum CompilationError {}

pub type Result = std::result::Result<(), CompilationError>;

pub struct LibraryCompiler {}
impl LibraryCompiler {
    pub fn new() -> LibraryCompiler {
        LibraryCompiler {}
    }
    pub fn compile(self, info: JanusBuild) -> Result {
        let JanusBuild { output, source } = info;
        let output = get_output_folder(output);
        Ok(())
    }
}

pub struct BinaryCompiler {}
impl BinaryCompiler {
    pub fn new() -> BinaryCompiler {
        BinaryCompiler {}
    }
    pub fn compile(self, info: JanusBuild) -> Result {
        let JanusBuild { output, source } = info;
        let output = get_output_folder(output);
        Ok(())
    }
}
