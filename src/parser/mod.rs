pub mod ast;

use ast::*;

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

        rule macro_call() -> ()
            = target:identifier() "!"
            {  }

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
            = target:member_expression() _ extra:assign_extra()? "=" _ value:expression() _ EOS()
            { Assignment { target, value, extra } }

        rule return_stmt() -> Return
            = "return" __ value:expression() _ EOS()
            { Return { value } }

        // Expressions
        pub rule expression() -> Expression
            = call_expression()
            / binary_expression()

        rule call_expression() -> Expression
            = target:member_expression() _ "(" _ arguments() _ ")"
            { Expression::Call(CallExpression { target, static_target: None, arguments: vec![] }) }

        rule primary() -> Expression
            = string_expression()
            / number_expression()
            / table_expression()
            / vector_expression()
            / tuple_expression()
            / enclosed_expression()

        rule member_expression() -> MemberExpression
            = primary() (_ "[" _ expression() _ "]")+ { MemberExpression(vec![]) }
            / primary() (_ "." _ primary())+ { MemberExpression(vec![]) }
            / e:identifier() { MemberExpression(vec![MemberSegment::Identifier(e)]) }

        rule arguments() = "a"

        rule binary_expression() -> Expression = precedence! {
            "-" _ expression:@ { UnaryExpression { expression, operator: Operator::Minus }.into() }
            "+" _ expression:@ { UnaryExpression { expression, operator: Operator::Plus }.into() }
            "#?" _ expression:@ { UnaryExpression { expression, operator: Operator::Count }.into() }
            "not" _ expression:@ { UnaryExpression { expression, operator: Operator::LogicNot }.into() }
            "~^" _ expression:@ { UnaryExpression { expression, operator: Operator::BWiseNot }.into() }
            // "!" _ expression:@ { UnaryExpression { expression, operator: Operator::Exclamation }.into() }
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
            left:(@) _ "&" _ right:@ { BinaryExpression { left, right, operator: Operator::BWiseAnd }.into() }
            left:(@) _ "|" _ right:@ { BinaryExpression { left, right, operator: Operator::BWiseOr }.into() }
            left:(@) _ "<<<" _ right:@ { BinaryExpression { left, right, operator: Operator::BWiseLShiftRoundtrip }.into() }
            left:(@) _ "<<" _ right:@ { BinaryExpression { left, right, operator: Operator::BWiseLShift }.into() }
            left:(@) _ ">>>" _ right:@ { BinaryExpression { left, right, operator: Operator::BWiseRShiftRoundtrip }.into() }
            left:(@) _ ">>" _ right:@ { BinaryExpression { left, right, operator: Operator::BWiseRShift }.into() }
            // Extra logic:
            // left:(@) _ "^" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicXOr }.into() }
            // left:(@) _ "¬&" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicNand }.into() }
            // left:(@) _ "¬|" _ right:@ { BinaryExpression { left, right, operator: Operator::LogicNor }.into() }
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
            e:atom() { e }
        }

        rule atom() -> Expression
            = string_expression()
            / number_expression()
            / lambda_expression()
            / vector_expression()
            / table_expression()
            / tuple_expression()
            / e:member_expression() { Expression::Reference(e) }
            / unit() { Expression::Unit }
            / enclosed_expression()

        // Literal-to-expression
        rule string_expression() -> Expression = e:string_literal() { Expression::String(e) }
        rule number_expression() -> Expression = e:number_literal() { Expression::Number(e) }
        rule lambda_expression() -> Expression = e:lambda_literal() { Expression::Lambda(Box::new(e)) }
        rule vector_expression() -> Expression = e:vector_literal() { Expression::Vector(e) }
        rule table_expression() -> Expression = e:table_literal() { Expression::Table(e) }
        rule tuple_expression() -> Expression = e:tuple_literal() { Expression::Tuple(e) }

        rule enclosed_expression() -> Expression
            = "(" _ e:expression() _ ")" { Expression::Tuple1(Box::new(e)) }

        rule lambda_literal() -> Lambda
            = FN() _ arguments:argument_list() _ "{" body:script() "}"
            { Lambda { arguments, body: ScriptOrExpression::Script(body) } }
            / FN() _ arguments:argument_list() _ ":" _ expr:expression()
            { Lambda { arguments, body: ScriptOrExpression::Expression(expr) } }
            / FN() _ arguments:argument_list() _ "{" _ "}"
            { Lambda { arguments, body: ScriptOrExpression::Script(Script { statements: vec![] }) } }

        // Literals
        rule number_literal() -> Number
            = value:$(DIGIT()+ "." DIGIT()+) { Number::Float(value.parse().unwrap()) }
            / value:$(DIGIT()+) { Number::Integer(value.parse().unwrap()) }

        rule string_literal() -> String
            = "\"" value:$((!"\"" ANY())*) "\"" { value.into() }
            / "'" value:$((!"'" ANY())*) "'" { value.into() }

        rule vector_literal() -> Vector
            = "[" _ expressions:comma_expr() _ "]"
            { Vector { expressions } }

        rule table_literal() -> Table
            = "{" _ key_values:table_kvs() _ "}"
            { Table { key_values } }

        rule tuple_literal() -> Tuple
            = "(" _ e:expression() **<2,> (_ "," _) _ ")"
            { Tuple(e) }

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
            = "@" _ target:call_expression() { Decorator { target } }

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
                Expression::Reference(MemberExpression(vec![]))
            ) }

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

pub type ParseResult = Result<Script, peg::error::ParseError<peg::str::LineCol>>;

#[derive(Debug, Clone)]
pub struct Script {
    pub statements: Vec<Statement>,
}
impl Script {
    pub fn parse<I>(input: I) -> ParseResult
    where
        I: Into<String>,
    {
        let fragment: String = input.into();
        saturnus_script::script(&fragment)
    }

    // This function is used only in tests for now.
    #[cfg(test)]
    pub fn parse_expression<I>(
        input: I,
    ) -> Result<ast::Expression, peg::error::ParseError<peg::str::LineCol>>
    where
        I: Into<String>,
    {
        let fragment: String = input.into();
        saturnus_script::expression(&fragment)
    }
}
