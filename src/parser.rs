use std::vec;

peg::parser! {
    grammar matra_script() for str {
        pub rule script() -> Script
            = _ statements:statement() ** __ _
            { Script { statements } }

        // Statements
        rule statement() -> Statement
            = e:class() { Statement::Class(e) }
            / e:func() { Statement::Function(e) }
            / e:if_stmt() { Statement::If(e) }
            / e:declare_var() { Statement::Declaration(e) }
            / e:assignment() { Statement::Assignment(e) }
            / e:return_stmt() { Statement::Return(e) }
            / e:expression() _ EOS() { Statement::Expression(e) }

        rule if_stmt() -> If
            = "if" __ condition:expression() __ "then" body:script()
              branches:("else" __ "if" __ c:expression() __ "then" s:script() { (c, s) })*
              else_branch:("else" e:script() {e})?
              "end"
            { If { condition, body, branches, else_branch } }

        rule func() -> Function
            = decorators:decorator_list() FN() __ name:identifier() _ arguments:argument_list() body:script() END()
            { Function { name, decorators, body, arguments } }

        rule class() -> Class
            = decorators:decorator_list() CLASS() __ name:identifier() __ END()
            { Class { name, decorators } }

        rule declare_var() -> Declaration
            = "let" __ target:identifier() value:(_ "=" _ e:expression(){e})? _ EOS()
            { Declaration { target, value } }

        rule assignment() -> Assignment
            = target:identifier() _ "=" _ value:expression() _ EOS()
            { Assignment { target, value } }

        rule return_stmt() -> Return
            = "return" __ value:expression() _ EOS()
            { Return { value } }

        // Expressions
        pub rule expression() -> Expression = precedence! {
            "-" _ expression:@ { UnaryExpression { expression, operator: Operator::Minus }.into() }
            "+" _ expression:@ { UnaryExpression { expression, operator: Operator::Plus }.into() }
            "#?" _ expression:@ { UnaryExpression { expression, operator: Operator::Count }.into() }
            "not" _ expression:@ { UnaryExpression { expression, operator: Operator::LogicNot }.into() }
            "!" _ expression:@ { UnaryExpression { expression, operator: Operator::Exclamation }.into() }
            "~" _ expression:@ { UnaryExpression { expression, operator: Operator::Tilde }.into() }
            "¬" _ expression:@ { UnaryExpression { expression, operator: Operator::Bolted }.into() }
            "$" _ expression:@ { UnaryExpression { expression, operator: Operator::Dollar }.into() }
            "!?" _ expression:@ { UnaryExpression { expression, operator: Operator::ExclamationQuestion }.into() }
            --
            left:(@) _ "+" _ right:@ { BinaryExpression { left, right, operator: Operator::Plus }.into() }
            left:(@) _ "-" _ right:@ { BinaryExpression { left, right, operator: Operator::Minus }.into() }
            --
            left:(@) _ "*" _ right:@ { BinaryExpression { left, right, operator: Operator::Product }.into() }
            left:(@) _ "/" _ right:@ { BinaryExpression { left, right, operator: Operator::Quotient }.into() }
            --
            left:@ _ "**" _ right:(@) { BinaryExpression { left, right, operator: Operator::Power }.into() }
            --
            left:(@) _ "%" _ right:@ { BinaryExpression { left, right, operator: Operator::Remainder }.into() }
            --
            left:(@) _ ">=<" _ right:@ { BinaryExpression { left, right, operator: Operator::Funnel }.into() }
            left:(@) _ ">=" _ right:@ { BinaryExpression { left, right, operator: Operator::GreaterEqual }.into() }
            left:(@) _ ">" _ right:@ { BinaryExpression { left, right, operator: Operator::Greater }.into() }
            left:(@) _ "<=>" _ right:@ { BinaryExpression { left, right, operator: Operator::Starship }.into() }
            left:(@) _ "<=" _ right:@ { BinaryExpression { left, right, operator: Operator::LessEqual }.into() }
            left:(@) _ "<>" _ right:@ { BinaryExpression { left, right, operator: Operator::NotEqual }.into() }
            left:(@) _ "<" _ right:@ { BinaryExpression { left, right, operator: Operator::Less }.into() }
            left:(@) _ "==" _ right:@ { BinaryExpression { left, right, operator: Operator::Equal }.into() }
            --
            left:(@) _ "and" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicAnd }.into() }
            left:(@) _ "or" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicOr }.into() }
            left:(@) _ "xor" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicXOr }.into() }
            left:(@) _ "nand" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicNand }.into() }
            left:(@) _ "nor" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicNor }.into() }
            --
            left:(@) _ "<~>" _ right:@ { BinaryExpression { left, right, operator: Operator::Elastic }.into() }
            left:(@) _ "<~" _ right:@ { BinaryExpression { left, right, operator: Operator::ElasticLeft }.into() }
            left:(@) _ "~>" _ right:@ { BinaryExpression { left, right, operator: Operator::ElasticRight }.into() }
            left:(@) _ "<:>" _ right:@ { BinaryExpression { left, right, operator: Operator::PinguBoth }.into() }
            left:(@) _ "<:" _ right:@ { BinaryExpression { left, right, operator: Operator::PinguLeft }.into() }
            left:(@) _ ":>" _ right:@ { BinaryExpression { left, right, operator: Operator::PinguRight }.into() }
            left:(@) _ "<-|->" _ right:@ { BinaryExpression { left, right, operator: Operator::ArrowStandBoth }.into() }
            left:(@) _ "<-|" _ right:@ { BinaryExpression { left, right, operator: Operator::ArrowStandLeft }.into() }
            left:(@) _ "|->" _ right:@ { BinaryExpression { left, right, operator: Operator::ArrowStandRight }.into() }
            left:(@) _ "<->" _ right:@ { BinaryExpression { left, right, operator: Operator::BothWays }.into() }
            left:(@) _ "<-" _ right:@ { BinaryExpression { left, right, operator: Operator::ArrowLeft }.into() }
            left:(@) _ "->" _ right:@ { BinaryExpression { left, right, operator: Operator::ArrowRight }.into() }
            left:(@) _ "<|>" _ right:@ { BinaryExpression { left, right, operator: Operator::Disjoin }.into() }
            left:(@) _ "<|" _ right:@ { BinaryExpression { left, right, operator: Operator::PipeLeft }.into() }
            left:(@) _ "|>" _ right:@ { BinaryExpression { left, right, operator: Operator::PipeRight }.into() }
            --
            left:(@) _ "?:" _ right:@ { BinaryExpression { left, right, operator: Operator::Elvis }.into() }
            left:(@) _ "??" _ right:@ { BinaryExpression { left, right, operator: Operator::Coalesce }.into() }
            --
            e:string() { Expression::String(e) }
            e:number() { Expression::Number(e) }
            e:lambda() { Expression::Lambda(Box::new(e)) }
            e:call_expr()  { Expression::Call(e) }
            e:dot_expr() { Expression::Reference(e) }
            unit() { Expression::Unit }
            e:tuple_expr() { Expression::Tuple(e) }
            "(" _ e:expression() _ ")" { e }
        }

        rule dot_expr() -> DotExpression
            = value:identifier() ++ (_ "." _) { DotExpression(value) }

        rule lambda() -> Lambda
            = FN() _ arguments:argument_list() _ expr:expression() _ END()
            { Lambda { arguments, body: LambdaBody::Simple(expr) } }
            / FN() _ arguments:argument_list() body:script() END()
            { Lambda { arguments, body: LambdaBody::Complex(body) } }

        rule call_expr() -> CallExpression
            = target:dot_expr() _ arguments:tuple_expr()
            { CallExpression { target, arguments } }

        // Literals
        rule number() -> Number
            = value:$(DIGIT()+ "." DIGIT()+) { Number::Float(value.parse().unwrap()) }
            / value:$(DIGIT()+) { Number::Integer(value.parse().unwrap()) }

        rule string() -> String
            = "\"" value:$((!"\"" ANY())*) "\"" { value.into() }

        // Auxiliaries and sub-expressions
        rule argument_list() -> Vec<Argument>
            = "(" _ args:argument() ** (_ "," _) _ ")" { args }

        rule argument() -> Argument
            = decorators:decorator_list() name:identifier()
            { Argument { name, decorators } }

        rule decorator_list() -> Vec<Decorator>
            = e:decorator() ++ __ __ { e }
            / { vec![] }

        rule decorator() -> Decorator
            = "@" _ target:call_expr() { Decorator { target: target.target, arguments: Some(target.arguments) } }
            / "@" _ target:dot_expr() { Decorator { target, arguments: None } }

        rule identifier() -> Identifier
            = value:$(IDENT())
            { Identifier(value.into()) }

        rule comma_expr() -> Vec<Expression>
            = e:expression() ** (_ "," _) { e }

        rule tuple_expr() -> Tuple
            = "(" _ expr:comma_expr() _ ")"
            { Tuple(expr) }

        rule unit() -> Expression = "()" { Expression::Unit }

        // Tokens
        rule IDENT() = ALPHA() (ALPHA() / DIGIT())*
        rule LET() = "let"
        rule MUT() = "mut"
        rule CLASS() = "class"
        rule END() = "end"
        rule FN() = "fn"
        rule ANY() = [_]
        rule BLANK() = ['\t'|' ']
        rule WS() = BLANK() / EOL()
        rule EOL() = ['\r'|'\n']
        rule EOS() = EOL() / ";"
        rule ALPHA() = ['A'..='Z'|'a'..='z'|'_']
        rule DIGIT() = ['0'..='9']
        rule _ = WS()*
        rule __ = WS()+
    }
}

