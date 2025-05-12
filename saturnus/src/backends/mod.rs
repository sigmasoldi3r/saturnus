use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    code::IndentedBuilder,
    compiler::{Compiler, CompilerError, CompilerOptions, ModuleType, Result},
    parsing::{
        ast::{
            ArrayAccess, ArrayLiteral, Assignment, AssignmentTarget, Boolean, Bop, Call, ClassDef,
            ClassField, DefModifiers, Destructure, DestructureEntry, ElseIf, Expr, Fn, For,
            Identifier, IfStatement, IntoAssignmentTarget, IntoExpr, IntoMapKey, IntoStatement,
            LambdaExpr, Let, Loop, MapKey, MapLiteral, Member, MemberOp, Number, Operator, Param,
            Return, SatString, Statement, TupleLiteral, Uop, Use, While,
        },
        builders::{AddArrayAccess, AddMember, LeafCollector},
        grammar::ProgramParser,
    },
    source::{SaturnusIR, SourceCode},
};

pub struct LuaCompiler {
    code: IndentedBuilder,
    options: CompilerOptions,
    module_root_expr: Expr,
}
impl LuaCompiler {
    pub fn new() -> Self {
        Self {
            module_root_expr: Identifier::new("__modules__", false),
            code: IndentedBuilder::new(),
            options: Default::default(),
        }
    }
    fn compile_call(&mut self, call: Call) -> Result {
        let Call {
            mut target,
            arguments,
            is_null_safe,
        } = call;
        if let Expr::Member(member) = &mut *target {
            if let MemberOp::Member = member.op {
                member.op = MemberOp::Dispatch;
            }
        }
        if is_null_safe {
            let original = target.clone();
            target = Box::new(Bop::new(
                Bop::new(
                    *original.clone(),
                    Operator::Neq,
                    TupleLiteral::unit().into_expr(),
                ),
                Operator::And,
                *original,
            ));
        }
        self.compile_expr(*target)?;
        self.code.write("(");
        let mut args = arguments.into_iter();
        if let Some(first) = args.next() {
            self.compile_expr(first)?;
        }
        for expr in args {
            self.code.write(", ");
            self.compile_expr(expr)?;
        }
        self.code.write(")");
        Ok(())
    }
    fn compile_number(&mut self, num: Number) -> Result {
        match num {
            Number::Int(value) => self.code.write(value),
            Number::Float(value) => self.code.write(value),
        };
        Ok(())
    }
    fn translate_identifier(value: String) -> String {
        let mut other = String::new();
        let value = value
            .chars()
            .collect::<Vec<_>>()
            .into_iter()
            .skip(1)
            .rev()
            .skip(1)
            .rev()
            .map(|ch| {
                match ch {
                    ' ' => "space",
                    '+' => "plus",
                    '-' => "minus",
                    '*' => "times",
                    '/' => "divide",
                    '^' => "power",
                    '?' => "question",
                    '¿' => "reverse_question",
                    '!' => "exclamation",
                    '¡' => "reverse_exclamation",
                    '&' => "ampersand",
                    '¬' => "not",
                    '%' => "percent",
                    '$' => "dollar",
                    '#' => "hashbang",
                    '"' => "double_quote",
                    '@' => "at",
                    '|' => "pipe",
                    'º' => "degrees",
                    'ª' => "super_a",
                    '·' => "dot",
                    '.' => "stop",
                    ',' => "comma",
                    ':' => "double_dot",
                    ';' => "semi",
                    '[' => "l_bracket",
                    ']' => "r_bracket",
                    '(' => "l_brace",
                    ')' => "r_brace",
                    '{' => "l_curly",
                    '}' => "r_curly",
                    '=' => "eq",
                    '<' => "lt",
                    '>' => "gt",
                    '\'' => "single_quote",
                    'ç' => "cedilla",
                    'ñ' => "enne",
                    '\\' => "backlash",
                    alt => {
                        other = alt.to_string();
                        other.as_str()
                    }
                }
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("_");
        format!("__{value}__")
    }
    fn compile_identifier(&mut self, ident: Identifier) -> Result {
        let Identifier { value, is_escaped } = ident;
        if is_escaped {
            let value = Self::translate_identifier(value);
            self.code.write(value);
        } else {
            let value = match value.as_str() {
                "then" => "__then__",
                "elseif" => "__elseif__",
                "do" => "__do__",
                "local" => "__local__",
                "end" => "__end__",
                "until" => "__until__",
                "repeat" => "__repeat__",
                _ => value.as_str(),
            };
            self.code.write(value);
        }
        Ok(())
    }
    fn infer_native_operator(op: &Operator) -> Option<String> {
        let op = match op {
            Operator::Add => format!("+"),
            Operator::Sub => format!("-"),
            Operator::Mul => format!("*"),
            Operator::Div => format!("/"),
            Operator::Pow => format!("^"),
            Operator::And => format!("and"),
            Operator::Or => format!("or"),
            Operator::Not => format!("not"),
            Operator::BAnd => format!("&"),
            Operator::BOr => format!("|"),
            Operator::BXor => format!("~"),
            Operator::BNot => format!("~"),
            Operator::LShift => format!("<<"),
            Operator::RShift => format!(">>"),
            Operator::StrCat => format!(".."),
            Operator::Lt => format!("<"),
            Operator::LtEq => format!("<="),
            Operator::Gt => format!(">"),
            Operator::GtEq => format!(">="),
            Operator::Eq => format!("=="),
            Operator::Neq => format!("~="),
            _ => return None,
        };
        Some(op)
    }
    fn compile_custom_operator(
        &mut self,
        value: String,
        left: Box<Expr>,
        right: Option<Box<Expr>>,
    ) -> Result {
        let arguments = match right {
            Some(right) => vec![*left, *right],
            None => vec![*left],
        };
        self.compile_call(Call {
            target: Box::new(Expr::Identifier(Identifier {
                value: format!("`{value}`"),
                is_escaped: true,
            })),
            arguments,
            is_null_safe: false,
        })?;
        Ok(())
    }
    // fn compile_bop_to_call(
    //     &mut self,
    //     call: impl std::fmt::Display,
    //     left: Box<Expr>,
    //     right: Box<Expr>,
    // ) -> Result {
    //     self.code.write(call).write("(");
    //     self.compile_expr(*left)?;
    //     self.code.write(", ");
    //     self.compile_expr(*right)?;
    //     self.code.write(")");
    //     Ok(())
    // }
    fn compile_binary_expr(&mut self, bop: Bop) -> Result {
        let Bop { left, op, right } = bop;
        if let Some(op) = Self::infer_native_operator(&op) {
            self.compile_expr(*left)?;
            self.code.write(" ");
            self.code.write(op);
            self.code.write(" ");
            self.compile_expr(*right)?;
        } else {
            match op {
                Operator::LShiftRot => {
                    self.compile_custom_operator("<<<".into(), left, Some(right))?
                }
                Operator::RShiftRot => {
                    self.compile_custom_operator(">>>".into(), left, Some(right))?
                }
                Operator::Range => self.compile_custom_operator("..".into(), left, Some(right))?,
                Operator::Custom(value) => {
                    self.compile_custom_operator(value, left, Some(right))?
                }
                _ => panic!(
                    "Unhandled operator panic! This shouldn't be reachable, report this bug please."
                ),
            }
        }
        Ok(())
    }
    fn compile_member_access(&mut self, value: Member) -> Result {
        let Member { target, op, field } = value;
        self.compile_expr(*target.clone())?;
        match op {
            MemberOp::Member => {
                self.code.write(".");
            }
            MemberOp::CoalesceMember => {
                self.code.write(" ~= nil and ");
                self.compile_expr(*target)?;
                self.code.write(".");
            }
            MemberOp::Static => {
                self.code.write(".");
            }
            MemberOp::Dispatch => {
                self.code.write(":");
            }
        }
        self.compile_identifier(field)?;
        Ok(())
    }
    fn compile_array_access(&mut self, expr: ArrayAccess) -> Result {
        let ArrayAccess {
            target,
            arguments,
            is_null_safe,
        } = expr;
        self.compile_expr(*target.clone())?;
        if is_null_safe {
            self.code.write(" ~= nil and ");
            self.compile_expr(*target)?;
        }
        for item in arguments {
            self.code.write("[");
            self.compile_expr(item)?;
            self.code.write("]");
        }
        Ok(())
    }
    fn compile_string(&mut self, expr: SatString) -> Result {
        if expr.value.find("\n").is_some() {
            self.code.write("[[").write(expr.value).write("]]");
        } else {
            self.code.write("\"").write(expr.value).write("\"");
        }
        Ok(())
    }
    fn compile_boolean(&mut self, expr: Boolean) -> Result {
        let value = match expr {
            Boolean::True => "true",
            Boolean::False => "false",
        };
        self.code.write(value);
        Ok(())
    }
    fn compile_map_key(&mut self, map_key: MapKey) -> Result {
        match map_key {
            MapKey::Identifier(identifier) => self.compile_identifier(identifier)?,
            MapKey::SatString(value) => {
                self.code.write("[");
                self.compile_string(value)?;
                self.code.write("]");
            }
            MapKey::Expr(expr) => {
                self.code.write("[");
                self.compile_expr(expr)?;
                self.code.write("]");
            }
        }
        Ok(())
    }
    fn compile_map(&mut self, map_literal: MapLiteral) -> Result {
        if self.options.use_std_collections {
            self.code.write("std.Map ");
        }
        self.code.write("{ ");
        let mut iter = map_literal.entries.into_iter();
        if let Some((k, v)) = iter.next() {
            self.compile_map_key(k)?;
            self.code.write(" = ");
            self.compile_expr(v)?;
        }
        for (k, v) in iter {
            self.code.write(format!(", "));
            self.compile_map_key(k)?;
            self.code.write(" = ");
            self.compile_expr(v)?;
        }
        self.code.write(" }");
        Ok(())
    }
    fn compile_array(&mut self, array_literal: ArrayLiteral) -> Result {
        if self.options.use_std_collections {
            self.code.write("std.Array ");
        }
        self.code.write("{ ");
        let mut iter = array_literal.values.into_iter();
        if let Some(first) = iter.next() {
            self.compile_expr(first)?;
        }
        for value in iter {
            self.code.write(format!(", "));
            self.compile_expr(value)?;
        }
        self.code.write(" }");
        Ok(())
    }
    fn compile_tuple(&mut self, tuple_literal: TupleLiteral) -> Result {
        if tuple_literal.is_unit() {
            if self.options.unit_interop {
                self.code.write("nil");
            } else {
                self.code.write("std.Unit()");
            }
            return Ok(());
        }
        if self.options.use_std_collections {
            self.code.write("std.Tuple ");
        }
        self.code.write("{ ");
        let mut iter = tuple_literal.values.into_iter();
        let mut i = 0;
        if let Some(first) = iter.next() {
            self.code.write(format!("__{i} = "));
            self.compile_expr(first)?;
            i += 1;
        }
        for value in iter {
            self.code.write(format!(", __{i} = "));
            self.compile_expr(value)?;
            i += 1;
        }
        self.code.write(" }");
        Ok(())
    }
    fn compile_lambda(&mut self, lambda_expr: LambdaExpr) -> Result {
        self.code.write("function(");
        let mut iter = lambda_expr.params.into_iter();
        if let Some(first) = iter.next() {
            self.compile_identifier(first.name)?;
        }
        for param in iter {
            self.code.write(", ");
            self.compile_identifier(param.name)?;
        }
        self.code.write(")").push();
        self.compile_program(lambda_expr.body)?;
        self.code.pop().line().write("end");
        Ok(())
    }
    fn compile_unary(&mut self, uop: Uop) -> Result {
        let Uop { op, expr } = uop;
        if let Some(op) = Self::infer_native_operator(&op) {
            self.code.write(op);
            self.code.write(" ");
            self.compile_expr(*expr)?;
        } else {
            match op {
                Operator::Custom(value) => self.compile_custom_operator(value, expr, None)?,
                _ => panic!(
                    "Unhandled operator panic! This shouldn't be reachable, report this bug please."
                ),
            }
        }
        Ok(())
    }
    fn compile_expr(&mut self, expr: Expr) -> Result {
        match expr {
            Expr::Call(value) => self.compile_call(value)?,
            Expr::Number(value) => self.compile_number(value)?,
            Expr::Identifier(value) => self.compile_identifier(value)?,
            Expr::Bop(value) => self.compile_binary_expr(value)?,
            Expr::Member(value) => self.compile_member_access(value)?,
            Expr::ArrayAccess(value) => self.compile_array_access(value)?,
            Expr::SatString(value) => self.compile_string(value)?,
            Expr::Boolean(value) => self.compile_boolean(value)?,
            Expr::Uop(uop) => self.compile_unary(uop)?,
            Expr::LambdaExpr(lambda_expr) => self.compile_lambda(lambda_expr)?,
            Expr::MapLiteral(map_literal) => self.compile_map(map_literal)?,
            Expr::ArrayLiteral(array_literal) => self.compile_array(array_literal)?,
            Expr::TupleLiteral(tuple_literal) => self.compile_tuple(tuple_literal)?,
        }
        Ok(())
    }
    fn compile_if(&mut self, stmt: IfStatement) -> Result {
        let IfStatement {
            condition,
            body,
            else_if_blocks,
            else_block,
        } = stmt;
        self.code.write("if ");
        self.compile_expr(*condition)?;
        self.code.write(" then").push();
        self.compile_program(body)?;
        for else_if in else_if_blocks.into_iter() {
            let ElseIf { condition, body } = else_if;
            self.code.pop().line().write("elseif ");
            self.compile_expr(*condition)?;
            self.code.write(" then").push();
            self.compile_program(body)?;
        }
        if let Some(else_block) = else_block {
            self.code.pop().line().write("else").push();
            self.compile_program(else_block)?;
        }
        self.code.pop().line().write("end");
        Ok(())
    }
    fn loop_optimized_for_range(
        &mut self,
        assignment: &Destructure,
        expr: &Box<Expr>,
        body: &Vec<Statement>,
    ) -> std::result::Result<bool, CompilerError> {
        let Expr::Bop(Bop { left, op, right }) = &**expr else {
            return Ok(false);
        };
        if Operator::Range != *op {
            return Ok(false);
        };
        let Destructure::Identifier(var_name) = assignment else {
            return Ok(false);
        };
        self.compile_identifier(var_name.clone())?;
        self.code.write(" = ");
        self.compile_expr((**left).clone())?;
        self.code.write(", ");
        self.compile_expr((**right).clone())?;
        self.code.write(" do").push();
        self.build_loop_body(body.clone())?;
        Ok(true)
    }
    fn loop_optimized_for_pairs_iter(
        &mut self,
        assignment: &mut Destructure,
        expr: &Box<Expr>,
        body: &Vec<Statement>,
    ) -> std::result::Result<bool, CompilerError> {
        let Expr::Call(Call {
            target,
            arguments,
            is_null_safe,
        }) = &**expr
        else {
            return Ok(false);
        };
        if *is_null_safe {
            return Ok(false);
        }
        let Expr::Identifier(fn_name) = &**target else {
            return Ok(false);
        };
        if fn_name.is_escaped || (fn_name.value != "pairs" && fn_name.value != "ipairs") {
            return Ok(false);
        }
        let Destructure::Tuple(elements) = &assignment else {
            return Ok(false);
        };
        if elements.len() != 2 {
            return Ok(false);
        }
        let k = &elements[0];
        let DestructureEntry::Identifier(k) = k else {
            return Ok(false);
        };
        self.compile_identifier(k.clone())?;
        if let DestructureEntry::Identifier(v) = &elements[1] {
            self.code.write(", ");
            self.compile_identifier(v.clone())?;
            self.code.write(" in ");
        } else {
            self.code.write(", __destructure_value__ in ");
        }
        self.compile_call(Call {
            target: target.clone(),
            arguments: arguments.clone(),
            is_null_safe: false,
        })?;
        self.code.write(" do").push();
        self.build_loop_body(body.clone())?;
        Ok(true)
    }
    /// To avoid ambiguities and further code generation complexities, Saturnus offers
    /// assignments only in the form of statements.
    fn compile_assignment(&mut self, stmt: Assignment) -> Result {
        let Assignment { left, right, op } = stmt;
        if let Some(op) = op {
            let right = Bop::new(left.clone().to_expr(), op, *right);
            self.compile_assignment(Assignment::new(left, None, right))?;
        } else {
            match left {
                AssignmentTarget::Member(member) => self.compile_member_access(member)?,
                AssignmentTarget::ArrayAccess(array_access) => {
                    self.compile_array_access(array_access)?
                }
                AssignmentTarget::Identifier(identifier) => self.compile_identifier(identifier)?,
            }
            self.code.write(" = ");
            self.compile_expr(*right)?;
            self.code.write(";");
        }
        Ok(())
    }
    fn compile_array_destructure(&mut self, root: Expr, items: Vec<DestructureEntry>) -> Result {
        let mut i = 0;
        for entry in items {
            i += 1;
            let root = root.clone().array_access(Number::Int(i).into_expr());
            match entry {
                DestructureEntry::Identifier(identifier) => {
                    if identifier.is_void() {
                        continue;
                    }
                    let left = AssignmentTarget::Identifier(identifier.clone());
                    self.compile_statement(Assignment::new(left, None, root).into_statement())?;
                }
                DestructureEntry::Array(items) => self.compile_array_destructure(root, items)?,
                DestructureEntry::Map(items) => self.compile_map_destructure(root, items)?,
                DestructureEntry::Tuple(items) => self.compile_tuple_destructure(root, items)?,
                DestructureEntry::Aliasing(_, _) => panic!(
                    "This branch should not be reachable, there's a problem in the AST. Please report this bug."
                ),
            }
        }
        Ok(())
    }
    fn compile_map_entry(
        &mut self,
        root: Expr,
        item: DestructureEntry,
        skip_member: bool,
    ) -> Result {
        match item {
            DestructureEntry::Identifier(identifier) => {
                if identifier.is_void() {
                    return Ok(());
                }
                let left = AssignmentTarget::Identifier(identifier.clone());
                let root = if skip_member {
                    root
                } else {
                    root.add_member(identifier.clone())
                };
                self.compile_statement(Assignment::new(left, None, root).into_statement())?;
            }
            DestructureEntry::Array(items) => self.compile_array_destructure(root, items)?,
            DestructureEntry::Map(items) => self.compile_map_destructure(root, items)?,
            DestructureEntry::Tuple(items) => self.compile_tuple_destructure(root, items)?,
            DestructureEntry::Aliasing(identifier, destructure_entry) => {
                if identifier.is_void() {
                    return Ok(());
                }
                let root = root.add_member(identifier);
                self.compile_map_entry(root, *destructure_entry, true)?;
            }
        }
        Ok(())
    }
    fn compile_map_destructure(&mut self, root: Expr, items: Vec<DestructureEntry>) -> Result {
        for entry in items {
            let root = root.clone();
            self.compile_map_entry(root, entry, false)?;
        }
        Ok(())
    }
    fn compile_tuple_destructure(&mut self, root: Expr, items: Vec<DestructureEntry>) -> Result {
        let mut i = 0;
        for entry in items {
            let root = root.clone().add_member(Identifier {
                value: format!("__{i}"),
                is_escaped: false,
            });
            i += 1;
            match entry {
                DestructureEntry::Identifier(identifier) => {
                    if identifier.is_void() {
                        continue;
                    }
                    let left = AssignmentTarget::Identifier(identifier.clone());
                    self.compile_statement(Assignment::new(left, None, root).into_statement())?;
                }
                DestructureEntry::Array(items) => self.compile_array_destructure(root, items)?,
                DestructureEntry::Map(items) => self.compile_map_destructure(root, items)?,
                DestructureEntry::Tuple(items) => self.compile_tuple_destructure(root, items)?,
                DestructureEntry::Aliasing(_, _) => panic!(
                    "This branch should not be reachable, there's a problem in the AST. Please report this bug."
                ),
            }
        }
        Ok(())
    }
    /// Compiles expressions like `let [a, b, c] = arr;`
    fn compile_destructure_assignment_list(&mut self, destructure: Destructure) -> Result {
        let root = Identifier::new("__destructure_target__", false);
        match destructure {
            Destructure::Identifier(ident) => {
                self.code.line();
                self.compile_let(Let::new(ident, DefModifiers::new(), root))?;
            }
            Destructure::Array(items) => self.compile_array_destructure(root, items)?,
            Destructure::Map(items) => self.compile_map_destructure(root, items)?,
            Destructure::Tuple(items) => self.compile_tuple_destructure(root, items)?,
        }
        Ok(())
    }
    fn compile_let(&mut self, expr: Let) -> Result {
        let Let {
            name,
            type_def: _,
            initializer,
            modifiers,
        } = expr;
        match name {
            Destructure::Identifier(identifier) => {
                self.code.write("local ");
                self.compile_identifier(identifier.clone())?;
                if let Some(val) = initializer {
                    self.code.write(" = ");
                    self.compile_expr(val)?;
                }
                self.code.write(";");
                self.export_symbol(&modifiers, identifier)?;
            }
            other => {
                self.code.write("local ");
                let mut leaves = other.collect_leaves().into_iter().filter(|x| !x.is_void());
                let leaves_clone = leaves.clone();
                if let Some(first) = leaves.next() {
                    self.compile_identifier(first)?;
                }
                for leaf in leaves {
                    self.code.write(", ");
                    self.compile_identifier(leaf)?;
                }
                self.code
                    .write(";")
                    .line()
                    .write("do")
                    .push()
                    .line()
                    .write("local __destructure_target__ = ");
                self.compile_expr(initializer.expect("Can't destructure without initializer expression. Report this bug (Type checker should sort this out)."))?;
                self.code.write(";");
                self.compile_destructure_assignment_list(other)?;
                self.code.pop().line().write("end");
                for name in leaves_clone {
                    self.export_symbol(&modifiers, name)?;
                }
            }
        }
        Ok(())
    }
    fn build_loop_body(&mut self, body: Vec<Statement>) -> Result {
        self.compile_program(body)?;
        self.code
            .line()
            .write("::loop_end::")
            .pop()
            .line()
            .write("end");
        Ok(())
    }
    fn compile_for(&mut self, stmt: For) -> Result {
        let For {
            mut assignment,
            expr,
            body,
        } = stmt;
        self.code.write("for ");
        // Try to optimize away by removing iterators:
        if self.loop_optimized_for_range(&assignment, &expr, &body)?
            || self.loop_optimized_for_pairs_iter(&mut assignment, &expr, &body)?
        {
            return Ok(());
        }
        // Not optimized, proceed with standard iterator control.
        self.code.write("__destructure_value__ in ");
        self.compile_expr(*expr)?;
        self.code.write(" do").push();
        self.compile_destructure_assignment_list(assignment)?;
        self.build_loop_body(body)?;
        Ok(())
    }
    fn compile_while(&mut self, stmt: While) -> Result {
        let While { condition, body } = stmt;
        self.code.write("while ");
        self.compile_expr(*condition)?;
        self.code.write(" do").push();
        self.compile_program(body)?;
        self.code
            .line()
            .write("::loop_end::")
            .pop()
            .line()
            .write("end");
        Ok(())
    }
    fn compile_loop(&mut self, stmt: Loop) -> Result {
        self.code.write("while true do").push();
        self.compile_program(stmt.body)?;
        self.code
            .line()
            .write("::loop_end::")
            .pop()
            .line()
            .write("end");
        Ok(())
    }
    fn compile_class_def(&mut self, class_def: ClassDef) -> Result {
        let ClassDef {
            name,
            parent,
            fields,
            modifiers,
        } = class_def;
        // Declare the class table
        self.process_pub_symbol(&modifiers)?;
        self.compile_identifier(name.clone())?;
        self.code.write(" = {};").line();
        // Filter out field initializers:
        let methods = fields
            .iter()
            .filter_map(|f| match f {
                ClassField::Fn(fn_def) => Some(fn_def.clone()),
                ClassField::Let(_) => None,
            })
            .collect::<Vec<_>>();
        let fields = fields
            .into_iter()
            .filter_map(|f| match f {
                ClassField::Fn(_) => None,
                ClassField::Let(let_def) => Some(let_def),
            })
            .collect::<Vec<_>>();
        // Expand fields earlier, so important methametods take over precedence (Eg: you declare fn __meta__() or smth).
        for method in methods {
            let Fn {
                name: method,
                modifiers,
                arguments,
                body,
            } = method;
            self.code.line().write("function ");
            self.compile_identifier(name.clone())?;
            if modifiers.is_static() {
                self.code.write(".");
            } else {
                self.code.write(":");
            }
            self.compile_identifier(method)?;
            self.code.write("(");
            let mut iter = arguments.into_iter();
            if let Some(param) = iter.next() {
                let Param { name, .. } = param;
                self.compile_identifier(name)?;
            }
            for param in iter {
                let Param { name, .. } = param;
                self.code.write(", ");
                self.compile_identifier(name)?;
            }
            self.code.write(")").push();
            self.compile_program(body)?;
            self.code.pop().line().write("end");
        }
        // Build the metatable.
        self.code.line();
        self.compile_identifier(name.clone())?;
        self.code.write(".__meta__ = ");
        let access_expr = Call::new(
            Identifier::new("rawget", false),
            vec![
                Identifier::new("self", false),
                Identifier::new("key", false),
            ],
            false,
        );
        let mut index_body = vec![IfStatement::new(
            Bop::new(
                access_expr.clone(),
                Operator::Neq,
                TupleLiteral::unit().into_expr(),
            ),
            vec![Return::new(access_expr.clone())],
            vec![],
            None,
        )];
        if let Some(ident) = parent {
            let access = name
                .clone()
                .into_expr()
                .array_access(Identifier::new("key", false));
            index_body.push(IfStatement::new(
                Bop::new(
                    access.clone(),
                    Operator::Neq,
                    TupleLiteral::unit().into_expr(),
                ),
                vec![Return::new(access)],
                vec![],
                None,
            ));
            index_body.push(Return::new(
                ident
                    .clone()
                    .into_expr()
                    .array_access(Identifier::new("key", false)),
            ));
        } else {
            index_body.push(Return::new(
                name.clone()
                    .into_expr()
                    .array_access(Identifier::new("key", false)),
            ));
        }
        self.compile_map(MapLiteral {
            entries: vec![(
                Identifier::new("__index", false)
                    .unwrap_identifier()
                    .into_mapkey(),
                LambdaExpr::new(
                    vec![
                        Param {
                            name: Identifier::new("self", false).unwrap_identifier(),
                            type_def: None,
                        },
                        Param {
                            name: Identifier::new("key", false).unwrap_identifier(),
                            type_def: None,
                        },
                    ],
                    index_body,
                ),
            )],
        })?;
        self.code.write(";").line();
        // Set metatable for the class object:
        let mut ctor_body = vec![
            IfStatement {
                condition: Box::new(
                    Bop {
                        left: Box::new(Identifier::new("values", false)),
                        op: Operator::Eq,
                        right: Box::new(TupleLiteral::unit().into_expr()),
                    }
                    .into_expr(),
                ),
                body: vec![
                    Assignment {
                        left: AssignmentTarget::Identifier(
                            Identifier::new("values", false).unwrap_identifier(),
                        ),
                        right: Box::new(MapLiteral { entries: vec![] }.into_expr()),
                        op: None,
                    }
                    .into_statement(),
                ],
                else_if_blocks: vec![],
                else_block: None,
            }
            .into_statement(),
        ];
        // Ctor should initialize the fields first, if any.
        for field in fields {
            let Let {
                name, initializer, ..
            } = field;
            let init = initializer.unwrap_or(TupleLiteral::unit().into_expr());
            let Destructure::Identifier(left) = name else {
                return Err(CompilerError::SyntaxError(format!(
                    "Fields should be declared as names, destructuring assignment is invalid in class field position!"
                )));
            };
            let assign = Identifier::new("values", false)
                .add_member(left)
                .unwrap_member();
            ctor_body.push(IfStatement::new(
                Bop::new(
                    assign.clone().into_expr(),
                    Operator::Eq,
                    TupleLiteral::unit().into_expr(),
                ),
                vec![Assignment::new(assign.into_assignmenttarget(), None, init).into_statement()],
                vec![],
                None,
            ));
        }
        ctor_body.push(
            Return {
                value: Box::new(Call::new(
                    Identifier::new("setmetatable", false),
                    vec![
                        Identifier::new("values", false),
                        Identifier::new("Self", false)
                            .add_member(Identifier::new("__meta__", false).unwrap_identifier()),
                    ],
                    false,
                )),
            }
            .into_statement(),
        );
        let metatable = MapLiteral {
            entries: vec![(
                MapKey::Identifier(Identifier::new("__call", false).unwrap_identifier()),
                LambdaExpr {
                    params: vec![
                        Param {
                            name: Identifier::new("Self", false).unwrap_identifier(),
                            type_def: None,
                        },
                        Param {
                            name: Identifier::new("values", false).unwrap_identifier(),
                            type_def: None,
                        },
                    ],
                    body: ctor_body,
                }
                .into_expr(),
            )],
        }
        .into_expr();
        self.compile_call(
            Call::new(
                Identifier::new("setmetatable", false),
                vec![name.clone().into_expr(), metatable],
                false,
            )
            .unwrap_call(),
        )?;
        self.code.write(";").line();
        self.export_symbol(&modifiers, name)?;
        Ok(())
    }
    fn export_symbol(&mut self, modifiers: &DefModifiers, name: Identifier) -> Result {
        match &self.options.module_type {
            ModuleType::Saturnus => {
                if modifiers.is_pub() {
                    let target = AssignmentTarget::Member(
                        self.module_root_expr
                            .clone()
                            .add_member(name.clone())
                            .unwrap_member(),
                    );
                    self.compile_statement(
                        Assignment::new(target, None, name.into_expr()).into_statement(),
                    )?;
                }
            }
            ModuleType::LocalModuleReturn => todo!(),
            _ => (),
        }
        Ok(())
    }
    fn process_pub_symbol(&mut self, modifiers: &DefModifiers) -> Result {
        if ModuleType::PubAsGlobal != self.options.module_type || !modifiers.is_pub() {
            self.code.write("local ");
        }
        Ok(())
    }
    fn compile_fn(&mut self, fn_def: crate::parsing::ast::Fn) -> Result {
        let crate::parsing::ast::Fn {
            name,
            modifiers,
            arguments,
            body,
        } = fn_def;
        self.process_pub_symbol(&modifiers)?;
        self.code.write("function ");
        self.compile_identifier(name.clone())?;
        self.code.write("(");
        let mut iter = arguments.into_iter();
        if let Some(first) = iter.next() {
            self.compile_identifier(first.name)?;
        }
        for item in iter {
            self.code.write(", ");
            self.compile_identifier(item.name)?;
        }
        self.code.write(")").push();
        self.compile_program(body)?;
        self.code.pop().line().write("end");
        self.export_symbol(&modifiers, name)?;
        Ok(())
    }
    fn compile_return(&mut self, return_stmt: Return) -> Result {
        self.code.write("return ");
        self.compile_expr(*return_stmt.value)?;
        self.code.write(";");
        Ok(())
    }
    fn compile_use(&mut self, use_stmt: Use, root: Option<Vec<Identifier>>) -> Result {
        let Use { path, use_tree } = use_stmt;
        if let Some(tree) = use_tree {
            let root = if let Some(root) = root {
                root.into_iter().chain(path.into_iter()).collect()
            } else {
                path
            };
            for item in tree {
                self.compile_use(item, Some(root.clone()))?;
            }
        } else {
            let name = path.last().cloned().unwrap();
            let mut iter = path.into_iter();
            if let Some(root) = root {
                iter = root.into_iter().chain(iter).collect::<Vec<_>>().into_iter();
            }
            let mut initializer = Identifier::new("__modules__", false);
            for item in iter {
                initializer = initializer.add_member(item);
            }
            self.compile_let(Let {
                name: Destructure::Identifier(name),
                type_def: None,
                initializer: Some(initializer),
                modifiers: DefModifiers::new(),
            })?;
            self.code.line();
        }
        Ok(())
    }
    fn compile_statement(&mut self, stmt: Statement) -> Result {
        self.code.line();
        match stmt {
            Statement::IfStatement(if_statement) => self.compile_if(if_statement)?,
            Statement::Expr(expr) => {
                self.compile_expr(expr)?;
                self.code.write(";");
            }
            Statement::Skip(_) => {
                self.code.write("goto loop_end;");
            }
            Statement::Break(_) => {
                self.code.write("break;");
            }
            Statement::For(value) => self.compile_for(value)?,
            Statement::While(value) => self.compile_while(value)?,
            Statement::Loop(value) => self.compile_loop(value)?,
            Statement::Let(value) => self.compile_let(value)?,
            Statement::Assignment(value) => self.compile_assignment(value)?,
            Statement::ClassDef(class_def) => self.compile_class_def(class_def)?,
            Statement::Fn(fn_def) => self.compile_fn(fn_def)?,
            Statement::Return(return_stmt) => self.compile_return(return_stmt)?,
            Statement::Use(use_stmt) => self.compile_use(use_stmt, None)?,
        }
        Ok(())
    }
    pub fn compile_program(&mut self, ast: Vec<Statement>) -> Result {
        for stmt in ast {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }
    fn mock_module(&mut self, target: AssignmentTarget) -> Result {
        self.compile_statement(
            Assignment::new(
                target.clone(),
                None,
                Bop::new(
                    target.to_expr().clone(),
                    Operator::Or,
                    MapLiteral { entries: vec![] }.into_expr(),
                ),
            )
            .into_statement(),
        )?;
        Ok(())
    }
}

lazy_static! {
    static ref PATH_SEGMENT_SANITIZER: Regex = Regex::new(r"[^A-Za-z0-9_]").unwrap();
    static ref PATH_SEGMENT_START: Regex = Regex::new(r"^[^A-Za-z_]").unwrap();
}

impl Compiler for LuaCompiler {
    fn compile(
        &mut self,
        source: impl SourceCode,
        options: CompilerOptions,
    ) -> std::result::Result<SaturnusIR, CompilerError> {
        self.module_root_expr = Identifier::new("__modules__", false);
        if let Some(_) = &options.override_mod_path {
            todo!()
        }
        self.options = options;
        let location = source.location();
        let code = source.source();
        let parser = ProgramParser::new();
        let ast = match parser.parse(&code) {
            Ok(ast) => ast,
            Err(err) => {
                return Err(CompilerError::ParsingError(format!("{err:?}")));
            }
        };
        if ModuleType::Saturnus == self.options.module_type {
            let modules = Identifier::new("__modules__", false);
            // Initialize modules table
            self.mock_module(AssignmentTarget::Identifier(
                modules.clone().unwrap_identifier(),
            ))?;
            // Initialize this module, if not root.
            if let Some(path) = location {
                let mut out = self.module_root_expr.clone();
                for rest in path.iter() {
                    let segment = rest
                        .to_str()
                        .ok_or(CompilerError::ParsingError(
                            "Invalid source location path! Can't continue!".into(),
                        ))?
                        .to_string();
                    let segment = PATH_SEGMENT_SANITIZER
                        .replace_all(&segment, "_")
                        .into_owned();
                    let segment = PATH_SEGMENT_START.replace(&segment, "_").into_owned();
                    out = out.add_member(Identifier {
                        value: segment,
                        is_escaped: false,
                    });
                    self.mock_module(AssignmentTarget::Member(out.clone().unwrap_member()))?;
                }
                self.module_root_expr = out;
            }
        };
        self.compile_program(ast)?;
        let output = std::mem::replace(&mut self.code, IndentedBuilder::new()).unwrap();
        Ok(SaturnusIR::from(output))
    }
}
