use crate::{
    code,
    code::{VisitError, Visitor},
    lua,
    parser::Script,
};

#[derive(Debug)]
pub enum RuntimeError {
    EvaluationError(rlua::Error),
    ParseError(peg::error::ParseError<peg::str::LineCol>),
    CompilationError(VisitError),
}

pub struct EvaluationOutput {
    pub stdout: String,
    pub vars: (),
}

/// # Runtime Host
///
/// The runtime host is an abstraction that takes care of running the Saturnus
/// code on the fly. This abstraction contains the current implementation of
/// the virtual machine, parser and such.
///
/// This host will take care of evaluating the incoming Saturnus code.
pub struct RuntimeHost {
    host: rlua::Lua,
    indent: String,
}

impl RuntimeHost {
    pub fn new(indent: String) -> RuntimeHost {
        let host = rlua::Lua::new();
        RuntimeHost { host, indent }
    }

    pub fn run(&self, code: &String) -> Result<EvaluationOutput, RuntimeError> {
        let parsed = Script::parse(code).map_err(|err| RuntimeError::ParseError(err))?;
        self.evaluate(&parsed)
    }

    pub fn evaluate(&self, script: &Script) -> Result<EvaluationOutput, RuntimeError> {
        let code = lua::LuaEmitter
            .visit_script(
                code::Builder::new(self.indent.clone())
                    .put("-- Compiled by Saturnus compiler, warning: Changes may be discarded!"),
                &script,
            )
            .map_err(|err| RuntimeError::CompilationError(err))?
            .collect();
        self.host
            .context(move |ctx| -> rlua::Result<()> {
                ctx.load(&code).eval()?;
                Ok(())
            })
            .map_err(|err| RuntimeError::EvaluationError(err))?;
        Ok(EvaluationOutput {
            stdout: "".into(),
            vars: (),
        })
    }
}
