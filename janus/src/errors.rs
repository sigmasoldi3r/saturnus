use std::process::exit;

pub enum ExitCode {
    CannotOpenFile,
    BadJanusFile,
}
impl ExitCode {
    pub fn exit(self) -> ! {
        match self {
            ExitCode::CannotOpenFile => exit(1),
            ExitCode::BadJanusFile => exit(2),
        }
    }
}