#[derive(Debug)]
pub struct Decorator {
    pub target: DotExpression,
    pub arguments: Option<Tuple>,
}

#[derive(Debug)]
pub struct Argument {
    pub name: Identifier,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug)]
pub struct Function {
    pub name: Identifier,
    pub arguments: Vec<Argument>,
    pub decorators: Vec<Decorator>,
    pub body: Script,
}

#[derive(Debug)]
pub enum LambdaBody {
    Complex(Script),
    Simple(Expression),
}

#[derive(Debug)]
pub struct Lambda {
    pub arguments: Vec<Argument>,
    pub body: LambdaBody,
}

#[derive(Debug)]
pub struct Tuple(pub Vec<Expression>);

#[derive(Debug)]
pub struct Identifier(pub String);

#[derive(Debug)]
pub struct DotExpression(pub Vec<Identifier>);

#[derive(Debug)]
pub struct Declaration {
    pub target: Identifier,
    pub value: Option<Expression>,
}

#[derive(Debug)]
pub struct Assignment {
    pub target: Identifier,
    pub value: Expression,
}

#[derive(Debug)]
pub struct Class {
    pub name: Identifier,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug)]
pub struct CallExpression {
    pub target: DotExpression,
    pub arguments: Tuple,
}

#[derive(Debug)]
pub struct Return {
    pub value: Expression,
}

