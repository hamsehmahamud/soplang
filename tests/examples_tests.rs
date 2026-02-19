//! Integration tests: run example files, assert stdout.
//! 7.1: ad-hoc example checks. 7.2: each example vs .expected file.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run_file(path: &Path) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .arg(path)
        .output()
        .expect("run soplang");
    String::from_utf8(out.stdout).unwrap()
}

/// Path to .expected for a .sop file (e.g. examples/hello.sop -> examples/hello.expected).
fn expected_path(sop_path: &Path) -> PathBuf {
    sop_path
        .with_file_name(sop_path.file_stem().unwrap_or_default().to_string_lossy().to_string() + ".expected")
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

/// Phase 7.2: Run each examples/*.sop that has a .expected file; assert stdout matches.
/// Regenerate .expected: for f in examples/*.sop; do stem=$(basename "$f" .sop); ./target/debug/soplang "$f" 2>/dev/null > "examples/${stem}.expected"; done
/// Skip 43_random_function.expected (non-deterministic).
#[test]
fn test_each_example_against_expected() {
    let examples_dir = Path::new("examples");
    let dir = match fs::read_dir(examples_dir) {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut sop_paths: Vec<PathBuf> = dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "sop").unwrap_or(false))
        .collect();
    sop_paths.sort();

    for sop_path in sop_paths {
        let expected_path = expected_path(&sop_path);
        if !expected_path.exists() {
            continue;
        }
        let expected = fs::read_to_string(&expected_path).unwrap_or_default();
        let actual = run_file(&sop_path);
        assert_eq!(
            actual,
            expected,
            "example {} stdout did not match {}",
            sop_path.display(),
            expected_path.display()
        );
    }
}
