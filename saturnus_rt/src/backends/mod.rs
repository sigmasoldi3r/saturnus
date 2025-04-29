use crate::core::Table;

mod lua;

pub use lua::LuaRt;

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub source_name: String,
}

pub struct RtEnv {
    pub globals: Table,
}

pub trait Runtime {
    fn run(&mut self, chunks: Vec<(String, String)>) -> Result<(), RuntimeError>;
}

pub struct RtProvider;
impl RtProvider {
    pub fn default(config: RtEnv) -> Box<dyn Runtime> {
        Box::new(lua::LuaRt::default(config))
    }
}
