pub mod ast;

use ast::*;
use std::vec;

peg::parser! {
    grammar saturnus_script() for str {
        pub rule script() -> Script
            = _ statements:statement() ** __ _
            { Script { statements } }

        // Statements
        rule statement() -> Statement
            = e:class() { Statement::Class(e) }
            / e:func() { Statement::Function(e) }
            / e:for_each() { Statement::For(e) }
            / e:while_loop() { Statement::While(e) }
            / e:loop_loop() {  Statement::Loop(e) }
            / e:if_stmt() { Statement::If(e) }
            / e:declare_var() { Statement::Let(e) }
            / e:assignment() { Statement::Assignment(e) }
            / e:return_stmt() { Statement::Return(e) }
            / e:expression() _ EOS() { Statement::Expression(e) }

        rule if_stmt() -> If
            = "if" __ condition:expression() __ "{" body:script()
              branches:("}" _ "else" __ "if" __ c:expression() __ "{" s:script() { (c, s) })*
              else_branch:("}" _ "else" _ "{" e:script() {e})?
              "}"
            { If { condition, body, branches, else_branch } }

        rule for_each() -> For
            = "for" __ handler:identifier() __ "in" __ target:expression() _ "{"
              body:script() "}"
            { For { handler, target, body } }

        rule while_loop() -> While
            = "while" __ c:expression() _ "{" body:script() "}"
            { While { condition: ExpressionOrLet::Expression(c), body } }
            / "while" __ c:let_expression() _ "{" body:script() "}"
            { While { condition: ExpressionOrLet::Let(c), body } }

        rule loop_loop() -> Loop
            = "loop" _ "{" body:script() "}"
            { Loop { body } }

        rule func() -> Function
            = decorators:decorator_list() FN() __ name:identifier() _ arguments:argument_list() _ "{" body:script() "}"
            { Function { name, decorators, body, arguments } }
            / decorators:decorator_list() FN() __ name:identifier() _ arguments:argument_list() _ "{" _ "}"
            { Function { name, decorators, body: Script { statements: vec![] }, arguments } }

        rule class() -> Class
            = decorators:decorator_list() CLASS()
              __ name:identifier() _ "{"
              fields:(_ f:class_fields() _ {f})*
              _ "}"
            { Class { name, fields, decorators } }

        rule declare_var() -> Let
            = e:let_expression() _ EOS() { e }

        rule assignment() -> Assignment
            = target:dot_expr() _ extra:assign_extra()? "=" _ value:expression() _ EOS()
            { Assignment { target, value, extra } }

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
            left:(@) _ "++" _ right:@ { BinaryExpression { left, right, operator: Operator::Concat }.into() }
            left:(@) _ ".." _ right:@ { BinaryExpression { left, right, operator: Operator::Range }.into() }

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
            left:(@) _ "<?" _ right:@ { BinaryExpression { left, right, operator: Operator::AskRight }.into() }
            left:(@) _ "?>" _ right:@ { BinaryExpression { left, right, operator: Operator::AskLeft }.into() }
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
            e:vector_expr() { Expression::Vector(e) }
            e:table_expr() { Expression::Table(e) }
            e:tuple_expr() { Expression::Tuple(e) }
            "(" _ e:expression() _ ")" { e }
        }

        rule dot_expr() -> DotExpression
            = value:dot_segment() ++ (_ "." _) { DotExpression(value) }

        rule dot_segment() -> DotSegment
            = e:identifier() { DotSegment::Identifier(e) }
            / e:("[" _ e:expression() _ "]" {e}) { DotSegment::Expression(e) }

        rule lambda() -> Lambda
            = FN() _ arguments:argument_list() _ "{" body:script() "}"
            { Lambda { arguments, body: ScriptOrExpression::Script(body) } }
            / FN() _ arguments:argument_list() _ ":" _ expr:expression()
            { Lambda { arguments, body: ScriptOrExpression::Expression(expr) } }
            / FN() _ arguments:argument_list() _ "{" _ "}"
            { Lambda { arguments, body: ScriptOrExpression::Script(Script { statements: vec![] }) } }

        rule call_expr() -> CallExpression
            = target:dot_expr() static_target:(_ "::" _ e:identifier(){e})? _ arguments:wrapped_comma_expr()
            { CallExpression { target, static_target, arguments } }
            / target:dot_expr() static_target:(_ "::" _ e:identifier(){e})? _ arg:table_expr()
            { CallExpression { target, static_target, arguments: vec![Expression::Table(arg)] } }
            / target:dot_expr() static_target:(_ "::" _ e:identifier(){e})? _ arg:vector_expr()
            { CallExpression { target, static_target, arguments: vec![Expression::Vector(arg)] } }
            / target:dot_expr() static_target:(_ "::" _ e:identifier(){e})? _ arg:string()
            { CallExpression { target, static_target, arguments: vec![Expression::String(arg)] } }

        // Literals
        rule number() -> Number
            = value:$(DIGIT()+ "." DIGIT()+) { Number::Float(value.parse().unwrap()) }
            / value:$(DIGIT()+) { Number::Integer(value.parse().unwrap()) }

        rule string() -> String
            = "\"" value:$((!"\"" ANY())*) "\"" { value.into() }
            / "'" value:$((!"'" ANY())*) "'" { value.into() }

        rule vector_expr() -> Vector
            = "[" _ expressions:comma_expr() _ "]"
            { Vector { expressions } }

        rule table_expr() -> Table
            = "{" _ key_values:table_kvs() _ "}"
            { Table { key_values } }

        // Auxiliaries and sub-expressions
        rule let_expression() -> Let
            = "let" __ target:identifier() value:(_ "=" _ e:expression(){e})?
            { Let { target, value } }

        rule class_fields() -> ClassField
            = e:declare_var() { ClassField::Let(e) }
            / e:func() { ClassField::Method(e) }
            / "operator" _ operator:any_operator() _ arguments:argument_list() _ "{" body:script() "}"
            { ClassField::Operator(OperatorOverload { operator, arguments, body }) }

        rule assign_extra() -> Operator
            = "+" { Operator::Plus }
            / "-" { Operator::Minus }
            / "*" { Operator::Product }
            / "/" { Operator::Quotient }
            / "++" { Operator::Concat }

        rule argument_list() -> Vec<Argument>
            = "(" _ args:argument() ** (_ "," _) _ ")" { args }

        rule argument() -> Argument
            = decorators:decorator_list() name:identifier()
            { Argument { name, decorators } }

        rule decorator_list() -> Vec<Decorator>
            = e:decorator() ++ __ __ { e }
            / { vec![] }

        rule decorator() -> Decorator
            = "@" _ e:call_expr() { Decorator { target: Expression::Call(e) } }
            / "@" _ e:dot_expr() { Decorator { target: Expression::Reference(e) } }

        rule identifier() -> Identifier
            = value:$(IDENT())
            { Identifier(value.into()) }

        rule wrapped_comma_expr() -> Vec<Expression>
            = "(" _ e:comma_expr() _ ")" { e }

        rule comma_expr() -> Vec<Expression>
            = e:expression() ** (_ "," _) { e }

        rule table_kvs() -> Vec<(TableKeyExpression, Expression)>
            = kv:table_kv_pair() ** (_ "," _)
            { kv }

        rule table_kv_pair() -> (TableKeyExpression, Expression)
            = k:identifier() _ ":" _ v:expression()
            { (TableKeyExpression::Identifier(k), v) }
            / "[" _ k:expression() _ "]" _ ":" _ v:expression()
            { (TableKeyExpression::Expression(k), v) }
            / k:identifier()
            { (
                TableKeyExpression::Implicit(k.clone()),
                Expression::Reference(DotExpression(vec![DotSegment::Identifier(k)]))
            ) }

        rule tuple_expr() -> Tuple
            = "(" _ e:expression() **<2,> (_ "," _) _ ")"
            { Tuple(e) }

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
        rule WS() = BLANK() / LINE_COMMENT() / BLOCK_COMMENT() / EOL()
        rule LINE_COMMENT() = "//" (!EOL() ANY())* EOL()
        rule BLOCK_COMMENT() = "/*" (!"*/" ANY())* "*/"
        rule EOL() = ['\r'|'\n']
        rule EOS() = EOL() / ";"
        rule ALPHA() = ['A'..='Z'|'a'..='z'|'_']
        rule DIGIT() = ['0'..='9']
        rule _ = WS()*
        rule __ = WS()+

        // Special matching rule: Any Binary Operator
        rule any_operator() -> Operator
            = "++" { Operator::Concat }
            / ".." { Operator::Range }
            / "+" { Operator::Plus }
            / "-" { Operator::Minus }
            / "*" { Operator::Product }
            / "/" { Operator::Quotient }
            / "**" { Operator::Power }
            / "%" { Operator::Remainder }
            / ">=<" { Operator::Funnel }
            / ">=" { Operator::GreaterEqual }
            / "<=>" { Operator::Starship }
            / "<=" { Operator::LessEqual }
            / "<>" { Operator::NotEqual }
            / "==" { Operator::Equal }
            / "and" { Operator::LogicAnd }
            / "or" { Operator::LogicOr }
            / "xor" { Operator::LogicXOr }
            / "nand" { Operator::LogicNand }
            / "nor" { Operator::LogicNor }
            / "<~>" { Operator::Elastic }
            / "<~" { Operator::ElasticLeft }
            / "~>" { Operator::ElasticRight }
            / "<:>" { Operator::PinguBoth }
            / "<:" { Operator::PinguLeft }
            / ":>" { Operator::PinguRight }
            / "<-|->" { Operator::ArrowStandBoth }
            / "<-|" { Operator::ArrowStandLeft }
            / "|->" { Operator::ArrowStandRight }
            / "<->" { Operator::BothWays }
            / "<-" { Operator::ArrowLeft }
            / "->" { Operator::ArrowRight }
            / "<|>" { Operator::Disjoin }
            / "<|" { Operator::PipeLeft }
            / "|>" { Operator::PipeRight }
            / "?>" { Operator::AskRight }
            / "<?" { Operator::AskLeft }
            / "?:" { Operator::Elvis }
            / "??" { Operator::Coalesce }
            / ">" { Operator::Greater }
            / "<" { Operator::Less }
        }
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

#[derive(Debug, Clone)]
pub struct Script {
    pub statements: Vec<Statement>,
}
impl Script {
    pub fn parse<I>(input: I) -> Result<Self, ParseFailure>
    where
        I: Into<String>,
    {
        let fragment: String = input.into();
        saturnus_script::script(&fragment).map_err(|e| ParseFailure::new(e, &fragment))
    }

    // This function is used only in tests for now.
    #[cfg(test)]
    pub fn parse_expression<I>(input: I) -> Result<Expression, ParseFailure>
    where
        I: Into<String>,
    {
        let fragment: String = input.into();
        saturnus_script::expression(&fragment).map_err(|e| ParseFailure::new(e, &fragment))
    }
}
