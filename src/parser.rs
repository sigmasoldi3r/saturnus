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
          = dec:(e:decorator_list() __ {e})?  FN() __ name:identifier() _ "()" body:script() END()
          { Function { name, decorators: dec.unwrap_or(vec![]), body } }

      rule class() -> Class
          = dec:(e:decorator_list() __ {e})? CLASS() __ name:identifier() __ END()
          { Class { name, decorators: dec.unwrap_or(vec![]) } }

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
          / e:call_expr()  { Expression::Call(e) }
          / e:dot_expr() { Expression::Reference(e) }
          / e:tuple_expr() { Expression::Tuple(e) }
          / "(" _ e:expression() _ ")" { e }

      rule dot_expr() -> DotExpression
          = value:identifier() ++ (_ "." _) { DotExpression(value) }

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
      rule decorator_list() -> Vec<Decorator>
          = e:decorator() ** _ { e }

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
pub struct Function {
    pub name: Identifier,
    pub decorators: Vec<Decorator>,
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
