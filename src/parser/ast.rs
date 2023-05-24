use super::Script;

#[derive(Debug, Clone)]
pub struct Decorator {
    pub target: Expression,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: Identifier,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Identifier,
    pub arguments: Vec<Argument>,
    pub decorators: Vec<Decorator>,
    pub body: Script,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub arguments: Vec<Argument>,
    pub body: ScriptOrExpression,
}

#[derive(Debug, Clone)]
pub struct Tuple(pub Vec<Expression>);

#[derive(Debug, Clone)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub enum MemberSegment {
    ArrayAccess(Expression),
    Expression(Expression),
    Identifier(Identifier),
}

#[derive(Debug, Clone)]
pub struct MemberExpression(pub Vec<MemberSegment>);

#[derive(Debug, Clone)]
pub struct Let {
    pub target: Identifier,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: MemberExpression,
    pub value: Expression,
    pub extra: Option<Operator>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Identifier,
    pub decorators: Vec<Decorator>,
    pub fields: Vec<ClassField>,
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub target: MemberExpression,
    pub static_target: Option<Identifier>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub enum Number {
    Float(f64),
    Integer(i64),
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Expression,
    pub body: Script,
    pub branches: Vec<(Expression, Script)>,
    pub else_branch: Option<Script>,
}

#[derive(Debug, Clone)]
pub struct For {
    pub handler: Identifier,
    pub target: Expression,
    pub body: Script,
}

#[derive(Debug, Clone)]
pub enum ExpressionOrLet {
    Expression(Expression),
    Let(Let),
}

#[derive(Debug, Clone)]
pub struct While {
    pub condition: ExpressionOrLet,
    pub body: Script,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub body: Script,
}

#[derive(Debug, Clone)]
pub enum ScriptOrExpression {
    Script(Script),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Match {
    pub target: Expression,
    pub branches: Vec<(Expression, ScriptOrExpression)>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    If(If),
    Match(Match),
    For(For),
    Loop(Loop),
    While(While),
    Return(Return),
    Class(Class),
    Function(Function),
    Assignment(Assignment),
    Let(Let),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct OperatorOverload {
    pub operator: Operator,
    pub arguments: Vec<Argument>,
    pub body: Script,
}

#[derive(Debug, Clone)]
pub enum ClassField {
    Method(Function),
    Let(Let),
    Operator(OperatorOverload),
}

#[derive(Debug, Clone)]
pub enum Operator {
    // Arithmetic
    Plus,
    Minus,
    Quotient,
    Product,
    Power,
    Remainder,
    Concat,
    Range,
    // Comparison
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    NotEqual,
    Starship,
    Funnel,
    // Logic
    LogicOr,
    LogicAnd,
    LogicNor,
    LogicNand,
    LogicXOr,
    LogicNot,
    // Binary
    BWiseAnd,
    BWiseOr,
    BWiseNot,
    BWiseLShift,
    BWiseRShift,
    BWiseLShiftRoundtrip,
    BWiseRShiftRoundtrip,
    // Special operators (No native equivalent for these)
    Count, // Except this, in Lua.
    ArrowRight,
    ArrowLeft,
    BothWays,
    ArrowStandRight,
    ArrowStandLeft,
    ArrowStandBoth,
    Exclamation,
    Tilde,
    Disjoin,
    Elastic,
    ElasticRight,
    ElasticLeft,
    Elvis,
    Coalesce,
    PinguRight,
    PinguLeft,
    PinguBoth,
    PipeRight,
    PipeLeft,
    AskRight,
    AskLeft,
    Bolted,
    Dollar,
    ExclamationQuestion,
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Expression,
    pub right: Expression,
    pub operator: Operator,
}
impl Into<Expression> for BinaryExpression {
    fn into(self) -> Expression {
        Expression::Binary(Box::new(self))
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub expression: Expression,
    pub operator: Operator,
}
impl Into<Expression> for UnaryExpression {
    fn into(self) -> Expression {
        Expression::Unary(Box::new(self))
    }
}

#[derive(Debug, Clone)]
pub struct Vector {
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub key_values: Vec<(TableKeyExpression, Expression)>,
}

#[derive(Debug, Clone)]
pub enum TableKeyExpression {
    Identifier(Identifier),
    Expression(Expression),
    Implicit(Identifier),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Lambda(Box<Lambda>),
    Reference(MemberExpression),
    Call(CallExpression),
    Tuple(Tuple),
    Tuple1(Box<Expression>),
    Table(Table),
    Vector(Vector),
    Number(Number),
    String(String),
    Binary(Box<BinaryExpression>),
    Unary(Box<UnaryExpression>),
    Unit,
}
