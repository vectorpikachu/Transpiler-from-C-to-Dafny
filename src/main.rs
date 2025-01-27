mod traverse;
mod converter;
mod dafny_ast;
mod context;
mod printer;
mod preprocess;

use context::Context;
use preprocess::extract_assertion;
use tree_sitter::Parser;
use tree_sitter_c::LANGUAGE;
use std::fs;
use clap::{Arg, Command};
use std::io::{Result, Write};

use crate::traverse::traverse_tree;
use crate::converter::*;
use crate::printer::*;


fn main() -> Result<()> {

    let matches = Command::new("C-parser")
        .version("1.0")
        .about("Parses a C file using tree-sitter and outputs the syntax tree")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Sets the input C file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output file")
                .required(true),
        )
        .get_matches();

    let input_file: &String = matches.get_one("input").expect("input file not specified");
    let output_file: &String = matches.get_one("output").expect("output file not specified");

    let c_code = fs::read_to_string(input_file).expect("Error reading input file");
    let c_code = extract_assertion(&c_code);

    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).expect("Error loading C grammar");

    let tree = parser.parse(&c_code, None).expect("Failed to parse code");

    // 获取语法树的根节点
    let root_node = tree.root_node();

    // 打开输出文件
    let mut output = fs::File::create(output_file)?;

    let mut syntax_file = fs::File::create("syntax_tree.txt")?;

    // 遍历语法树并将结果写入输出文件
    writeln!(syntax_file, "Syntax tree:")?;

    traverse_tree(&root_node, &c_code, 0, &mut syntax_file)?;
    
    let mut context = Context::new();
    let program = convert(tree, &mut context, &c_code);

    let dafny_code = DafnyPrinter::print_program(&program);
    println!("{}", dafny_code);
    write!(output, "{}", dafny_code)?;

    println!("Hello, world!");
    Ok(())
}
