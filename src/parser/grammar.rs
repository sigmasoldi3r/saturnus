use super::ast::*;
use super::helpers::generate_operator_function_name;

peg::parser! {
    grammar saturnus_script() for str {
        pub rule script() -> Script
            = _ statements:statement() ** __ _
            { Script { statements } }

        // Statements
        rule statement() -> Statement
            = e:macro_decorator() { Statement::MacroDecorator(Box::new(e)) }
            / e:use_statement() { Statement::UseStatement(e) }
            / e:class() { Statement::Class(e) }
            / e:func() { Statement::Function(e) }
            / e:extern_block() { Statement::Extern(e) }
            / e:for_each() { Statement::For(e) }
            / e:while_loop() { Statement::While(e) }
            / e:loop_loop() {  Statement::Loop(e) }
            / e:if_stmt() { Statement::If(e) }
            / e:declare_var() { Statement::Let(e) }
            / e:assignment() { Statement::Assignment(e) }
            / e:return_stmt() { Statement::Return(e) }
            / e:do_expression() { Statement::Expression(e) }
            / e:expression() _ EOS() { Statement::Expression(e) }

        rule macro_decorator() -> MacroDecorator
            = "#[" _ macros:identifier_or_call() ++ (_ "," _) _ "]"
            _ target:statement()
            {
                MacroDecorator {
                    macros,
                    target
                }
            }

        rule identifier_or_call() -> IdentifierOrCall
            = e:call_expression() { IdentifierOrCall::Call(e) }
            / e:identifier() { IdentifierOrCall::Identifier(e) }

        rule if_stmt() -> If
            = "if" __ condition:expression() __ "{" body:script()
              branches:("}" _ "else" __ "if" __ c:expression() __ "{" s:script() { (c, s) })*
              else_branch:("}" _ "else" _ "{" e:script() {e})?
              "}"
            { If { condition, body, branches, else_branch } }
            / expected!("If statement")

        rule for_each() -> For
            = "for" __ handler:assignment_target() __ "in" __ target:expression() _ "{"
              body:script() "}"
            { For { handler, target, body } }
            / expected!("For loop")

        rule while_loop() -> While
            = "while" __ c:expression() _ "{" body:script() "}"
            { While { condition: ExpressionOrLet::Expression(c), body } }
            / "while" __ c:let_expression() _ "{" body:script() "}"
            { While { condition: ExpressionOrLet::Let(c), body } }
            / expected!("While loop")

        rule loop_loop() -> Loop
            = "loop" _ "{" body:script() "}"
            { Loop { body } }
            / expected!("Loop")

        rule func() -> Function
            = decorators:decorator_list() FN() __ name:identifier() _ arguments:argument_list() _ body:func_body()
            { Function { name, decorators, body, arguments } }
            / expected!("Function declaration")

        rule func_body() -> Script
            = "{" body:script() "}" { body }
            / "{" _ "}" { Script { statements: vec![] } }
            / "=" _ value:expression() _ EOS() { Script { statements: vec![Statement::Return(Return { value })] } }

        rule class() -> Class
            = decorators:decorator_list() CLASS()
              __ name:identifier() _ "{"
              fields:(_ f:class_fields() _ {f})*
              _ "}"
            { Class { name, fields, decorators } }
            / expected!("Class declaration")

        rule declare_var() -> Let
            = e:let_expression() _ EOS() { e }
            / expected!("Variable declaration")

        rule extern_block() -> Extern
            = "<extern" _ id:string_literal() ">" src:$(!"</extern>" ANY())* "</extern>"
            { Extern { id: id.0, src: src.join("") } }

        rule assignment() -> Assignment
            = target:member_expression() _ extra:extra_operator()? "=" _ value:expression() _ EOS()
            { Assignment { target, value, extra } }

        rule extra_operator() -> Operator
            = value:$("++" / "+" / "-" / "*" / "/")
            { Operator(value.to_owned()) }

        rule return_stmt() -> Return
            = "return" __ value:expression() _ EOS()
            { Return { value } }

        rule use_segment() -> String
            = value:identifier() { value.0 }
            / value:string_literal() { value.0 }

        rule use_target() -> Vec<String>
            = value:use_segment() ++ (_ "." _) { value }

        rule use_statement() -> UseStatement
            = USE() _ "{" _ targets:(name:identifier() ** (_ "," _) { name }) _ "}" _ "in" __ module:use_target() _ EOS()
            { UseStatement { module, expanded: Some(targets) } }
            / USE() __ module:use_target() _ EOS()
            { UseStatement { module, expanded: None } }

        // Expressions
        pub rule expression() -> Expression
            = binary_expression()

        rule member_expression() -> MemberExpression
            = head:primary()
            tail:(
                _ "[" _ e:expression() _ "]" { MemberSegment::Computed(e) }
                / _ "." _ i:identifier() { MemberSegment::Identifier(i) }
            )*
            { MemberExpression { head, tail } }

        rule call_expression() -> CallExpression
            = head:(
                callee:member_expression() _ arguments:call_arguments()
                { CallSubExpression { callee: Some(callee), arguments }.into() }
                / callee:member_expression() _ arg:table_expression()
                { CallSubExpression { callee: Some(callee), arguments: vec![arg] } }
            )
            tail:(
                  _ "[" _ prop:expression() _ "]" { MemberSegment::Computed(prop).into() }
                / _ "." _ prop:identifier() { MemberSegment::Identifier(prop).into() }
                / _ "->" _ prop:identifier() { MemberSegment::Dispatch(prop).into() }
                / _ arguments:call_arguments() { CallSubExpression { callee: None, arguments }.into() }
            )*
            { CallExpression { head, tail } }

        rule primary() -> Expression
            = i:identifier() { Expression::Identifier(i) }
            / string_expression()
            / number_expression()
            / table_expression()
            / vector_expression()
            / tuple_expression()
            / lambda_expression()
            / do_expression()
            / enclosed_expression()

        rule call_arguments() -> Vec<Expression>
            = "(" _ args:(e:call_argument_list() _ { e })? ")"
            { args.unwrap_or(vec![]) }

        rule call_argument_list() -> Vec<Expression>
            = args:expression() ** (_ "," _)
            { args }

        rule binary_expression() -> Expression = precedence! {
            "..." _ expression:@ { Expression::Spread(Box::new(SpreadExpression { expression })) }
            --
            value:$("-") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            // value:$("+") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            value:$("#?") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            value:$("not") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            // value:$("~^") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            // value:$("!" _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            // value:$("~") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            // value:$("¬" _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            // value:$("$") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            // value:$("!?") _ expression:@ { UnaryExpression { expression, operator: Operator(value.into()) }.into() }
            --
            left:(@) _ value:$("++") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("..") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }

            left:(@) _ value:$("+") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            --
            left:(@) _ value:$("*") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("/") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            --
            left:@ _ value:$("**") _ right:(@) { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            --
            left:(@) _ value:$("%") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            --
            left:(@) _ value:$(">=<" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$(">=" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$(">" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("<=>" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("<=" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("<>" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("<" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("==" !"-") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            --
            left:(@) _ value:$("and") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("or") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("xor") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("nand") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("nor") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            --
            left:(@) _ value:$("&") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("|") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("<<<") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$("<<") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$(">>>") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            left:(@) _ value:$(">>") _ right:@ { BinaryExpression { left, right, operator: Operator(value.into()) }.into() }
            --
            // Extra logic:
            left:(@) _ value:$(ANY_OPERATOR()) _ right:@ {
                BinaryExpression { left, right, operator: Operator(value.into()) }.into()
            }
            --
            e:atom() { e }
        }

        rule atom() -> Expression
            = e:call_expression() { Expression::Call(Box::new(e)) }
            / lambda_expression()
            / string_expression()
            / number_expression()
            / vector_expression()
            / table_expression()
            / tuple_expression()
            / do_expression()
            / e:member_expression() { Expression::Reference(Box::new(e)) }
            / unit() { Expression::Unit }
            / enclosed_expression()

        // Literal-to-expression
        rule string_expression() -> Expression = e:string_literal() { Expression::String(e) }
        rule number_expression() -> Expression = e:number_literal() { Expression::Number(e) }
        rule lambda_expression() -> Expression = e:lambda_literal() { Expression::Lambda(Box::new(e)) }
        rule vector_expression() -> Expression = e:vector_literal() { Expression::Vector(e) }
        rule table_expression() -> Expression = e:table_literal() { Expression::Table(e) }
        rule tuple_expression() -> Expression = e:tuple_literal() { Expression::Tuple(e) }
        rule do_expression() -> Expression = e:do_literal() { Expression::Do(e) }

        rule enclosed_expression() -> Expression
            = "(" _ e:expression() _ ")" { Expression::Tuple1(Box::new(e)) }

        rule lambda_literal() -> Lambda
            = name:identifier() _ "=>" _ "{" body:script() "}"
            { Lambda { arguments: vec![Argument { name, decorators: vec![], spread: false }], body: ScriptOrExpression::Script(body) } }
            / name:identifier() _ "=>" _ body:expression()
            { Lambda { arguments: vec![Argument { name, decorators: vec![], spread: false }], body: ScriptOrExpression::Expression(body) } }
            / arguments:argument_list() _ "=>" _ "{" body:script() "}"
            { Lambda { arguments, body: ScriptOrExpression::Script(body) } }
            / arguments:argument_list() _ "=>" _ expr:expression()
            { Lambda { arguments, body: ScriptOrExpression::Expression(expr) } }
            / arguments:argument_list() _ "=>" _ "{" _ "}"
            { Lambda { arguments, body: ScriptOrExpression::Script(Script { statements: vec![] }) } }

        // Literals
        rule number_literal() -> Number
            = value:$(DIGIT()+ "." DIGIT()+) { Number::Float(value.parse().unwrap()) }
            / value:$(DIGIT()+) { Number::Integer(value.parse().unwrap()) }
            / "'" value:$(!"'" ANY()) "'" { Number::Integer(value.chars().nth(0).unwrap() as i64) }
            / expected!("Number literal")

        rule string_literal() -> StringLiteral
            = "\"" value:$(
                ( "\\\"" / (!"\"" ANY()) )*
            ) "\"" { StringLiteral(value.into()) }
            / expected!("String literal")

        rule vector_literal() -> Vector
            = "[" _ expressions:comma_expr() _ "]"
            { Vector { expressions } }
            / expected!("Vector literal")

        rule table_literal() -> Table
            = "{" _ key_values:table_kvs() _ "}"
            { Table { key_values } }
            / expected!("Table literal")

        rule tuple_literal() -> Tuple
            = "(" _ e:expression() **<2,> (_ "," _) _ ")"
            { Tuple(e) }
            / expected!("Tuple literal")

        rule do_literal() -> Do
            = "{" body:script() "}"
            { Do { body } }
            / expected!("Do block")

        // Auxiliaries and sub-expressions
        rule let_expression() -> Let
            = "let" __ target:assignment_target() value:(_ "=" _ e:expression(){e})?
            { Let { target, value } }

        rule assignment_target() -> AssignmentTarget
            = e:identifier() { AssignmentTarget::Identifier(e) }
            / e:destructure_expression() { AssignmentTarget::Destructuring(e) }

        rule destructure_expression() -> Destructuring
            = "{" _ targets:(name:identifier() ** (_ "," _) { name }) _ "}"
                { Destructuring(targets, DestructureOrigin::Table) }
            / "(" _ targets:(name:identifier() ** (_ "," _) { name }) _ ")"
                { Destructuring(targets, DestructureOrigin::Tuple) }
            / "[" _ targets:(name:identifier() ** (_ "," _) { name }) _ "]"
                { Destructuring(targets, DestructureOrigin::Array) }

        rule class_fields() -> ClassField
            = e:declare_var() { ClassField::Let(e) }
            / e:func() { ClassField::Method(e) }

        rule argument_list() -> Vec<Argument>
            = "(" _ args:argument() ** (_ "," _) _ ")" { args }

        rule argument() -> Argument
            = decorators:decorator_list() spread:(_ "*" _)? name:identifier()
            { Argument { name, decorators, spread: spread.is_some() } }

        rule decorator_list() -> Vec<Decorator>
            = e:decorator() ++ _ _ { e }
            / { vec![] }

        rule decorator() -> Decorator
            = "@" _ target:call_expression() { Decorator { target } }
            / expected!("Decorator")

        rule identifier() -> Identifier
            = "`" value:$(ANY_OPERATOR()) "`"
            { Identifier(generate_operator_function_name(value.to_owned()))
            }
            / value:$(IDENT()) { Identifier(value.into()) }
            / expected!("Identifier")

        rule wrapped_comma_expr() -> Vec<Expression>
            = "(" _ e:comma_expr() _ ")" { e }

        rule comma_expr() -> Vec<Expression>
            = e:expression() ** (_ "," _) (_ "," _)? { e }

        rule table_kvs() -> Vec<(TableKeyExpression, Option<Expression>)>
            = kv:table_kv_pair() ** (_ "," _) (_ "," _)?
            { kv }

        rule table_kv_pair() -> (TableKeyExpression, Option<Expression>)
            = k:identifier() _ ":" _ v:expression()
            { (TableKeyExpression::Identifier(k), Some(v)) }
            / k:string_expression() _ ":" _ v:expression()
            { (TableKeyExpression::Expression(k), Some(v)) }
            / "[" _ k:expression() _ "]" _ ":" _ v:expression()
            { (TableKeyExpression::Expression(k), Some(v)) }
            / k:identifier()
            { (
                TableKeyExpression::Implicit(k.clone()),
                None
            ) }

        rule unit() -> Expression = "()" { Expression::Unit }

        // Tokens
        rule IDENT() = ALPHA() (ALPHA() / DIGIT())*
        rule LET() = "let"
        rule MUT() = "mut"
        rule CLASS() = "class"
        rule END() = "end"
        rule USE() = "use"
        rule FN() = "fn"
        rule EXTERN() = "extern"
        rule NATIVE() = "native" // Deprecated!
        rule MACRO() = "macro"
        rule ANY_OPERATOR() = ['^'|'+'|'-'|'*'|'/'|'.'|'|'|'>'|'<'|'?'|'!'|'~'|'%'|'&'|'#'|'$'|':'|'=']+ !"="
        rule ANY() = quiet!{ [_] } / expected!("Any character")
        rule BLANK() = ['\t'|' '] / expected!("White space")
        rule WS() = BLANK() / LINE_COMMENT() / BLOCK_COMMENT() / EOL()
        rule LINE_COMMENT() = quiet!{ "//" (!EOL() ANY())* EOL() } / expected!("Line comment")
        rule BLOCK_COMMENT() = quiet!{ "/*" (!"*/" ANY())* "*/" } / expected!("Block comment")
        rule EOL() = quiet!{ ['\r'|'\n'] } / expected!("End of line")
        rule EOS() = quiet!{ ";" } / expected!("End of statement") //quiet!{ EOL() / ";" } / expected!("End of statement")
        rule ALPHA() = quiet!{ ['A'..='Z'|'a'..='z'|'_'] } / expected!("Alphanumeric")
        rule DIGIT() = quiet!{ ['0'..='9'] } / expected!("Digit")
        rule _ = WS()*
        rule __ = WS()+

        // Special matching rule: Any Binary Operator
        rule any_operator() -> Operator
            = value:$("++") { Operator(value.into()) }
            / value:$("..") { Operator(value.into()) }
            / value:$("+") { Operator(value.into()) }
            / value:$("-") { Operator(value.into()) }
            / value:$("*") { Operator(value.into()) }
            / value:$("/") { Operator(value.into()) }
            / value:$("**") { Operator(value.into()) }
            / value:$("%") { Operator(value.into()) }
            / value:$(">=<") { Operator(value.into()) }
            / value:$(">=") { Operator(value.into()) }
            / value:$("<=>") { Operator(value.into()) }
            / value:$("<=") { Operator(value.into()) }
            / value:$("<>") { Operator(value.into()) }
            / value:$("==") { Operator(value.into()) }
            / value:$("and") { Operator(value.into()) }
            / value:$("or") { Operator(value.into()) }
            / value:$("xor") { Operator(value.into()) }
            / value:$("nand") { Operator(value.into()) }
            / value:$("nor") { Operator(value.into()) }
            / value:$(['^'|'+'|'-'|'*'|'/'|'.'|'|'|'>'|'<'|'='|'?'|'!'|'~'|'%'|'&'|'#'|'$'|':']+) { Operator(value.into()) }
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
    ) -> Result<Expression, peg::error::ParseError<peg::str::LineCol>>
    where
        I: Into<String>,
    {
        let fragment: String = input.into();
        saturnus_script::expression(&fragment)
    }
}
