pub mod ast;
pub mod builders;

#[cfg(test)]
mod grammar_test;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    pub grammar,
    "/parsing/grammar.rs"
);
