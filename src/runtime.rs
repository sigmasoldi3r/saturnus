use std::error::Error;

use crate::parser::Script;

pub struct RuntimeHost;

impl RuntimeHost {
    pub fn evaluate(self, ast: Script) -> Result<(), dyn Error> {
        let rt = rlua::Lua::new();
        rt.context(move |ctx| -> rlua::Result<()> {
            ctx.load(&output).eval()?;
            Ok(())
        })
    }
}
