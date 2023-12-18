use crate::{
    code::{
        ast_visitor::{VisitError, Visitor},
        builder::Builder,
    },
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
#[deprecated(
    since = "v0.2.0",
    note = "We have to rethink how runtime is evaluated! - Saturnus's primary VM will be always Lua."
)]
pub struct RuntimeHost {
    host: rlua::Lua,
    compiler: Box<dyn Visitor>,
    indent: String,
}

impl RuntimeHost {
    pub fn new(indent: String, compiler: Box<dyn Visitor>) -> RuntimeHost {
        let host = rlua::Lua::new();
        RuntimeHost {
            host,
            indent,
            compiler,
        }
    }

    pub fn run(&self, code: &String) -> Result<EvaluationOutput, RuntimeError> {
        let parsed = Script::parse(code).map_err(|err| RuntimeError::ParseError(err))?;
        self.evaluate(&parsed)
    }

    #[deprecated]
    pub fn evaluate(&self, script: &Script) -> Result<EvaluationOutput, RuntimeError> {
        let code = self
            .compiler
            .visit_script(Builder::new(self.indent.clone()), &script)
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
