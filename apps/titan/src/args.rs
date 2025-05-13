use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub enum Args {
    Compile {
        /// Overrides the path of the `titan.toml` file, can be used if your cwd is not the root of the file or in batch compilation.
        #[arg(long, short)]
        project: Option<PathBuf>,
    },
    Run {
        /// Overrides the path of the `titan.toml` file, can be used if your cwd is not the root of the file or in batch compilation.
        #[arg(long, short)]
        project: Option<PathBuf>,
    },
    New {},
    Init {},
    Add {},
    Test {},
}
