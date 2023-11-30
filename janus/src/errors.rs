use std::process::exit;

pub enum ExitCode {
    CannotOpenFile,
    BadJanusFile,
    TargetNotSupported,
    UnknownModuleSystem,
    CannotCreateDistFolders,
    CannotResolveDependencies,
    FailedCompilation,
    Unknown,
    Ok,
}
impl ExitCode {
    pub fn exit(self) -> ! {
        match self {
            ExitCode::CannotOpenFile => exit(1),
            ExitCode::BadJanusFile => exit(2),
            ExitCode::TargetNotSupported => exit(3),
            ExitCode::UnknownModuleSystem => exit(4),
            ExitCode::CannotCreateDistFolders => exit(5),
            ExitCode::CannotResolveDependencies => exit(6),
            ExitCode::FailedCompilation => exit(7),
            ExitCode::Unknown => exit(-1),
            ExitCode::Ok => exit(0),
        }
    }
}
