pub struct RuntimeHost;

#[derive(Debug, Clone)]
pub struct RuntimeError(rlua::Error);

impl RuntimeHost {
    pub fn evaluate(self, code: &String) -> Result<(), RuntimeError> {
        let rt = rlua::Lua::new();
        rt.context(move |ctx| -> rlua::Result<()> {
            ctx.load(&code).eval()?;
            Ok(())
        })
        .map_err(|err| RuntimeError(err))
    }
}
