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
            / e:declare_var() { Statement::Declaration(e) }
            / e:assignment() { Statement::Assignment(e) }
            / e:return_stmt() { Statement::Return(e) }
            / e:expression() _ EOS() { Statement::Expression(e) }

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
        rule expression() -> Expression
            = e:string() { Expression::String(e) }
            / e:number() { Expression::Number(e) }
            / e:lambda() { Expression::Lambda(e) }
            / e:call_expr()  { Expression::Call(e) }
            / e:dot_expr() { Expression::Reference(e) }
            / e:tuple_expr() { Expression::Tuple(e) }
            / "(" _ e:expression() _ ")" { e }

        rule dot_expr() -> DotExpression
            = value:identifier() ++ (_ "." _) { DotExpression(value) }

        rule lambda() -> Lambda
            = FN() _ arguments:argument_list() body:script() END()
            { Lambda { arguments, body } }

        rule call_expr() -> CallExpression
            = target:dot_expr() _ arguments:tuple_expr()
            { CallExpression { target, arguments } }

        // Literals
        rule number() -> Number
            = value:$("-"? DIGIT()+ "." DIGIT()+) { Number::Float(value.parse().unwrap()) }
            / value:$("-"? DIGIT()+) { Number::Integer(value.parse().unwrap()) }

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

        rule unit() -> Expression = "nil" { Expression::Unit }

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
pub struct Lambda {
    pub arguments: Vec<Argument>,
    pub body: Script,
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
pub enum Statement {
    If,
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
pub enum Expression {
    Lambda(Lambda),
    Reference(DotExpression),
    Call(CallExpression),
    Tuple(Tuple),
    Number(Number),
    String(String),
    Unit,
}

#[derive(Debug)]
pub struct Script {
    pub statements: Vec<Statement>,
}
impl Script {
    pub fn parse(input: &str) -> Self {
        matra_script::script(input).unwrap()
    }
}
