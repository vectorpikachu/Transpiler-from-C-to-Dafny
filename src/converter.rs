use tree_sitter::{Node, Tree};

use crate::{context::Context, dafny_ast::*};

/// 将 C 语法树 转换为 Dafny AST
pub fn convert(tree: Tree, context: &mut Context, source: &str) -> Program {
    let mut global_vars = vec![];
    let mut methods = vec![];
    let root_node = tree.root_node();
    for child in root_node.children(&mut root_node.walk()) {
        match child.kind() {
            "declaration" => {
                if let Some(global) = parse_global_var(&child, context, source) {
                    global_vars.extend(global);
                }
            }
            "function_definition" => {
                if let Some(method) = parse_function(&child, context, source) {
                    methods.push(method);
                }
            }
            _ => {}
        }
    }

    println!("{:?}", global_vars);
    println!("{:?}", methods);

    // 创建 Dafny Class
    let dafny_class = ClassDecl {
        id: "CProgram".to_string(),
        extends: None,
        fields: global_vars.clone(),
        constructor: Some(generate_constructor(&global_vars, context, source)),
        methods,
    };

    Program {
        declarations: vec![Declaration::Class(dafny_class)],
    }
}

/// 解析全局变量
/// 一个 Declaration里面可能声明了很多变量
fn parse_global_var(node: &Node, context: &mut Context, source: &str) -> Option<Vec<FieldDecl>> {
    let mut fields = vec![];

    for decl in node.children_by_field_name("declarator", &mut node.walk()) {
        if decl.kind() == "function_declarator" {
            continue;
        }
        let mut name = decl.utf8_text(source.as_bytes());
        let mut init = None;
        if let Some(init_value) = decl.child_by_field_name("value") {
            name = decl
                .child_by_field_name("declarator")?
                .utf8_text(source.as_bytes());
            init = parse_expr(&init_value, context, source);
        }

        match name {
            Ok(name) => {
                let ty = node.child_by_field_name("type")?;
                let dafny_ty = parse_type(&ty, context, source)?;
                context.declare_var(name.to_string(), dafny_ty.clone());
                fields.push(FieldDecl {
                    id: name.to_string(),
                    type_: dafny_ty,
                    init: init,
                });
            }
            Err(e) => {
                println!("Error parsing field name: {}", e);
            }
        }
    }

    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

/// 解析 类型
fn parse_type(node: &Node, context: &mut Context, source: &str) -> Option<Type> {
    let ty_name = node.utf8_text(source.as_bytes());
    match ty_name {
        Ok(ty_name) => {
            let ty_name = ty_name.trim();
            match ty_name {
                "int" => Some(Type::Int),
                "unsigned int" => Some(Type::Bv(32)),
                "unsigned char" => Some(Type::Bv(8)),
                _ => {
                    // 处理其他类型
                    None
                }
            }
        }
        Err(e) => {
            println!("Error parsing type: {}", e);
            None
        }
    }
}

/// 解析 函数
fn parse_function(node: &Node, context: &mut Context, source: &str) -> Option<MethodDecl> {
    let func_name = parse_func_name(node, context, source);
    let func_type = parse_func_type(node, context, source);
    let func_params = parse_func_params(node, context, source);

    // 必须要有函数体
    let func_body = node.child_by_field_name("body")?;
    let mut stmts = vec![];
    for child in func_body.children(&mut func_body.walk()) {
        let stmt = parse_stmt(&child, context, source);
        if stmt.is_none() {
            continue;
        }
        stmts.extend(stmt.unwrap());
    }

    let returns = match func_type.clone() {
        Some(t) => {
            vec![ReturnVar {
                id: "ret".to_string(),
                type_: t,
            }]
        }
        None => vec![],
    };

    let params_ty = match func_params.clone() {
        Some(params) => params
            .iter()
            .map(|param| param.type_.clone())
            .collect(),
        None => vec![],
    };
    let function_type = Type::Function(params_ty, Box::new(func_type.clone()));
    context.declare_var(func_name.clone().unwrap(), function_type.clone());

    let modify_expr = Expr::Primary(PrimaryExpr::Literal(Literal::This), Type::This);
    let decrease_expr = Expr::Primary(PrimaryExpr::Literal(Literal::Star), Type::Star);
    let method_decl = MethodDecl {
        id: func_name.unwrap(),
        params: func_params.unwrap(),
        returns: returns,
        return_type: func_type,
        requires: vec![],
        ensures: vec![],
        modifies: vec![modify_expr],
        decreases: vec![decrease_expr],
        block: Block { stmts },
    };

    Some(method_decl)
}

fn parse_func_name(node: &Node, context: &mut Context, source: &str) -> Option<String> {
    // 获取函数名
    let func_decl = node.child_by_field_name("declarator")?;
    let func_name = func_decl.child_by_field_name("declarator")?;
    let func_name = func_name.utf8_text(source.as_bytes()).unwrap();
    Some(func_name.to_string())
}

fn parse_func_type(node: &Node, context: &mut Context, source: &str) -> Option<Type> {
    // 获取函数类型
    let func_type = node.child_by_field_name("type")?;
    let func_type = parse_type(&func_type, context, source)?;
    Some(func_type)
}

fn parse_func_params(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Param>> {
    // 获取函数参数列表
    let func_decl = node.child_by_field_name("declarator")?;
    let func_params = func_decl.child_by_field_name("parameters")?;

    let mut params = vec![];

    for param in func_params.children(&mut node.walk()) {
        println!("param: {:?}'s kind {}", param, param.kind());
        match param.kind() {
            "parameter_declaration" => {
                let param_type = param.child_by_field_name("type")?;
                let param_type = parse_type(&param_type, context, source)?;
                let param_name = param.child_by_field_name("declarator")?;
                let param_name = param_name.utf8_text(source.as_bytes());
                match param_name {
                    Ok(name) => {
                        context.declare_var(name.to_string(), param_type.clone());
                        let param = Param {
                            id: name.to_string(),
                            type_: param_type,
                        };
                        params.push(param);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    Some(params)
}

fn parse_stmt(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    println!("\nparse_stmt: {:?}\n", node);
    println!("kind {:?}", node.kind());

    let mut stmts = vec![];

    let stmt = match node.kind() {
        "return_statement" => vec![parse_return(node, context, source).unwrap()],
        "declaration" => parse_declaration(node, context, source).unwrap(),
        "expression_statement" => {
            // 包含了函数调用 / 变量赋值 等
            let stmt_body = node.child(0).unwrap();
            parse_expr_stmt(&stmt_body, context, source).unwrap()
        }
        "while_statement" => {
            let stmt = parse_while_stmt(&node, context, source);
            match stmt {
                Some(stmt) => stmt,
                None => vec![],
            }
        }
        "compound_statement" => {
            let stmt = parse_compound_stmt(&node, context, source);
            match stmt {
                Some(stmt) => stmt,
                None => vec![],
            }
        }
        "if_statement" => {
            println!("看看看看if_statement: {:?}", node);
            let stmt = parse_if_stmt(&node, context, source);
            match stmt {
                Some(stmt) => stmt,
                None => vec![],
            }
        }
        "goto_statement" => {
            // TODO: Parse goto statement more accurately
            // A way of tackling the goto statement is `rellic`.
            let label = node.child(1).unwrap();
            let label_name = label
                .utf8_text(source.as_bytes())
                .expect("Failed to parse label name");
            let mut stmt = vec![];
            match label_name {
                "ERROR" => {
                    stmt.push(Stmt::Assert(Expr::Primary(
                        PrimaryExpr::Literal(Literal::Boolean(false)),
                        Type::Bool,
                    )));
                }
                "LOOPEND" => {
                    stmt.push(Stmt::Break);
                }
                _ => {}
            }
            stmt
        }
        "labeled_statement" => {
            // TODO: Parse labeled statement more accurately
            let stmt_body = node.child(2).unwrap();
            let stmt = parse_stmt(&stmt_body, context, source);
            match stmt {
                Some(stmt) => stmt,
                None => vec![],
            }
        }
        "break_statement" => {
            vec![Stmt::Break]
        }
        "continue_statement" => {
            vec![Stmt::Continue]
        }
        _ => vec![], // TODO: Parse other statements
    };

    stmts.extend(stmt);

    if stmts.is_empty() {
        return None;
    }

    Some(stmts)
}

/// 生成构造函数
fn generate_constructor(vars: &Vec<FieldDecl>, context: &Context, source: &str) -> ConstructorDecl {
    let modify_expr = Expr::Primary(PrimaryExpr::Literal(Literal::This), Type::This);

    let mut stmts = vec![];
    for var in vars {
        if var.init.is_some() {
            let stmt = Stmt::Assign(Assign {
                lhs: Lhs::Identifier(var.id.to_string()),
                expr: var.init.clone().unwrap(),
            });
            stmts.push(stmt);
        } else {
            let zero_expr = Expr::Primary(
                PrimaryExpr::Literal(Literal::Integer("0".to_string())),
                var.type_.clone(),
            );
            let stmt = Stmt::Assign(Assign {
                lhs: Lhs::Identifier(var.id.to_string()),
                expr: zero_expr,
            });
            stmts.push(stmt);
        }
    }

    ConstructorDecl {
        params: vec![],
        requires: vec![],
        ensures: vec![],
        modifies: vec![modify_expr],
        block: Block { stmts: stmts },
    }
}

fn parse_return(node: &Node, context: &mut Context, source: &str) -> Option<Stmt> {
    println!("stmt return expr {:?}", node.field_name_for_child(2));
    let mut ret_expr = None;
    for child_node in node.children(&mut node.walk()) {
        match child_node.kind() {
            "identifier" => {
                let name = child_node
                    .utf8_text(source.as_bytes())
                    .expect("identifier failed")
                    .to_string();
                ret_expr = Some(Expr::Primary(
                    PrimaryExpr::Identifier(name.clone()),
                    context.lookup_var(&name).unwrap().clone(),
                ));
            }
            ";" => {}
            "return" => {}
            _ => {
                // Expressions
                println!("{:?}", child_node.utf8_text(source.as_bytes()));
                ret_expr = parse_expr(&child_node, context, source);
            }
        }
    }
    Some(Stmt::Return(ret_expr))
}

fn parse_declaration(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let mut stmts = vec![];
    for decl in node.children_by_field_name("declarator", &mut node.walk()) {
        if decl.kind() == "function_declarator" {
            continue;
        }

        println!("声明的类型是: {:?}", decl.kind());

        if decl.kind() == "array_declarator" {
            // TODO: 处理数组
            continue;
        }

        let mut name = decl.utf8_text(source.as_bytes());
        let mut init = None;
        if let Some(init_value) = decl.child_by_field_name("value") {
            name = decl
                .child_by_field_name("declarator")?
                .utf8_text(source.as_bytes());
            init = parse_expr(&init_value, context, source);
        }

        match name {
            Ok(name) => {
                let ty = node.child_by_field_name("type")?;
                let dafny_ty = parse_type(&ty, context, source)?;
                context.declare_var(name.to_string(), dafny_ty.clone());
                stmts.push(Stmt::DeclVar(Var {
                    id: name.to_string(),
                    type_: dafny_ty,
                    init: init,
                }));
            }
            Err(e) => {
                println!("Error parsing field name: {}", e);
            }
        }
    }

    if stmts.is_empty() {
        None
    } else {
        Some(stmts)
    }
}

fn parse_expr_stmt(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let mut stmts = vec![];
    match node.kind() {
        "assignment_expression" => {
            let stmt = parse_assign_statement(&node, context, source);
            if stmt.is_some() {
                stmts.push(stmt.unwrap());
            }
        }
        "comma_expression" => {
            let comma_stmts = parse_comma_statement(&node, context, source);
            if comma_stmts.is_some() {
                stmts.extend(comma_stmts.unwrap());
            }
        }
        "update_expression" => {
            let update_stmt = parse_update_statement(&node, context, source);
            if update_stmt.is_some() {
                stmts.extend(update_stmt.unwrap());
            }
        }
        _ => {}
    }

    Some(stmts)
}

fn parse_assign_statement(node: &Node, context: &mut Context, source: &str) -> Option<Stmt> {
    let id = node.child(0).unwrap();
    let expr = node.child(2).unwrap();

    let id_name = id
        .utf8_text(source.as_bytes())
        .expect("Failed to get identifier name");

    let expr = parse_expr(&expr, context, source);
    let expr = match expr {
        Some(expr) => expr,
        None => Expr::Primary(
            PrimaryExpr::Literal(Literal::Integer("0".to_string())),
            context.lookup_var(&id_name).unwrap().clone(),
        ),
    };

    Some(Stmt::Assign(Assign {
        lhs: Lhs::Identifier(id_name.to_string()),
        expr: expr,
    }))
}

fn parse_comma_statement(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let mut stmts = vec![];
    for child in node.children(&mut node.walk()) {
        if child.kind() == "," {
            continue;
        }
        let stmt = parse_expr_stmt(&child, context, source);
        if stmt.is_some() {
            stmts.extend(stmt.unwrap());
        }
    }
    Some(stmts)
}

fn parse_update_statement(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let mut stmts = vec![];
    let id;
    let op: Node<'_>;
    if node.child(0).unwrap().kind() == "identifier" {
        id = node.child(0).unwrap().utf8_text(source.as_bytes()).unwrap();
        op = node.child(1).unwrap();
    } else {
        op = node.child(0).unwrap();
        id = node.child(1).unwrap().utf8_text(source.as_bytes()).unwrap();
    }

    let ty = context.lookup_var(&id).unwrap().clone();

    match op.kind() {
        "++" => stmts.push(Stmt::Assign(Assign {
            lhs: Lhs::Identifier(id.to_string()),
            expr: Expr::Additive(
                AdditiveOp::Add,
                Box::new(Expr::Primary(
                    PrimaryExpr::Identifier(id.to_string()),
                    ty.clone(),
                )),
                Box::new(Expr::Primary(
                    PrimaryExpr::Literal(Literal::Integer("1".to_string())),
                    ty.clone(),
                )),
                ty.clone(),
            ),
        })),
        "--" => stmts.push(Stmt::Assign(Assign {
            lhs: Lhs::Identifier(id.to_string()),
            expr: Expr::Additive(
                AdditiveOp::Sub,
                Box::new(Expr::Primary(
                    PrimaryExpr::Identifier(id.to_string()),
                    ty.clone(),
                )),
                Box::new(Expr::Primary(
                    PrimaryExpr::Literal(Literal::Integer("1".to_string())),
                    ty.clone(),
                )),
                ty.clone(),
            ),
        })),
        _ => {}
    }

    if stmts.is_empty() {
        return None;
    }

    Some(stmts)
}

fn parse_while_stmt(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let cond = parse_expr(&node.child(1).unwrap(), context, source);
    let body = parse_stmt(&node.child(2).unwrap(), context, source);
    if cond.is_some() && body.is_some() {
        return Some(vec![Stmt::WhileLoop(WhileLoop {
            cond: cond.unwrap(),
            invariants: vec![],
            decreases: vec![Expr::Primary(
                PrimaryExpr::Literal(Literal::Star),
                Type::Star,
            )],
            block: Block {
                stmts: body.unwrap(),
            },
        })]);
    }
    None
}

/// 解析 if 语句
/// if: if 0
/// parenthesized_expression: (
/// compound_statement: {
/// else_clause: else
fn parse_if_stmt(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let cond = parse_expr(&node.child(1).unwrap(), context, source);
    let then_body = parse_stmt(&node.child(2).unwrap(), context, source);
    let else_body = if let Some(else_clause) = node.child(3) {
        parse_stmt(&else_clause.child(1).unwrap(), context, source)
    } else {
        None
    };

    println!("请注意看cond: {:?} -> {:?}", node.child(1), cond);
    println!("请注意看then_body: {:?}", then_body);

    if cond.is_some() && then_body.is_some() {
        let if_stmt = Stmt::IfElse(IfElse {
            cond: cond.unwrap(),
            then_block: Block {
                stmts: then_body.unwrap(),
            },
            else_block: match else_body {
                Some(else_body) => Some(Block { stmts: else_body }),
                None => None,
            },
        });
        return Some(vec![if_stmt]);
    }
    None
}

fn parse_compound_stmt(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let mut stmts = vec![];
    for child in node.children(&mut node.walk()) {
        if child.kind() == "{" || child.kind() == "}" {
            continue;
        }
        let stmt = parse_stmt(&child, context, source);
        if stmt.is_some() {
            stmts.extend(stmt.unwrap());
        }
    }
    if stmts.is_empty() {
        return None;
    }
    Some(stmts)
}

// TODO: Parse expressions
fn parse_expr(node: &Node, context: &mut Context, source: &str) -> Option<Expr> {
    println!("我们的表达式类型是: {:?}", node.kind());
    let exp = match node.kind() {
        "parenthesized_expression" => parse_expr(&node.child(1).unwrap(), context, source),
        "binary_expression" => parse_binary_expr(node, context, source),
        "identifier" => {
            let id = node.utf8_text(source.as_bytes()).unwrap();
            Some(Expr::Primary(
                PrimaryExpr::Identifier(id.to_string()),
                context.lookup_var(id).unwrap().clone(),
            ))
        }
        "unary_expression" => parse_unary_expr(node, context, source),
        "call_expression" => parse_call_expr(node, context, source),
        "number_literal" => {
            let num = node.utf8_text(source.as_bytes()).unwrap();
            Some(Expr::Primary(
                PrimaryExpr::Literal(Literal::Integer(num.to_string())),
                Type::Number,
            ))
        }
        "update_expression" => {
            if node.child(0).unwrap().kind() == "identifier" {
                let id = node.child(0).unwrap().utf8_text(source.as_bytes()).unwrap();
                let op = node.child(1).unwrap().kind();
                Some(Expr::Primary(
                    PrimaryExpr::Identifier(id.to_string()),
                    context.lookup_var(id).unwrap().clone(),
                ))
            } else {
                let id = node.child(1).unwrap().utf8_text(source.as_bytes()).unwrap();
                let op = node.child(0).unwrap().kind();
                let ty = context.lookup_var(id).unwrap().clone();
                match op {
                    "++" => Some(Expr::Additive(
                        AdditiveOp::Add,
                        Box::new(Expr::Primary(
                            PrimaryExpr::Identifier(id.to_string()),
                            ty.clone(),
                        )),
                        Box::new(Expr::Primary(
                            PrimaryExpr::Literal(Literal::Integer("1".to_string())),
                            ty.clone(),
                        )),
                        ty.clone(),
                    )),
                    "--" => Some(Expr::Additive(
                        AdditiveOp::Sub,
                        Box::new(Expr::Primary(
                            PrimaryExpr::Identifier(id.to_string()),
                            ty.clone(),
                        )),
                        Box::new(Expr::Primary(
                            PrimaryExpr::Literal(Literal::Integer("1".to_string())),
                            ty.clone(),
                        )),
                        ty.clone(),
                    )),
                    _ => None,
                }
            }
        }
        _ => {
            // TODO: Parse other expressions
            None
        }
    };
    exp
}
// TODO: get any expression's type

fn parse_unary_expr(node: &Node, context: &mut Context, source: &str) -> Option<Expr> {
    let op = node.child(0).unwrap();
    let expr = parse_expr(&node.child(1).unwrap(), context, source);
    if expr.is_none() {
        return None;
    }
    let expr = expr.unwrap();
    // TODO : Conversion Type
    match op.kind() {
        "+" => Some(expr),
        "-" => Some(Expr::Unary(UnaryOp::Neg, Box::new(expr), Type::Int)),
        "!" => Some(Expr::Unary(UnaryOp::Not, Box::new(expr), Type::Bool)),
        _ => None,
    }
}

fn parse_binary_expr(node: &Node, context: &mut Context, source: &str) -> Option<Expr> {
    let lhs = parse_expr(&node.child(0).unwrap(), context, source);
    let rhs = parse_expr(&node.child(2).unwrap(), context, source);
    if lhs.is_none() || rhs.is_none() {
        println!("Error: parse_binary_expr {:?} {:?}", lhs, rhs);
        return None;
    }
    let op = node.child(1).unwrap();
    println!("我们的运算符是op: {:?}", op);
    let lhs = lhs.unwrap();
    let rhs = rhs.unwrap();
    let exp = match op.kind() {
        "<" => Expr::Comparison(
            ComparisonOp::Lt,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bool,
        ),
        ">" => Expr::Comparison(
            ComparisonOp::Gt,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bool,
        ),
        "<=" => Expr::Comparison(
            ComparisonOp::Le,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bool,
        ),
        ">=" => Expr::Comparison(
            ComparisonOp::Ge,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bool,
        ),
        "==" => Expr::Equality(
            EqualityOp::Eq,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bool,
        ),
        "!=" => Expr::Equality(
            EqualityOp::Ne,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bool,
        ),
        "+" => Expr::Additive(
            AdditiveOp::Add,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            max_ty(lhs.clone().get_type(), rhs.clone().get_type()),
        ),
        "-" => Expr::Additive(
            AdditiveOp::Sub,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            max_ty(lhs.clone().get_type(), rhs.clone().get_type()),
        ),
        "*" => Expr::Mult(
            MultOp::Mul,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            max_ty(lhs.clone().get_type(), rhs.clone().get_type()),
        ),
        "/" => Expr::Mult(
            MultOp::Div,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            max_ty(lhs.clone().get_type(), rhs.clone().get_type()),
        ),
        "%" => Expr::Mult(
            MultOp::Mod,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Int,
        ),
        "&&" => Expr::LogicalAnd(Box::new(lhs.clone()), Box::new(rhs.clone()), Type::Bool),
        "||" => Expr::LogicalOr(Box::new(lhs.clone()), Box::new(rhs.clone()), Type::Bool),
        "<<" => Expr::Shift(
            ShiftOp::Shl,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bv(32),
        ),
        ">>" => Expr::Shift(
            ShiftOp::Shr,
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
            Type::Bv(32),
        ),
        "&" => Expr::BitwiseAnd(Box::new(lhs.clone()), Box::new(rhs.clone()), Type::Bv(32)),
        "|" => Expr::BitwiseOr(Box::new(lhs.clone()), Box::new(rhs.clone()), Type::Bv(32)),
        "^" => Expr::BitwiseXor(Box::new(lhs.clone()), Box::new(rhs.clone()), Type::Bv(32)),
        _ => {
            // TODO: 处理其他比较操作符
            return None;
        }
    };
    Some(exp)
}

fn parse_call_expr(node: &Node, context: &mut Context, source: &str) -> Option<Expr> {
    let func_name = node.child(0).unwrap();
    let mut args = vec![];
    let arg_list = node.child(1).unwrap();
    for arg in arg_list.children(&mut arg_list.walk()) {
        if arg.kind() == "(" || arg.kind() == ")" || arg.kind() == "," {
            continue;
        }
        let arg_expr = parse_expr(&arg, context, source);
        if arg_expr.is_some() {
            args.push(arg_expr.unwrap());
        }
    }
    let func_name = func_name.utf8_text(source.as_bytes()).unwrap();

    if func_name == "unknown_int" {
        return Some(Expr::Primary(PrimaryExpr::Literal(Literal::Star), Type::Star));
    }

    let func_ty = context.lookup_var(func_name).unwrap().clone();

    let ret_ty = match func_ty.clone() {
        Type::Function(_, ret_ty) => *ret_ty,
        _ => None,
    };

    let ret_ty = match ret_ty {
        Some(ty) => ty,
        None => Type::Star,
    };

    Some(Expr::Primary(PrimaryExpr::Call(
        Box::new(Expr::Primary(PrimaryExpr::Identifier(
            func_name.to_string(),
        ), func_ty.clone())),
        args,
    ), ret_ty))
}
