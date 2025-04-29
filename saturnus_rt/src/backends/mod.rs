use crate::core::Table;

mod lua;

pub use lua::LuaRt;

#[derive(Debug)]
pub enum RuntimeError {
    Unknown(String),
}

pub struct RtEnv {
    pub globals: Table,
}

pub trait Runtime {
    fn run(&mut self, code: String) -> Result<(), RuntimeError>;
}

pub struct RtProvider;
impl RtProvider {
    pub fn default(config: RtEnv) -> Box<dyn Runtime> {
        Box::new(lua::LuaRt::default(config))
    }
}
