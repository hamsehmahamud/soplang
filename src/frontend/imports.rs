//! Resolve `keen` imports by loading and flattening imported modules at compile time.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{codes, import_error_ex, ErrorMeta, SoplangError};
use crate::frontend::ast::Stmt;
use crate::frontend::{Lexer, Parser};

/// Expand `Stmt::Import` nodes into the imported module's top-level statements.
pub fn resolve_imports(
    stmts: Vec<Stmt>,
    source_path: Option<&Path>,
) -> Result<Vec<Stmt>, SoplangError> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    if let Some(path) = source_path {
        if let Ok(canon) = path.canonicalize() {
            seen.insert(canon);
        }
    }
    flatten_imports(stmts, source_path, &mut out, &mut seen)?;
    Ok(out)
}

fn flatten_imports(
    stmts: Vec<Stmt>,
    source_path: Option<&Path>,
    out: &mut Vec<Stmt>,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), SoplangError> {
    let base_dir = source_path
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    for stmt in stmts {
        match stmt {
            Stmt::Import(path) => {
                let import_path = resolve_import_path(&base_dir, &path)?;
                let canon = import_path.canonicalize().map_err(|_| {
                    import_error_ex(
                        format!("Faylka '{}' ma helin", path),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Hubi in magaca faylka iyo jidka ay sax yihiin."),
                    )
                })?;
                if !seen.insert(canon) {
                    continue;
                }
                let source = fs::read_to_string(&import_path).map_err(|_| {
                    import_error_ex(
                        format!("Faylka '{}' ma helin", path),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Hubi in faylka uu jiro oo akhriska loo ogol yahay."),
                    )
                })?;
                let tokens = Lexer::new(&source).tokenize()?;
                let imported = Parser::new(tokens).parse()?;
                flatten_imports(imported, Some(&import_path), out, seen)?;
            }
            other => out.push(other),
        }
    }
    Ok(())
}

fn resolve_import_path(base_dir: &Path, path: &str) -> Result<PathBuf, SoplangError> {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        Ok(base_dir.join(candidate))
    }
}
