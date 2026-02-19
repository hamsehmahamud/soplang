//! Integration tests: run example files, assert stdout.

use std::path::Path;
use std::process::Command;

fn run_file(path: &Path) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .arg(path)
        .output()
        .expect("run soplang");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn test_example_hello() {
    let path = Path::new("examples/hello.sop");
    if !path.exists() {
        return;
    }
    let output = run_file(path);
    assert_eq!(output, "Salaan, Adduunka!\n");
}

#[test]
fn test_example_01_dynamic_typing() {
    let path = Path::new("examples/01_dynamic_typing.sop");
    if !path.exists() {
        return;
    }
    let output = run_file(path);
    assert!(output.contains("Initial value (number): 10"));
    assert!(output.contains("Changed value (string): waa qoraal hadda"));
}

#[test]
fn test_example_03_type_checking() {
    let path = Path::new("examples/03_type_checking.sop");
    if !path.exists() {
        return;
    }
    let output = run_file(path);
    assert!(output.contains("Type of integer_var: abn"));
    assert!(output.contains("Type of list_var: teed"));
}
