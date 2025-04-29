use std::path::PathBuf;

use console::style;

pub fn get_output_folder(output: Option<PathBuf>) -> PathBuf {
    if let Some(output) = output {
        output
    } else {
        println!(
            "{}",
            style("Dist folder not specified, will default to dist")
                .color256(8_u8)
                .italic()
                .dim()
        );
        "dist".into()
    }
}

pub fn get_source_folder(source: Option<PathBuf>) -> PathBuf {
    if let Some(source) = source {
        source
    } else {
        println!(
            "{}",
            style("Source folder not specified, will default to src")
                .color256(8_u8)
                .italic()
                .dim()
        );
        "src".into()
    }
}
