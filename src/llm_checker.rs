use std::process::Command;

/// 调用一个 Python 脚本来修复代码
pub fn fix_code_with_python(dafny_file: &str) {
    let output = Command::new("python")
        .arg("src/checker.py")
        .arg(dafny_file)
        .output()
        .expect("Failed to execute python");
    if !output.status.success() {
        // 打印标准错误输出，帮助调试
        let error_message = String::from_utf8_lossy(&output.stderr);
        panic!("Python script failed with error: {}", error_message);
    }
    println!("{}", String::from_utf8(output.stdout).expect("Failed to convert output to string"));
}