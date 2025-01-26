use tree_sitter::{Node, Tree};

use crate::{context::Context, dafny_ast::*};


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
                    global_vars.push(global);
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
fn parse_global_var(node: &Node, context: &mut Context, source: &str) -> Option<FieldDecl> {
    let decl = node
        .child_by_field_name("declarator")?;
    if decl.kind() == "function_declarator" {
        return None;
    }
    println!("parse_global_var: {:?}", decl);
    let mut name = decl.utf8_text(source.as_bytes());
    let mut init = None;
    println!("xxxx {:?} {:?}", decl.field_name_for_child(2), decl.child_by_field_name("value"));
    if let Some(init_value) = decl.child_by_field_name("value") {
        name = decl.child_by_field_name("declarator")?.utf8_text(source.as_bytes());
        init = parse_expr(&init_value, context, source);
    }
    
    println!("name: {:?}", name);
    match name {
        Ok(name) => {
            let ty = node.child_by_field_name("type")?;
            println!("ty: {:?}", ty);
            let dafny_ty = parse_type(&ty, context, source)?;
            Some(FieldDecl {
                id: name.to_string(),
                type_: dafny_ty,
                init: init,
            })
        }
        Err(e) => {
            println!("Error parsing field name: {}", e);
            None
        }
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
    let mut stmts = vec![];
    for child in func_body.children(&mut func_body.walk()) {
        let stmt = parse_stmt(&child, context, source);
        if stmt.is_none() {
            continue;
        }
        stmts.push(stmt.unwrap());
    }

    let returns = match func_type.clone() {
        Some(t) => {
            vec![ReturnVar {id: "ret".to_string(), type_: t}]
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


// todo
fn parse_stmt(node: &Node, context: &Context, source: &str) -> Option<Stmt> {
    None
}


// todo
fn parse_expr(node: &Node, context: &Context, source: &str) -> Option<Expr> {
    None
}

/// 生成构造函数
fn generate_constructor(vars: &Vec<FieldDecl>, context: &Context, source: &str) -> ConstructorDecl {
    let modify_expr = Expr::Primary(PrimaryExpr::Literal(Literal::This));
    
    let mut stmts = vec![];
    for var in vars {
        if var.init.is_some() {
            let stmt = Stmt::Assign(Assign { lhs: Lhs::Identifier(var.id.to_string()), expr: var.init.clone().unwrap() });
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
