use super::{
    IntoStResult, Result,
    types::{Any, IntoSaturnus},
};
use mlua::Chunk;

pub struct Program<'a>(pub(crate) Chunk<'a>);

impl<'a> Program<'a> {
    pub fn exec(self) -> Result<()> {
        self.0.exec().wrap()
    }
    pub fn eval(self) -> Result<Any> {
        self.0
            .eval::<mlua::Value>()
            .wrap()
            .map(IntoSaturnus::into_saturnus)
    }
}
