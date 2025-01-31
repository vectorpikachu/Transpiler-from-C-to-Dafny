use std::vec;

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
            "preproc_def" => {
                parse_marcro(&child, context, source); // add the marcro to the context
                // replace the marcro in the identifier
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

fn parse_marcro(node: &Node, context: &mut Context, source: &str) {
    let id = node.child(1).unwrap();
    let id = id.utf8_text(source.as_bytes()).expect("Error parsing macro id").trim();
    let val = node.child(2).unwrap();
    let val = val.utf8_text(source.as_bytes()).expect("Error parsing macro val").trim();
    context.insert_macro(id.to_string(), val.to_string());
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
                "_Bool" => Some(Type::Bool), // 处理 _Bool 类型
                "bool" => Some(Type::Bool),
                "unsigned short" => Some(Type::Bv(16)),
                "unsigned long long" => Some(Type::Bv(64)),
                "float" => Some(Type::Real), // 处理 float 类型, float 的溢出也是一个 UB.
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
    let method_decl = MethodDecl {
        id: func_name.clone().unwrap(),
        params: func_params.clone(),
        returns: vec![],
        return_type: func_type.clone(),
        requires: vec![],
        ensures: vec![],
        modifies: vec![],
        decreases: vec![],
        block: Block { stmts: vec![] },
    };
    context.enter_method(method_decl.clone());
    for child in func_body.children(&mut func_body.walk()) {
        let stmt = parse_stmt(&child, context, source);
        if stmt.is_none() {
            continue;
        }
        stmts.extend(stmt.unwrap());
    }

    let mut has_return = false;
    for stmt in stmts.iter() {
        if is_return(stmt) {
            has_return = true;
            break;
        }
    }

    if !has_return && func_type.is_some() {
        stmts.push(Stmt::Return(Some(
            Expr::Primary(PrimaryExpr::Literal(Literal::Integer("0".to_string())), Type::Int)
        )));
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
    let requires_expr = if func_name.clone().unwrap() == "errorFn" {
        Expr::Primary(PrimaryExpr::Literal(Literal::Boolean(false)), Type::Bool)
    } else {
        Expr::Primary(PrimaryExpr::Literal(Literal::Boolean(true)), Type::Bool)
    };
    let method_decl = MethodDecl {
        id: func_name.unwrap(),
        params: func_params,
        returns: returns,
        return_type: func_type,
        requires: vec![requires_expr],
        ensures: vec![],
        modifies: vec![modify_expr],
        decreases: vec![decrease_expr],
        block: Block { stmts },
    };
    context.exit_method();
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
        match param.kind() {
            "parameter_declaration" => {
                // TODO: Parse array parameters
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
                "END" => {
                    let method = context.get_current_method().unwrap();
                    let return_type = method.return_type.clone();
                    match return_type {
                        Some(t) => {
                            // TODO: further processing
                            stmt.push(Stmt::Return(Some(Expr::Primary(
                                PrimaryExpr::Literal(Literal::Integer("0".to_string())),
                                t,
                            ))));
                        }
                        None => {
                            stmt.push(Stmt::Return(None));
                        }
                    }
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
        "for_statement" => {
            let stmt = parse_for_stmt(&node, context, source);
            match stmt {
                Some(stmt) => stmt,
                None => vec![],
            }
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
        let var_ty = var.type_.clone();
        if var.init.is_some() {
            let stmt = Stmt::Assign(Assign {
                lhs: Lhs::Identifier(var.id.to_string(), var_ty.clone()),
                expr: var.init.clone().unwrap(),
            });
            stmts.push(stmt);
        } else {
            let zero_expr = Expr::Primary(
                PrimaryExpr::Literal(Literal::Integer("0".to_string())),
                var.type_.clone(),
            );
            let stmt = Stmt::Assign(Assign {
                lhs: Lhs::Identifier(var.id.to_string(), var_ty.clone()),
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

        if decl.kind() == "array_declarator" {
            //? 在此处进入了, 意味着没有初始化列表.
            let ty = node.child_by_field_name("type")?;
            let dafny_ty = parse_type(&ty, context, source)?;
            let ret = parse_array_decl(&decl, context, source, &dafny_ty);
            return ret;
        }

        let mut name = decl.utf8_text(source.as_bytes());
        let mut init = None;
        if decl.kind() == "init_declarator" {
            let decl_node = decl.child_by_field_name("declarator")?;
            let init_value = decl.child_by_field_name("value")?;
            if decl_node.kind() == "array_declarator" {
                // TODO: 处理数组
                let ty = node.child_by_field_name("type")?;
                let dafny_ty = parse_type(&ty, context, source)?;
                let stmt = parse_array_decl(&decl_node, context, source, &dafny_ty);
                if stmt.is_some() {
                    stmts.extend(stmt.clone().unwrap());
                }
                let array_id = stmt.unwrap()[0].get_decl_id();
                println!("初始化列表为{:?}", init_value);
                let init_stmts = parse_init_list(&init_value, context, source, &array_id);
                if init_stmts.is_some() {
                    stmts.extend(init_stmts.unwrap());
                }
                println!("初始化列表为{:?}", stmts);

                return Some(stmts);
            }
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

fn parse_array_decl(node: &Node, context: &mut Context, source: &str, base: &Type) -> Option<Vec<Stmt>> {
    // let mut stmts = vec![];
    //? 只需要写成 var a := new T[m, n] -> 自动识别出是array<array<T>>类型的
    let id = node.child(0).unwrap();
    let array_name = id.utf8_text(source.as_bytes()).unwrap();
    let mut dims = vec![];
    let mut stmts = vec![];
    let mut dafny_ty = base.clone();
    for child in node.children(&mut node.walk()) {
        if child.kind() == "[" || child.kind() == "]" || child.kind() == "identifier" {
            continue;
        }
        println!("我们的数组大小是{:?}", child);
        let dim = parse_expr(&child, context, source);
        println!("我们的数组大小是{:?}", dim);
        if dim.is_some() {
            dims.push(dim.unwrap());
        }
        dafny_ty = Type::Array(Box::new(dafny_ty.clone()));
    }

    let array_new_expr = Expr::ArrayInit(dims, base.clone(), dafny_ty.clone());
    stmts.push(Stmt::DeclVar(Var {
        id: array_name.to_string(),
        type_: dafny_ty.clone(),
        init: Some(array_new_expr),
    }));
    context.declare_var(array_name.to_string(), dafny_ty.clone());
    Some(stmts)
}


/// 现在只能处理一维数组
/// TODO: 需要处理多维数组
fn parse_init_list(node: &Node, context: &mut Context, source: &str, array_id: &str) -> Option<Vec<Stmt>> {
    let mut stmts = vec![];
    let mut counter = 0;
    let array_type = context.lookup_var(array_id).unwrap().clone();
    for child in node.children(&mut node.walk()) {
        if child.kind() == "{" || child.kind() == "}" || child.kind() == "," {
            continue;
        }
        println!("我们的{:?}", child);
        let expr = parse_expr(&child, context, source);
        if expr.is_some() {
            // TODO: Parse init list
            stmts.push(
                Stmt::Assign(Assign { 
                    lhs: Lhs::Index(Box::new(Expr::Primary(PrimaryExpr::Identifier(array_id.to_string()), array_type.clone())), Box::new(Expr::Primary(PrimaryExpr::Literal(Literal::Integer(counter.to_string())), Type::Int))),
                    expr: expr.unwrap(), })
            );
        } else {
            println!("Error parsing init list: {}", child.kind());
            return None;
        }
        counter += 1;
    }
    if stmts.is_empty() {
        return None;
    }
    Some(stmts)
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
        "call_expression" => {
            let call_stmt = parse_call_statement(&node, context, source);
            if call_stmt.is_some() {
                stmts.extend(call_stmt.unwrap());
            }
        }
        "parenthesized_expression" => {
            let stmt = parse_expr_stmt(&node.child(1).unwrap(), context, source);
            if stmt.is_some() {
                stmts.extend(stmt.unwrap());
            }
        }
        _ => {}
    }

    Some(stmts)
}

fn parse_lhs(node: &Node, context: &mut Context, source: &str) -> Option<Lhs> {
    match node.kind() {
        "identifier" => {
            let id_name = node
                .utf8_text(source.as_bytes())
                .expect("Failed to get identifier name");
            let var_ty = context.lookup_var(&id_name).unwrap().clone();
            Some(Lhs::Identifier(id_name.to_string(), var_ty))
        }
        "subscript_expression" => {
            // TODO: multi-dimensional array
            let id_name = node
                .child(0)
                .unwrap()
                .utf8_text(source.as_bytes())
                .expect("Failed to get identifier name");
            println!("我们的id_name: {:?}", id_name);
            let var_ty = context.lookup_var(&id_name).unwrap().clone();
            println!("我们的var_ty: {:?}", var_ty);
            let index = parse_expr(&node.child(2).unwrap(), context, source);
            println!("我们的index: {:?}", index);
            if index.is_some() {
                Some(Lhs::Index(
                    Box::new(Expr::Primary(PrimaryExpr::Identifier(id_name.to_string()), var_ty)),
                    Box::new(index.unwrap()),
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_assign_statement(node: &Node, context: &mut Context, source: &str) -> Option<Stmt> {
    let lhs = node.child(0).unwrap();
    println!("我们的lhs: {:?}", lhs);
    let lhs_expr = parse_expr(&lhs, context, source);
    let lhs = parse_lhs(&lhs, context, source);
    if lhs.is_none() {
        return None;
    }
    let lhs = lhs.unwrap();
    let expr = node.child(2).unwrap();
    let expr = parse_expr(&expr, context, source);
    let expr = match expr {
        Some(expr) => expr,
        None => Expr::Primary(
            PrimaryExpr::Literal(Literal::Integer("0".to_string())),
            lhs.get_type().clone(),
        ),
    };
    
    if lhs_expr.is_none() {
        return None;
    }
    let lhs_expr = lhs_expr.unwrap();
    
    let assign_op = node.child(1).unwrap().kind(); // =, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=
    match assign_op {
        "=" => {
            return Some(Stmt::Assign(Assign {
                lhs: lhs,
                expr: expr,
            }));
        }
        "+=" => {
            let ty = lhs.get_type().clone();
            let expr_ty = max_ty(ty.clone(), expr.get_type().clone());
            return Some(Stmt::Assign(Assign {
                lhs: lhs,
                expr: Expr::Additive(
                    AdditiveOp::Add,
                    Box::new(lhs_expr),
                    Box::new(expr.clone()),
                    expr_ty,
                ),
            }));
        }
        "-=" => {
            let ty = lhs.get_type().clone();
            let expr_ty = max_ty(ty.clone(), expr.get_type().clone());
            return Some(Stmt::Assign(Assign {
                lhs: lhs,
                expr: Expr::Additive(
                    AdditiveOp::Sub,
                    Box::new(lhs_expr),
                    Box::new(expr),
                    expr_ty,
                ),
            }));
        }
        "*=" => {
            let ty = lhs.get_type().clone();
            let expr_ty = max_ty(ty.clone(), expr.get_type().clone());
            return Some(Stmt::Assign(Assign {
                lhs: lhs,
                expr: Expr::Mult(
                    MultOp::Mul,
                    Box::new(lhs_expr),
                    Box::new(expr),
                    expr_ty,
                ),
            }));
        }
        "/=" => {
            let ty = lhs.get_type().clone();
            let expr_ty = max_ty(ty.clone(), expr.get_type().clone());
            return Some(Stmt::Assign(Assign {
                lhs: lhs,
                expr: Expr::Mult(
                    MultOp::Div,
                    Box::new(lhs_expr),
                    Box::new(expr),
                    expr_ty,
                ),
            }));
        }
        "%=" => {
            let ty = lhs.get_type().clone();
            let expr_ty = max_ty(ty.clone(), expr.get_type().clone());
            return Some(Stmt::Assign(Assign {
                lhs: lhs,
                expr: Expr::Mult(
                    MultOp::Mod,
                    Box::new(lhs_expr),
                    Box::new(expr),
                    expr_ty,
                ),
            }));
        }
        "&=" => {
            let ty = lhs.get_type().clone();
            let expr_ty = max_ty(ty.clone(), expr.get_type().clone());
            return Some(Stmt::Assign(Assign {
                lhs: lhs,
                expr: Expr::BitwiseAnd(
                    Box::new(lhs_expr),
                    Box::new(expr),
                    expr_ty,
                ),
            }));
        }
        _ => {}
    } 

    Some(Stmt::Assign(Assign {
        lhs: lhs,
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
            lhs: Lhs::Identifier(id.to_string(), ty.clone()),
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
            lhs: Lhs::Identifier(id.to_string(), ty.clone()),
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

fn parse_for_stmt(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    /*
     * for_statement
     * ├── for : for
     * ├── ( : (
     * ├── assignment_satement : ... can't be empty 
     * │   ├── ...
     * │   └── ; : ;
     * ├── binary_expression : ... can be empty
     * ├── ; : ;
     * ├── update_expression : ... can be empty
     * ├── ) : )
     * └── compound_statement : ...
     */

    let mut init_stmt = vec![];
    let mut cond = vec![];
    let mut update_stmt = vec![];
    let mut body = vec![];

    let mut counter = 0;
    for child in node.children(&mut node.walk()) {
        if child.kind() == ";" {
            counter += 1;
            continue;
        }
        if child.kind() == "(" || child.kind() == "for" {
            continue;
        }
        if child.kind() == ")" {
            counter += 1;
            continue;
        }
        if counter == 0 {
            let stmt = parse_stmt(&child, context, source);
            if stmt.is_some() {
                init_stmt.extend(stmt.unwrap());
            }
            counter += 1;
        } else if counter == 1 {
            let cond_exp = parse_expr(&child, context, source);

            if cond_exp.is_none() {
                cond.push(Expr::Primary(
                    PrimaryExpr::Literal(Literal::Boolean(true)),
                    Type::Bool,
                ));
                continue;
            }
            cond.push(cond_exp.unwrap());
        } else if counter == 2 {
            let stmt = parse_expr_stmt(&child, context, source);
            if stmt.is_some() {
                update_stmt.extend(stmt.unwrap());
            }
        } else if counter == 3 {
            let stmt = parse_stmt(&child, context, source);
            if stmt.is_some() {
                body.extend(stmt.unwrap());
            }
        }
    }

    body.extend(update_stmt);
    let cond = if cond.is_empty() {
        Expr::Primary(PrimaryExpr::Literal(Literal::Boolean(true)), Type::Bool)
    } else {
        cond[0].clone()
    };

    let for_stmt = Stmt::WhileLoop(
        WhileLoop { 
            cond: cond,
            invariants: vec![],
            decreases: vec![Expr::Primary(
                PrimaryExpr::Literal(Literal::Star),
                Type::Star,
            )],
            block: Block {
                stmts: body,
            },
        }
    );

    let mut stmts = vec![];
    stmts.extend(init_stmt);
    stmts.push(for_stmt);
    Some(stmts)
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
    let cond = match cond {
        Some(cond) => cond,
        None => {
            Expr::Primary(PrimaryExpr::Literal(Literal::Boolean(true)), Type::Bool)
        }
    };
    let then_body = match then_body {
        Some(then_body) => then_body,
        None => {
            vec![]
        }
    };
    let if_stmt = Stmt::IfElse(IfElse {
        cond: cond,
        then_block: Block {
            stmts: then_body,
        },
        else_block: match else_body {
            Some(else_body) => Some(Block { stmts: else_body }),
            None => None,
        },
    });
    Some(vec![if_stmt])
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

fn parse_call_statement(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Stmt>> {
    let mut stmts = vec![];

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

    if func_name == "assume" {
        let stmt = Stmt::Assume(
            args[0].clone()
        );
        stmts.push(stmt);
        return Some(stmts);
    }

    let call_stmt = Stmt::Call(
        Call { id: func_name.to_string(), 
            args: args, }
    );
    stmts.push(call_stmt);

    Some(stmts)
}

// TODO: Parse expressions
// Don't parse the comma expression, I don't know the best way to do it
fn parse_expr(node: &Node, context: &mut Context, source: &str) -> Option<Expr> {
    let exp = match node.kind() {
        "parenthesized_expression" => parse_expr(&node.child(1).unwrap(), context, source),
        "binary_expression" => parse_binary_expr(node, context, source),
        "identifier" => {
            let id = node.utf8_text(source.as_bytes()).unwrap();
            let val_ty = context.lookup_macro(id);
            if val_ty.is_some() {
                let val = &val_ty.unwrap().0;
                let ty = &val_ty.unwrap().1;
                match ty {
                    Type::Int => {
                        return Some(
                            Expr::Primary(PrimaryExpr::Literal(Literal::Integer(val.clone())), ty.clone())
                        );
                    }
                    Type::Bool => {
                        let bool_val = if val == "true" {
                            true
                        } else {
                            false
                        };
                        return Some(
                            Expr::Primary(PrimaryExpr::Literal(Literal::Boolean(bool_val)), ty.clone())
                        );
                    }
                    Type::Real => {
                        let float_val = val.parse::<f64>().unwrap();
                        return Some(
                            Expr::Primary(PrimaryExpr::Literal(Literal::Real(float_val)), ty.clone())
                        );
                    }
                    _ => {}
                }
            }
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
        "false" => Some(Expr::Primary(
            PrimaryExpr::Literal(Literal::Boolean(false)),
            Type::Bool,
        )),
        "true" => Some(Expr::Primary(
            PrimaryExpr::Literal(Literal::Boolean(true)),
            Type::Bool,
        )),
        "subscript_expression" => {
            parse_subscript_expr(node, context, source)
        }
        "initializer_list" => {
            // TODO: Parse initializer list
            unimplemented!()
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

    if func_name == "unknown_int" 
        || func_name == "unknown" 
        || func_name == "unknown_uint"
        || func_name == "unknown_bool" 
        || func_name == "unknown_uchar"
        || func_name == "unknown1"
        || func_name == "unknown2"
        || func_name == "unknown3"
        || func_name == "unknown_ushort"
        || func_name == "unknown_float" {
        return Some(Expr::Primary(PrimaryExpr::Literal(Literal::Star), Type::Star));
    }

    if func_name == "assume" {
        return None;
    }
    let func_ty = context.lookup_var(func_name);
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

fn parse_subscript_expr(node: &Node, context: &mut Context, source: &str) -> Option<Expr> {
    let name = node.child(0).unwrap();
    let name = name.utf8_text(source.as_bytes()).unwrap();
    let index = parse_expr(&node.child(2).unwrap(), context, source);
    if index.is_none() {
        return None;
    }
    let index = index.unwrap();
    let ty = context.lookup_var(name).unwrap().clone();
    let base_ty = ty.get_base_type();

    Some(Expr::Primary(PrimaryExpr::Index(Box::new(Expr::Primary(PrimaryExpr::Identifier(name.to_string()), ty)), Box::new(index)), base_ty))
}