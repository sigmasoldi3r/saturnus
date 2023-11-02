mod builder;
mod generator;
mod visitor;

pub use builder::Builder;
pub use generator::Generator;
pub use visitor::{VisitError, Visitor};

pub trait BuilderVisitor: Visitor<Builder> {}
