//todo: printer
use crate::dafny_ast::*;
pub struct DafnyPrinter;

impl DafnyPrinter {
    pub fn print_program(program: &Program) -> String {
        let mut result = String::new();
        for decl in &program.declarations {
            result.push_str(&Self::print_declaration(decl, 0));
            result.push('\n');
        }
        result
    }

    fn print_declaration(decl: &Declaration, indent_level: usize) -> String {
        match decl {
            Declaration::Method(method) => Self::print_method_decl(method, indent_level),
            Declaration::Function(function) => Self::print_function_decl(function, indent_level),
            Declaration::Predicate(predicate) => {
                Self::print_predicate_decl(predicate, indent_level)
            }
            Declaration::Datatype(datatype) => Self::print_datatype_decl(datatype, indent_level),
            Declaration::Class(class) => Self::print_class_decl(class, indent_level),
        }
    }

    fn print_method_decl(method: &MethodDecl, indent_level: usize) -> String {
        let mut result = format!("{}method {}(", Self::indent(indent_level), method.id);
        result.push_str(&Self::print_params(&method.params));
        result.push(')');

        if !method.returns.is_empty() {
            result.push_str(" returns (");
            result.push_str(&Self::print_return_vars(&method.returns));
            result.push(')');
        }

        if !method.requires.is_empty() {
            result.push_str("\n");
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("requires ");
            result.push_str(&Self::print_expr_list(&method.requires));
        }

        if !method.ensures.is_empty() {
            result.push_str("\n");
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("ensures ");
            result.push_str(&Self::print_expr_list(&method.ensures));
        }

        if !method.decreases.is_empty() {
            result.push('\n');
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("decreases ");
            result.push_str(&Self::print_expr_list(&method.decreases));
        }

        if !method.modifies.is_empty() {
            result.push('\n');
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("modifies ");
            result.push_str(&Self::print_expr_list(&method.modifies));
        }

        result.push('\n');
        result.push_str(&Self::indent(indent_level));
        result.push_str("{\n");
        result.push_str(&Self::print_block(&method.block, indent_level + 1));
        result.push_str(&Self::indent(indent_level));
        result.push_str("}\n");
        result
    }

    fn print_function_decl(function: &FunctionDecl, indent_level: usize) -> String {
        let mut result = Self::indent(indent_level);
        result.push_str(&format!(
            "function {}({}) : {}",
            function.id,
            Self::print_params(&function.params),
            Self::print_type(&function.return_type)
        ));

        if !function.requires.is_empty() {
            result.push_str("\n");
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("requires ");
            result.push_str(&Self::print_expr_list(&function.requires));
        }

        if !function.ensures.is_empty() {
            result.push_str("\n");
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("ensures ");
            result.push_str(&Self::print_expr_list(&function.ensures));
        }

        result.push_str(" {\n");
        result.push_str(&Self::indent(indent_level + 1));
        result.push_str(&Self::print_expr(&function.body));
        result.push_str(";\n");
        result.push_str(&Self::indent(indent_level));
        result.push_str("}\n");
        result
    }

    fn print_predicate_decl(predicate: &PredicateDecl, indent_level: usize) -> String {
        let mut result = format!(
            "{}predicate {}({})",
            Self::indent(indent_level),
            predicate.id,
            Self::print_params(&predicate.params)
        );

        if !predicate.requires.is_empty() {
            result.push_str("\n");
            result.push_str(&Self::indent(indent_level));
            result.push_str("requires");
            result.push_str(&Self::print_expr_list(&predicate.requires));
        }

        result.push_str(" {\n  ");
        result.push_str(&Self::indent(indent_level + 1));
        result.push_str(&Self::print_expr(&predicate.body));
        result.push_str("\n}\n");
        result
    }

    fn print_datatype_decl(datatype: &DatatypeDecl, indent_level: usize) -> String {
        let mut result = format!("  datatype {} = ", datatype.id);
        let constructors: Vec<String> = datatype
            .constructors
            .iter()
            .map(|c| Self::print_constructor_decl(c, indent_level))
            .collect();
        result.push_str(&constructors.join(" | "));
        result.push_str(";\n");
        result
    }

