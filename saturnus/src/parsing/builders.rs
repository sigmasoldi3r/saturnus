use crate::parsing::ast::DestructureEntry;

use super::ast::{ArrayAccess, Destructure, Expr, Identifier, Member, MemberOp, Number, SatString};

pub trait AddMember {
    fn add_member(self, identifier: Identifier) -> Expr;
}
impl AddMember for Expr {
    fn add_member(self, identifier: Identifier) -> Expr {
        Expr::Member(Member {
            target: Box::new(self),
            op: MemberOp::Member,
            field: identifier,
        })
    }
}

pub trait AddArrayAccess {
    fn array_access(self, key: Expr) -> Expr;
}
impl AddArrayAccess for Expr {
    fn array_access(self, key: Expr) -> Expr {
        Expr::ArrayAccess(ArrayAccess {
            target: Box::new(self),
            arguments: vec![key],
            is_null_safe: false,
        })
    }
}

pub trait AsExpr {
    fn as_expr(self) -> Expr;
}
impl AsExpr for i64 {
    fn as_expr(self) -> Expr {
        Expr::Number(Number::Int(self))
    }
}
impl AsExpr for String {
    fn as_expr(self) -> Expr {
        Expr::SatString(SatString { value: self })
    }
}

pub trait LeafCollector {
    fn collect_leaves(&self) -> Vec<Identifier>;
}
impl LeafCollector for Destructure {
    fn collect_leaves(&self) -> Vec<Identifier> {
        fn collect(items: &Vec<DestructureEntry>) -> Vec<Identifier> {
            items
                .iter()
                .map(|item| match item {
                    DestructureEntry::Identifier(identifier) => vec![identifier.clone()],
                    DestructureEntry::Array(items) => collect(items),
                    DestructureEntry::Map(items) => collect(items),
                    DestructureEntry::Tuple(items) => collect(items),
                    DestructureEntry::Aliasing(_, destructure_entry) => {
                        collect(&vec![(**destructure_entry).clone()])
                    }
                })
                .flatten()
                .collect()
        }
        match self {
            Destructure::Identifier(identifier) => vec![identifier.clone()],
            Destructure::Array(items) => collect(items),
            Destructure::Map(items) => collect(items),
            Destructure::Tuple(items) => collect(items),
        }
    }
}
