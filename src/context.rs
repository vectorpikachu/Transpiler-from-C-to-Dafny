use std::collections::HashMap;
use crate::dafny_ast::*;

// 全局上下文管理
#[derive(Debug, Clone)]
pub struct Context {
    tmp_var_gen: TempVarGenerator,
    current_method: Option<String>,
    scope_stack: Vec<HashMap<String, Type>>, // include all the methods
    macros: HashMap<String, (String, Type)>, // the identifier and the macro string
}

impl Context {
    pub fn new() -> Self {
        Self {
            tmp_var_gen: TempVarGenerator::new("tmp_"),
            current_method: None,
            scope_stack: vec![HashMap::new()],
            macros: HashMap::new(),
        }
    }

    pub fn enter_method(&mut self, method_name: String) {
        self.current_method = Some(method_name);
    }

    pub fn exit_method(&mut self) {
        self.current_method = None;
    }

    pub fn get_tmp_var(&mut self) -> String {
        self.tmp_var_gen.next()
    }

    pub fn get_current_method(&self) -> Option<&String> {
        self.current_method.as_ref()
    }

    pub fn get_current_method_mut(&mut self) -> Option<&mut String> {
        self.current_method.as_mut()
    }

    pub fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn declare_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, ty);
        }
    }


    pub fn lookup_var(&self, name: &str) -> Option<&Type> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// a very very ugly implementation of macro
    pub fn insert_macro(&mut self, name: String, content: String) {
        let ty = if content.as_str().contains(".") {
            Type::Real
        } else if content.as_str().contains("true") || content.as_str().contains("false") {
            Type::Bool
        } else {
            Type::Int
        };

        // 如果有最外层的括号的话, 删除掉这个 macro 最外层的括号
        let content = if content.starts_with("(") && content.ends_with(")") {
            content[1..content.len()-1].to_string()
        } else {
            content
        };

        self.macros.insert(name, (content, ty));
    }

    pub fn lookup_macro(&self, name: &str) -> Option<&(String, Type)> {
        self.macros.get(name)
    }

}



#[derive(Debug, Clone)]
pub struct TempVarGenerator {
    counter: u32, // 计数器，用于生成唯一的变量名
    prefix: String, // 临时变量前缀
}

impl TempVarGenerator {
    // 构造函数
    pub fn new(prefix: &str) -> Self {
        Self {
            counter: 0,
            prefix: prefix.to_string(),
        }
    }

    // 生成下一个临时变量名
    pub fn next(&mut self) -> String {
        let name = format!("{}{}", self.prefix, self.counter);
        self.counter += 1; // 递增计数器
        name
    }

    // 重置计数器（可选）
    pub fn reset(&mut self) {
        self.counter = 0;
    }
}