    fn print_constructor_decl(constructor: &ConstructorDecl, indent_level: usize) -> String {
        let params = Self::print_params(&constructor.params);
        let body = Self::print_block(&constructor.block, indent_level + 1);
        format!(
            "{}constructor({}){{\n{}{}}}",
            Self::indent(indent_level),
            params,
            body,
            Self::indent(indent_level)
        )
    }

    fn print_class_decl(class: &ClassDecl, indent_level: usize) -> String {
        let mut result = format!("{}class {}", Self::indent(indent_level), class.id);
        if let Some(parent) = &class.extends {
            result.push_str(&format!("extends {}", Self::print_type(parent)));
        }
        result.push_str(" {\n");

        for field in &class.fields {
            result.push_str(&format!(
                "{}var {}: {}\n",
                Self::indent(indent_level + 1),
                field.id,
                Self::print_type(&field.type_)
            ));
        }

        if let Some(constructor) = &class.constructor {
            result.push_str(&Self::print_constructor_decl(constructor, indent_level + 1));
            result.push('\n');
        }

        for method in &class.methods {
            result.push_str(&Self::print_method_decl(method, indent_level + 1));
            result.push('\n');
        }

        result.push_str("}\n");
        result
    }

    fn print_params(params: &[Param]) -> String {
        params
            .iter()
            .map(|p| format!("{}: {}", p.id, Self::print_type(&p.type_)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn print_return_vars(returns: &[ReturnVar]) -> String {
        returns
            .iter()
            .map(|r| format!("{}: {}", r.id, Self::print_type(&r.type_)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn print_type(type_: &Type) -> String {
        match type_ {
            Type::Int => "int".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Real => "real".to_string(),
            Type::Bv(size) => format!("bv{}", size),
            Type::Set(inner) => format!("set<{}>", Self::print_type(inner)),
            Type::Seq(inner) => format!("seq<{}>", Self::print_type(inner)),
            Type::Array(inner) => format!("array<{}>", Self::print_type(inner)),
            Type::Named(name) => name.clone(),
            Type::Function(args, ret) => format!(
                "({}) -> {}",
                args.iter()
                    .map(|arg| Self::print_type(arg))
                    .collect::<Vec<_>>()
                    .join(", "),
                Self::print_type(ret)
            ),
        }
    }

    fn print_expr_list(exprs: &[Expr]) -> String {
        exprs
            .iter()
            .map(Self::print_expr)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn print_expr(expr: &Expr) -> String {
        match expr {
            Expr::LogicalOr(lhs, rhs) => {
                format!("({} || {})", Self::print_expr(lhs), Self::print_expr(rhs))
            }
            Expr::LogicalAnd(lhs, rhs) => {
                format!("({} && {})", Self::print_expr(lhs), Self::print_expr(rhs))
            }
            Expr::Equality(op, lhs, rhs) => format!(
                "({} {} {})",
                Self::print_expr(lhs),
                match op {
                    EqualityOp::Eq => "==",
                    EqualityOp::Ne => "!=",
                },
                Self::print_expr(rhs)
            ),
            Expr::Primary(primary) => Self::print_primary_expr(primary),
            Expr::Additive(op, lhs, rhs) => {
                format!(
                    "({} {} {})",
                    Self::print_expr(lhs),
                    match op {
                        AdditiveOp::Add => "+",
                        AdditiveOp::Sub => "-",
                    },
                    Self::print_expr(rhs)
                )
            }
            Expr::Mult(op, lhs, rhs) => {
                format!(
                    "({} {} {})",
                    Self::print_expr(lhs),
                    match op {
                        MultOp::Mul => "*",
                        MultOp::Div => "/",
                        MultOp::Mod => "%",
                    },
                    Self::print_expr(rhs)
                )
            }
            Expr::Unary(op, expr) => {
                format!(
                    "({} {})",
                    match op {
                        UnaryOp::Not => "!",
                        UnaryOp::Neg => "-",
                        UnaryOp::Old => "old",
                    },
                    Self::print_expr(expr)
                )
            }
            Expr::BitwiseAnd(lhs, rhs) => {
                format!("({} & {})", Self::print_expr(lhs), Self::print_expr(rhs))
            }
            Expr::BitwiseOr(lhs, rhs) => {
                format!("({} | {})", Self::print_expr(lhs), Self::print_expr(rhs))
            }
            Expr::BitwiseXor(lhs, rhs) => {
                format!("({} ^ {})", Self::print_expr(lhs), Self::print_expr(rhs))
            }
            Expr::Shift(op, lhs, rhs) => match op {
                ShiftOp::Shl => format!("({} << {})", Self::print_expr(lhs), Self::print_expr(rhs)),
                ShiftOp::Shr => format!("({} >> {})", Self::print_expr(lhs), Self::print_expr(rhs)),
            },
            Expr::Comparison(op, lhs, rhs) => match op {
                ComparisonOp::Lt => {
                    format!("({} < {})", Self::print_expr(lhs), Self::print_expr(rhs))
                }
                ComparisonOp::Gt => {
                    format!("({} > {})", Self::print_expr(lhs), Self::print_expr(rhs))
                }
                ComparisonOp::Le => {
                    format!("({} <= {})", Self::print_expr(lhs), Self::print_expr(rhs))
                }
                ComparisonOp::Ge => {
                    format!("({} >= {})", Self::print_expr(lhs), Self::print_expr(rhs))
                }
            },
            Expr::IfThenElse(cond, then_expr, else_expr) => {
                format!(
                    "if {} then {} else {}",
                    Self::print_expr(cond),
                    Self::print_expr(then_expr),
                    Self::print_expr(else_expr)
                )
            }
            _ => unimplemented!(),
        }
    }

    fn print_primary_expr(primary: &PrimaryExpr) -> String {
        match primary {
            PrimaryExpr::Literal(literal) => Self::print_literal(literal),
            PrimaryExpr::Identifier(name) => name.clone(),
            PrimaryExpr::Index(expr, id) => {
                format!("{}[{}]", Self::print_expr(expr), Self::print_expr(id))
            }
            PrimaryExpr::MemberAccess(expr, name) => format!("{}.{}", Self::print_expr(expr), name),
            PrimaryExpr::Call(expr, args) => {
                let mut args_str = String::new();
                for arg in args {
                    args_str += &Self::print_expr(arg);
                    args_str += ", ";
                }
                args_str.pop();
                args_str.pop();
                format!("{}({})", Self::print_expr(expr), args_str)
            }
        }
    }

    fn print_literal(literal: &Literal) -> String {
        match literal {
            Literal::Integer(value) => value.clone(),
            Literal::Boolean(value) => value.to_string(),
            Literal::Null => "null".to_string(),
            Literal::Star => "*".to_string(),
            Literal::This => "this".to_string(),
            Literal::Sequence(values) => {
                let mut result = "[".to_string();
                for value in values {
                    result += &format!("{}, ", Self::print_expr(value));
                }
                result += "]";
                result
            }
        }
    }

    fn print_block(block: &Block, indent_level: usize) -> String {
        let mut result = String::new();
        for stmt in &block.stmts {
            result.push_str(&Self::print_stmt(stmt, indent_level));
            result.push('\n');
        }
        result
    }

    fn print_stmt(stmt: &Stmt, indent_level: usize) -> String {
        let mut result = Self::indent(indent_level);
        match stmt {
            Stmt::Assign(assign) => {
                result.push_str(&Self::print_assign(assign));
            }
            Stmt::IfElse(if_else) => {
                result.push_str(&Self::print_if_else(if_else, indent_level));
            }
            Stmt::WhileLoop(while_loop) => {
                result.push_str(&Self::print_while_loop(while_loop, indent_level));
            }
            Stmt::ForLoop(for_loop) => {
                result.push_str(&Self::print_for_loop(for_loop, indent_level));
            }
            Stmt::Match(match_stmt) => {
                result.push_str(&Self::print_match(match_stmt, indent_level));
            }
            Stmt::Assert(expr) => {
                result.push_str(&format!("assert {};", Self::print_expr(expr)));
            }
            Stmt::Print(expr) => {
                result.push_str(&format!("print {};", Self::print_expr(expr)));
            }
            Stmt::Return(expr) => {
                result.push_str(&format!(
                    "return {};",
                    match expr {
                        Some(e) => Self::print_expr(e),
                        None => "".to_string(),
                    }
                ));
            }
            Stmt::DeclVar(var) => {
                if var.init.is_some() {
                    result.push_str(&format!(
                        "var {} : {} := {};",
                        var.id,
                        Self::print_type(&var.type_),
                        Self::print_expr(var.init.as_ref().unwrap())
                    ));
                } else {
                    result.push_str(&format!(
                        "var {} : {};",
                        var.id,
                        Self::print_type(&var.type_)
                    ));
                }
            }
            Stmt::Break => {
                result.push_str("break;");
            }
            Stmt::Continue => {
                result.push_str("continue;");
            }
        }
        result
    }

    fn print_assign(assign: &Assign) -> String {
        format!(
            "{} := {};",
            Self::print_lhs(&assign.lhs),
            Self::print_expr(&assign.expr)
        )
    }

    fn print_lhs(lhs: &Lhs) -> String {
        match lhs {
            Lhs::Identifier(name) => name.clone(),
            Lhs::MemberAccess(object, field) => format!("{}.{}", Self::print_expr(object), field),
            Lhs::Index(array, index) => {
                format!("{}[{}]", Self::print_expr(array), Self::print_expr(index))
            }
        }
    }

    fn print_if_else(if_else: &IfElse, indent_level: usize) -> String {
        let mut result = format!("if ({}) {{\n", Self::print_expr(&if_else.cond));
        result.push_str(&Self::print_block(&if_else.then_block, indent_level + 1));
        result.push_str(&Self::indent(indent_level));
        result.push('}');
        if let Some(else_block) = &if_else.else_block {
            result.push_str(" else {\n");
            result.push_str(&Self::print_block(else_block, indent_level + 1));
            result.push_str(&Self::indent(indent_level));
            result.push('}');
        }
        result
    }

    fn print_while_loop(while_loop: &WhileLoop, indent_level: usize) -> String {
        let mut result = format!("while {}\n", Self::print_expr(&while_loop.cond));
        if !while_loop.invariants.is_empty() {
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("invariant ");
            result.push_str(&Self::print_expr_list(&while_loop.invariants));
            result.push('\n');
        }
        if !while_loop.decreases.is_empty() {
            result.push_str(&Self::indent(indent_level + 1));
            result.push_str("decreases ");
            result.push_str(&Self::print_expr_list(&while_loop.decreases));
            result.push('\n');
        }
        result.push_str(&Self::indent(indent_level));
        result.push_str("{\n");
        result.push_str(&Self::print_block(&while_loop.block, indent_level + 1));
        result.push_str(&Self::indent(indent_level));
        result.push('}');
        result
    }

    fn print_for_loop(for_loop: &ForLoop, indent_level: usize) -> String {
        let mut result = format!(
            "for {} := {} to {} {{\n",
            for_loop.id,
            Self::print_expr(&for_loop.start),
            Self::print_expr(&for_loop.end)
        );
        result.push_str(&Self::print_block(&for_loop.block, indent_level + 1));
        result.push_str(&Self::indent(indent_level));
        result.push('}');
        result
    }

    fn print_match(match_stmt: &Match, indent_level: usize) -> String {
        let mut result = format!("match {} {{\n", Self::print_expr(&match_stmt.expr));
        for case in &match_stmt.cases {
            result.push_str(&Self::print_case(case, indent_level + 1));
            result.push('\n');
        }
        result.push_str(&Self::indent(indent_level));
        result.push('}');
        result
    }

    fn print_case(case: &Case, indent_level: usize) -> String {
        let mut result = format!(
            "{}case {} => {{\n",
            Self::indent(indent_level),
            Self::print_pattern(&case.pattern)
        );
        for stmt in &case.stmts {
            result.push_str("  ");
            result.push_str(&Self::print_stmt(stmt, indent_level + 1));
            result.push('\n');
        }
        result.push_str(&Self::indent(indent_level));
        result.push('}');
        result
    }

    fn print_pattern(pattern: &Pattern) -> String {
        match pattern {
            Pattern::Identifier(name) => name.clone(),
            Pattern::Constructor(name, patterns) => {
                let sub_patterns = patterns
                    .iter()
                    .map(Self::print_pattern)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name, sub_patterns)
            }
        }
    }

    fn indent(level: usize) -> String {
        "  ".repeat(level)
    }
}
