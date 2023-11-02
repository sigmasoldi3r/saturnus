mod builder;
mod generator;
pub mod macros;
mod visitor;

pub use builder::Builder;
pub use generator::Generator;
pub use visitor::{VisitError, Visitor};

pub trait BuilderVisitor: Visitor<Builder> {}
