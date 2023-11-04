mod builder;
mod generation;
pub mod macros;
mod visitor;

pub use builder::Builder;
pub use generation::CodeEmitter;
pub use visitor::{VisitError, Visitor};

pub trait BuilderVisitor: Visitor<Builder> {}
