//! Phase 1 (COMPILER_PLAN): semantic analysis passes on all example files.

use std::fs;
use std::path::Path;

use soplang::{semantic, Lexer, Parser};

fn run_semantic(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let tokens = Lexer::new(&source).tokenize().map_err(|e| e.to_string())?;
    let stmts = Parser::new(tokens).parse().map_err(|e| e.to_string())?;
    semantic::analyze(&stmts).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn test_semantic_all_examples() {
    let examples_dir = Path::new("examples");
    if !examples_dir.is_dir() {
        return;
    }
    let mut sop_files: Vec<_> = fs::read_dir(examples_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sop"))
        .map(|e| e.path())
        .collect();
    sop_files.sort();

    for path in sop_files {
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        // Skip examples that are intentionally type-error tests.
        if name.contains("type_error") || name.contains("reassignment") {
            continue;
        }
        let result = run_semantic(&path);
        assert!(
            result.is_ok(),
            "semantic analysis should pass for {}: {:?}",
            path.display(),
            result.err()
        );
    }
}
