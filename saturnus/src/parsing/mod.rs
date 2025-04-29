pub mod ast;
pub mod builders;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    pub grammar,
    "/parsing/grammar.rs"
);
