use tree_sitter::{Node, Tree};

use crate::{
    context::{self, Context},
    dafny_ast::*,
};

/// 将 C 语法树 转换为 Dafny AST
pub fn convert(tree: Tree, context: &mut Context, source: &str) -> Program {
    let mut global_vars = vec![];
    let mut methods = vec![];

    let root_node = tree.root_node();

    for child in root_node.children(&mut root_node.walk()) {
        println!("{}", child.kind());
    }

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
        println!("parse_global_var: {:?}", decl);
        let mut name = decl.utf8_text(source.as_bytes());
        let mut init = None;
        println!(
            "xxxx {:?} {:?}",
            decl.field_name_for_child(2),
            decl.child_by_field_name("value")
        );
        if let Some(init_value) = decl.child_by_field_name("value") {
            name = decl
                .child_by_field_name("declarator")?
                .utf8_text(source.as_bytes());
            init = parse_expr(&init_value, context, source);
        }

        println!("name: {:?}", name);
        match name {
            Ok(name) => {
                let ty = node.child_by_field_name("type")?;
                println!("ty: {:?}", ty);
                let dafny_ty = parse_type(&ty, context, source)?;
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
    println!("Parsing type: {:?}", ty_name);
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
    println!("Func params: {:?}", func_params);

    // 必须要有函数体
    let func_body = node.child_by_field_name("body")?;
    println!("Func body***: {:?}", func_body);
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

    let modify_expr = Expr::Primary(PrimaryExpr::Literal(Literal::This));

    let decrease_expr = Expr::Primary(PrimaryExpr::Literal(Literal::Star));

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

    println!("METHOD {:?}", method_decl);

    Some(method_decl)
}

fn parse_func_name(node: &Node, context: &mut Context, source: &str) -> Option<String> {
    // 获取函数名
    let func_decl = node.child_by_field_name("declarator")?;
    let func_name = func_decl.child_by_field_name("declarator")?;
    let func_name = func_name.utf8_text(source.as_bytes()).unwrap();
    println!("Parsing function: {:?}", func_name);
    Some(func_name.to_string())
}

fn parse_func_type(node: &Node, context: &mut Context, source: &str) -> Option<Type> {
    // 获取函数类型
    let func_type = node.child_by_field_name("type")?;
    let func_type = parse_type(&func_type, context, source)?;
    println!("Function type: {:?}", func_type);
    Some(func_type)
}

fn parse_func_params(node: &Node, context: &mut Context, source: &str) -> Option<Vec<Param>> {
    // 获取函数参数列表
    let func_decl = node.child_by_field_name("declarator")?;
    let func_params = func_decl.child_by_field_name("parameters")?;
    println!("func_params: {:?}", func_params);

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
            // TODO: Parse while statement
            vec![]
        }
        _ => vec![], // TODO: Parse other statements
    };

    stmts.extend(stmt);

    Some(stmts)
}

// TODO: Parse expressions
fn parse_expr(node: &Node, context: &Context, source: &str) -> Option<Expr> {
    None
}

/// 生成构造函数
fn generate_constructor(vars: &Vec<FieldDecl>, context: &Context, source: &str) -> ConstructorDecl {
    let modify_expr = Expr::Primary(PrimaryExpr::Literal(Literal::This));

    let mut stmts = vec![];
    for var in vars {
        if var.init.is_some() {
            let stmt = Stmt::Assign(Assign {
                lhs: Lhs::Identifier(var.id.to_string()),
                expr: var.init.clone().unwrap(),
            });
            stmts.push(stmt);
        } else {
            let zero_expr = Expr::Primary(PrimaryExpr::Literal(Literal::Integer("0".to_string())));
            let stmt = Stmt::Assign(Assign {
                lhs: Lhs::Identifier(var.id.to_string()),
                expr: zero_expr,
            });
            stmts.push(stmt);
        }
    }

    println!("Constructor {:?}", stmts);

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
                println!(
                    "stmt return id {:?}",
                    child_node.utf8_text(source.as_bytes())
                );
                ret_expr = Some(Expr::Primary(PrimaryExpr::Identifier(
                    child_node
                        .utf8_text(source.as_bytes())
                        .expect("identifier failed")
                        .to_string(),
                )));
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
        println!("parse_declaration: {:?}", decl);
        let mut name = decl.utf8_text(source.as_bytes());
        let mut init = None;
        println!(
            "xxxx {:?} {:?}",
            decl.field_name_for_child(2),
            decl.child_by_field_name("value")
        );
        if let Some(init_value) = decl.child_by_field_name("value") {
            name = decl
                .child_by_field_name("declarator")?
                .utf8_text(source.as_bytes());
            init = parse_expr(&init_value, context, source);
        }

        println!("name: {:?}", name);
        match name {
            Ok(name) => {
                let ty = node.child_by_field_name("type")?;
                println!("ty: {:?}", ty);
                let dafny_ty = parse_type(&ty, context, source)?;
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
    println!("parse_expr_stmt: {:?}", node.child(0));
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
        _ => {}
    }

    Some(stmts)
}

fn parse_assign_statement(node: &Node, context: &mut Context, source: &str) -> Option<Stmt> {
    let id = node.child(0).unwrap();
    let expr = node.child(2).unwrap();

    println!(
        "assign_statement: {:?} = {:?}",
        id.utf8_text(source.as_bytes()),
        expr.utf8_text(source.as_bytes())
    );

    let id_name = id
        .utf8_text(source.as_bytes())
        .expect("Failed to get identifier name");

    let expr = parse_expr(&expr, context, source);
    let expr = match expr {
        Some(expr) => expr,
        None => Expr::Primary(PrimaryExpr::Literal(Literal::Integer("0".to_string()))),
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
        println!("child: {:?}", child);
        let stmt = parse_expr_stmt(&child, context, source);
        if stmt.is_some() {
            stmts.extend(stmt.unwrap());
        }
    }
   Some(stmts)
}