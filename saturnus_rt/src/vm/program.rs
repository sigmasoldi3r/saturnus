use super::{
    IntoStResult, Result,
    types::{Any, IntoSaturnus},
};
use mlua::Chunk;

pub struct Program<'a>(pub(crate) Chunk<'a>);

impl<'a> Program<'a> {
    pub async fn exec(self) -> Result<()> {
        self.0.exec_async().await.wrap()
    }
    pub async fn eval(self) -> Result<Any> {
        self.0
            .eval_async::<mlua::Value>()
            .await
            .wrap()
            .map(IntoSaturnus::into_saturnus)
    }
}
