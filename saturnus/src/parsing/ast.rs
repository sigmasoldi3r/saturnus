use macros::{bitmask_impl, wrapper_enum};

#[derive(Debug, Clone)]
pub enum Number {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub struct SatString {
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum Boolean {
    True,
    False,
}

#[wrapper_enum]
#[derive(Debug, Clone)]
pub enum MapKey {
    Identifier,
    SatString,
    Expr,
}

#[derive(Debug, Clone)]
pub struct MapLiteral {
    pub entries: Vec<(MapKey, Expr)>,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub values: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct TupleLiteral {
    pub values: Vec<Expr>,
}
impl TupleLiteral {
    pub fn unit() -> Self {
        Self {
            values: Default::default(),
        }
    }
    pub fn is_unit(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    And,
    Or,
    Not,
    BAnd,
    BOr,
    BXor,
    BNot,
    LShift,
    LShiftRot,
    RShift,
    RShiftRot,
    StrCat,
    Range,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Eq,
    Neq,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Bop {
    pub left: Box<Expr>,
    pub op: Operator,
    pub right: Box<Expr>,
}
impl Bop {
    pub fn new(left: Expr, op: Operator, right: Expr) -> Expr {
        Expr::Bop(Bop {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Uop {
    pub op: Operator,
    pub expr: Box<Expr>,
}
impl Uop {
    pub fn new(op: Operator, expr: Expr) -> Expr {
        Self {
            op,
            expr: Box::new(expr),
        }
        .into_expr()
    }
}

#[wrapper_enum]
#[derive(Debug, Clone)]
pub enum AssignmentTarget {
    Member,
    ArrayAccess,
    Identifier,
}
impl AssignmentTarget {
    pub fn to_expr(self) -> Expr {
        match self {
            AssignmentTarget::Member(member) => Expr::Member(member),
            AssignmentTarget::ArrayAccess(array_access) => Expr::ArrayAccess(array_access),
            AssignmentTarget::Identifier(identifier) => Expr::Identifier(identifier),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub left: AssignmentTarget,
    pub right: Box<Expr>,
    pub op: Option<Operator>,
}
impl Assignment {
    pub fn new(left: AssignmentTarget, op: Option<Operator>, right: Expr) -> Self {
        Self {
            left,
            right: Box::new(right),
            op,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub target: Box<Expr>,
    pub arguments: Vec<Expr>,
    pub is_null_safe: bool,
}
impl Call {
    pub fn new(target: Expr, arguments: Vec<Expr>, is_null_safe: bool) -> Expr {
        Expr::Call(Self {
            target: Box::new(target),
            arguments,
            is_null_safe,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ArrayAccess {
    pub target: Box<Expr>,
    pub arguments: Vec<Expr>,
    pub is_null_safe: bool,
}
impl ArrayAccess {
    pub fn new(target: Expr, arguments: Vec<Expr>, is_null_safe: bool) -> Expr {
        Expr::ArrayAccess(Self {
            target: Box::new(target),
            arguments,
            is_null_safe,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub value: String,
    pub is_escaped: bool,
}
impl Identifier {
    pub fn new(value: impl Into<String>, is_escaped: bool) -> Expr {
        Expr::Identifier(Self {
            value: value.into(),
            is_escaped,
        })
    }
    pub fn is_void(&self) -> bool {
        self.value == "_"
    }
}

#[derive(Debug, Clone)]
pub enum Destructure {
    Identifier(Identifier),
    Array(Vec<DestructureEntry>),
    Map(Vec<DestructureEntry>),
    Tuple(Vec<DestructureEntry>),
}

#[derive(Debug, Clone)]
pub enum DestructureEntry {
    Identifier(Identifier),
    Array(Vec<DestructureEntry>),
    Map(Vec<DestructureEntry>),
    Tuple(Vec<DestructureEntry>),
    Aliasing(Identifier, Box<DestructureEntry>),
}

#[derive(Debug, Clone)]
pub enum MemberOp {
    Member,
    CoalesceMember,
    Static,
    Dispatch,
}

#[derive(Debug, Clone)]
pub struct Member {
    pub target: Box<Expr>,
    pub op: MemberOp,
    pub field: Identifier,
}
impl Member {
    pub fn new(target: Expr, op: MemberOp, field: Expr) -> Expr {
        Expr::Member(Self {
            target: Box::new(target),
            op,
            field: field.unwrap_identifier(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
}
impl LambdaExpr {
    pub fn new(params: Vec<Param>, body: Vec<Statement>) -> Expr {
        Expr::LambdaExpr(Self { params, body })
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Identifier,
    pub type_def: Option<TypeDef>,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: Identifier,
    pub generic_args: Option<Vec<TypeDef>>,
}

#[derive(Debug, Clone)]
pub struct ElseIf {
    pub condition: Box<Expr>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Box<Expr>,
    pub body: Vec<Statement>,
    pub else_if_blocks: Vec<ElseIf>,
    pub else_block: Option<Vec<Statement>>,
}
impl IfStatement {
    pub fn new(
        condition: Expr,
        body: Vec<Statement>,
        else_if_blocks: Vec<ElseIf>,
        else_block: Option<Vec<Statement>>,
    ) -> Statement {
        Statement::IfStatement(Self {
            condition: Box::new(condition),
            body,
            else_if_blocks,
            else_block,
        })
    }
}

#[derive(Debug, Clone)]
pub struct For {
    pub assignment: Destructure,
    pub expr: Box<Expr>,
    pub body: Vec<Statement>,
}
impl For {
    pub fn new(assignment: Destructure, expr: Expr, body: Vec<Statement>) -> Statement {
        Statement::For(Self {
            assignment,
            expr: Box::new(expr),
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct While {
    pub condition: Box<Expr>,
    pub body: Vec<Statement>,
}
impl While {
    pub fn new(condition: Expr, body: Vec<Statement>) -> Statement {
        Statement::While(Self {
            condition: Box::new(condition),
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub body: Vec<Statement>,
}
impl Loop {
    pub fn new(body: Vec<Statement>) -> Statement {
        Statement::Loop(Self { body })
    }
}

#[derive(Debug, Clone)]
#[bitmask_impl("pub", "static", "partial")]
pub struct DefModifiers {
    mask: u8,
}

#[derive(Debug, Clone)]
pub struct Fn {
    pub name: Identifier,
    pub modifiers: DefModifiers,
    pub arguments: Vec<Param>,
    pub body: Vec<Statement>,
}
impl Fn {
    pub fn new(
        name: Identifier,
        modifiers: DefModifiers,
        arguments: Vec<Param>,
        body: Vec<Statement>,
    ) -> Self {
        Self {
            name,
            modifiers,
            arguments,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: Destructure,
    pub type_def: Option<TypeDef>,
    pub initializer: Option<Expr>,
    pub modifiers: DefModifiers,
}
impl Let {
    pub fn new(name: Identifier, modifiers: DefModifiers, init: Expr) -> Self {
        Self {
            name: Destructure::Identifier(name),
            modifiers,
            type_def: None,
            initializer: Some(init),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: Identifier,
    pub parent: Option<Identifier>,
    pub fields: Vec<ClassField>,
    pub modifiers: DefModifiers,
}
impl ClassDef {
    pub fn new(
        name: Identifier,
        modifiers: DefModifiers,
        parent: Option<Identifier>,
        fields: Vec<ClassField>,
    ) -> Statement {
        Statement::ClassDef(Self {
            name,
            modifiers,
            parent,
            fields,
        })
    }
}

#[wrapper_enum]
#[derive(Debug, Clone)]
pub enum ClassField {
    Fn,
    Let,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Box<Expr>,
}
impl Return {
    pub fn new(expr: Expr) -> Statement {
        Statement::Return(Self {
            value: Box::new(expr),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Break;

#[derive(Debug, Clone)]
pub struct Skip;

#[derive(Debug, Clone)]
pub struct Use {
    pub path: Vec<Identifier>,
    pub use_tree: Option<Vec<Use>>,
}

#[wrapper_enum]
#[derive(Debug, Clone)]
pub enum Expr {
    Call,
    ArrayAccess,
    Bop,
    Uop,
    LambdaExpr,
    Number,
    Boolean,
    SatString,
    Identifier,
    Member,
    MapLiteral,
    ArrayLiteral,
    TupleLiteral,
}

#[wrapper_enum]
#[derive(Debug, Clone)]
pub enum Statement {
    Use,
    IfStatement,
    ClassDef,
    Assignment,
    Let,
    Fn,
    Loop,
    While,
    For,
    Break,
    Skip,
    Return,
    Expr,
}
