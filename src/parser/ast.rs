use super::grammar::Script;

#[derive(Debug, Clone)]
pub struct Decorator {
    pub target: CallExpression,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: Identifier,
    pub spread: bool,
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
pub struct Extern {
    pub id: String,
    pub src: String,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub arguments: Vec<Argument>,
    pub body: ScriptOrExpression,
}

#[derive(Debug, Clone)]
pub struct Do {
    pub body: Script,
}

#[derive(Debug, Clone)]
pub struct Tuple(pub Vec<Expression>);

#[derive(Debug, Clone)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub enum MemberSegment {
    Computed(Expression),
    Identifier(Identifier),
    Dispatch(Identifier),
}
impl Into<CallExpressionVariant> for MemberSegment {
    fn into(self) -> CallExpressionVariant {
        CallExpressionVariant::Member(self)
    }
}

#[derive(Debug, Clone)]
pub struct MemberExpression {
    pub head: Expression,
    pub tail: Vec<MemberSegment>,
}

#[derive(Debug, Clone)]
pub enum DestructureOrigin {
    Tuple,
    Array,
    Table,
}

#[derive(Debug, Clone)]
pub struct Destructuring(pub Vec<Identifier>, pub DestructureOrigin);

#[derive(Debug, Clone)]
pub enum AssignmentTarget {
    Destructuring(Destructuring),
    Identifier(Identifier),
}

#[derive(Debug, Clone)]
pub struct Let {
    pub target: AssignmentTarget,
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
pub struct CallSubExpression {
    pub callee: Option<MemberExpression>,
    pub arguments: Vec<Expression>,
}
impl Into<CallExpressionVariant> for CallSubExpression {
    fn into(self) -> CallExpressionVariant {
        CallExpressionVariant::Call(self)
    }
}

#[derive(Debug, Clone)]
pub enum CallExpressionVariant {
    Call(CallSubExpression),
    Member(MemberSegment),
}

// TODO: Implement macros!
#[derive(Debug, Clone)]
pub enum MacroCallArguments {
    FunctionLike(Vec<Expression>),
    BlockLike(Script),
}

#[derive(Debug, Clone)]
pub enum IdentifierOrCall {
    Call(CallExpression),
    Identifier(Identifier),
}

#[derive(Debug, Clone)]
pub struct MacroDecorator {
    pub macros: Vec<IdentifierOrCall>,
    pub target: Statement,
}

#[derive(Debug, Clone)]
pub struct MacroCallExpression {
    pub target: Identifier,
    pub arguments: MacroCallArguments,
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub head: CallSubExpression,
    pub tail: Vec<CallExpressionVariant>,
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
    pub handler: AssignmentTarget,
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
pub struct UseStatement {
    pub module: Vec<String>,
    pub expanded: Option<Vec<Identifier>>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    MacroDecorator(Box<MacroDecorator>),
    If(If),
    Match(Match),
    For(For),
    Loop(Loop),
    While(While),
    Return(Return),
    Class(Class),
    Extern(Extern),
    Function(Function),
    Assignment(Assignment),
    Let(Let),
    Expression(Expression),
    UseStatement(UseStatement),
}

#[derive(Debug, Clone)]
pub enum ClassField {
    Method(Function),
    Let(Let),
}

#[derive(Debug, Clone)]
pub struct Operator(pub String);

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
pub struct SpreadExpression {
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub struct Vector {
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub key_values: Vec<(TableKeyExpression, Option<Expression>)>,
}

#[derive(Debug, Clone)]
pub enum TableKeyExpression {
    Identifier(Identifier),
    Expression(Expression),
    Implicit(Identifier),
}

#[derive(Debug, Clone)]
pub struct StringLiteral(pub String);

#[derive(Debug, Clone)]
pub enum Expression {
    Lambda(Box<Lambda>),
    Reference(Box<MemberExpression>),
    Identifier(Identifier),
    Call(Box<CallExpression>),
    Tuple(Tuple),
    Tuple1(Box<Expression>),
    Table(Table),
    Do(Do),
    Vector(Vector),
    Number(Number),
    String(StringLiteral),
    Binary(Box<BinaryExpression>),
    Unary(Box<UnaryExpression>),
    Spread(Box<SpreadExpression>),
    Unit,
}
