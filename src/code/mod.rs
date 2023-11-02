mod builder;
mod visitor;

pub use builder::Builder;
pub use visitor::{VisitError, Visitor};

pub trait BuilderVisitor: Visitor<Builder> {}