#[derive(Debug)]
pub enum Number {
    Float(f64),
    Integer(i64),
}

#[derive(Debug)]
pub struct If {
    pub condition: Expression,
    pub body: Script,
    pub branches: Vec<(Expression, Script)>,
    pub else_branch: Option<Script>,
}

#[derive(Debug)]
pub enum Statement {
    If(If),
    For,
    Loop,
    While,
    Return(Return),
    Class(Class),
    Function(Function),
    Assignment(Assignment),
    Declaration(Declaration),
    Match,
    Expression(Expression),
}

#[derive(Debug)]
pub enum Operator {
    // Arithmetic
    Plus,
    Minus,
    Quotient,
    Product,
    Power,
    Remainder,
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct UnaryExpression {
    pub expression: Expression,
    pub operator: Operator,
}
impl Into<Expression> for UnaryExpression {
    fn into(self) -> Expression {
        Expression::Unary(Box::new(self))
    }
}

#[derive(Debug)]
pub enum Expression {
    Lambda(Box<Lambda>),
    Reference(DotExpression),
    Call(CallExpression),
    Tuple(Tuple),
    Number(Number),
    String(String),
    Binary(Box<BinaryExpression>),
    Unary(Box<UnaryExpression>),
    Unit,
}

pub struct ParseFailure {
    pub parse_error: Option<peg::error::ParseError<peg::str::LineCol>>,
    pub fragment: String,
}
impl std::fmt::Debug for ParseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.fragment.clone();
        let loc = self.parse_error.as_ref().unwrap().location.clone();
        let wave = " ".repeat(loc.column) + "^ here";
        f.write_fmt(format_args!(
            "Parse Failed! {:?}\n at {}:{},\n   {}\n  {}\n",
            self.parse_error.as_ref().unwrap(),
            loc.line,
            loc.column,
            line,
            wave
        ))
    }
}
impl ParseFailure {
    fn new(e: peg::error::ParseError<peg::str::LineCol>, fragment: &String) -> Self {
        ParseFailure {
            parse_error: Some(e.clone()),
            fragment: fragment
                .split("\n")
                .skip(e.location.line - 1)
                .next()
                .unwrap()
                .to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Script {
    pub statements: Vec<Statement>,
}
impl Script {
    pub fn parse<I>(input: I) -> Result<Self, ParseFailure>
    where
        I: Into<String>,
    {
        let fragment: String = input.into();
        matra_script::script(&fragment).map_err(|e| ParseFailure::new(e, &fragment))
    }
    pub fn parse_expression<I>(input: I) -> Result<Expression, ParseFailure>
    where
        I: Into<String>,
    {
        let fragment: String = input.into();
        matra_script::expression(&fragment).map_err(|e| ParseFailure::new(e, &fragment))
    }
}
