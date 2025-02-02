mod traverse;
mod converter;
mod dafny_ast;
mod context;
mod printer;
mod preprocess;
mod llm_checker;

use context::Context;
use llm_checker::fix_code_with_python;
use preprocess::delete_decreases_star;
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

    let matches = Command::new("CParser")
        .version("0.1.0")
        .about("Parses a C file using tree-sitter and translates it to Dafny")
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
        .arg(
          Arg::new("termination")
          .short('t')
          .long("termination")
          .value_name("BOOL")
          .help("Check termination of the program")
          .required(true)
        )
        .arg(
          Arg::new("llm")
          .short('l')
          .long("llm")
          .value_name("BOOL")
          .help("Check syntax error of the program with llm")
          .required(true)
        )
        .get_matches();

    let input_file: &String = matches.get_one("input").expect("input file not specified");
    let output_file: &String = matches.get_one("output").expect("output file not specified");

    let termination = matches
      .get_one::<String>("termination")
      .expect("termination not specified")
      .parse::<bool>()
      .expect("termination not a bool");
    
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

    let predef = r"
function to_bv32(n: int): bv32
  requires -0x80000000 <= n < 0x80000000
{
  if n >= 0 then
    n as bv32
  else
    (n + 0x100000000) as bv32  // 转换为补码形式
}

";

    let mut dafny_code = DafnyPrinter::print_program(&program);

    if termination.clone() {
      println!("Checking termination of the program. ☺");
      dafny_code = delete_decreases_star(&dafny_code);
    } else {
      println!("Not checking termination of the program. 😆");
    }

    println!("{}{}", predef, dafny_code);
    write!(output, "{}{}", predef, dafny_code)?;

    let llm = matches
      .get_one::<String>("llm")
      .expect("llm not specified")
      .parse::<bool>()
      .expect("llm not a bool");
    if llm {
      // fix code with python
      fix_code_with_python(output_file);
      let output_code = fs::read_to_string(output_file).expect("Error reading output file");
      println!("{}", output_code);
    }
    

    println!("Conversion finished! 😀");
    Ok(())
}
