//! delete the comment to expose the assertion
//! 从注释中抽取 assertion

use regex::Regex;

/// delete the comment to expose the assertion
pub fn extract_assertion(input: &str) -> String {
    let re_comment = Regex::new(r"//@").unwrap();
    let current_str = re_comment.replace_all(input, "").to_string();
    let re_false = Regex::new(r"\\false").unwrap();
    let current_str = re_false.replace_all(&current_str, "false").to_string();
    current_str
}