use std::path::PathBuf;

pub struct SaturnusIR {
    compiled_source: Vec<u8>,
}
impl From<String> for SaturnusIR {
    fn from(value: String) -> Self {
        Self {
            compiled_source: value.into_bytes(),
        }
    }
}
impl ToString for SaturnusIR {
    fn to_string(&self) -> String {
        String::from_utf8(self.compiled_source.clone()).unwrap()
    }
}

impl<'a> mlua::AsChunk<'a> for SaturnusIR {
    fn source(self) -> std::io::Result<std::borrow::Cow<'a, [u8]>> {
        Ok(std::borrow::Cow::from(self.compiled_source))
    }
}

pub trait SourceCode {
    fn source(self) -> String;
    fn location(&self) -> Option<PathBuf>;
}
impl SourceCode for &'static str {
    fn source(self) -> String {
        self.into()
    }
    fn location(&self) -> Option<PathBuf> {
        None
    }
}
