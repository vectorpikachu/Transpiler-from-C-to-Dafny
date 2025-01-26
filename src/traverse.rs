use tree_sitter::{Node, TreeCursor};
use std::fs;
use std::io::{self, Write};

/// 递归遍历语法树并将结果写入输出文件
pub fn traverse_tree(node: &Node, source: &str, depth: usize, output: &mut fs::File) -> io::Result<()> {
    // 获取当前节点的信息
    let kind = node.kind();
    let start = node.start_position();
    let end = node.end_position();
    let text = node.utf8_text(source.as_bytes()).unwrap();

    // 将节点信息写入输出文件
    writeln!(
        output,
        "{:indent$}{}: {} ({}:{} - {}:{})",
        "",
        kind,
        text,
        start.row + 1,
        start.column + 1,
        end.row + 1,
        end.column + 1,
        indent = depth * 2
    )?;

    // 递归遍历子节点
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            traverse_tree(&cursor.node(), source, depth + 1, output)?;
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    Ok(())
}