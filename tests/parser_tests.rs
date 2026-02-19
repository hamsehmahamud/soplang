//! Integration tests: parse snippets, assert AST structure.

use soplang::{Expr, Lexer, Parser, Stmt};

#[test]
fn test_parse_hello() {
    let source = r#"qor("Salaan, Adduunka!")"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 1);
    if let Stmt::Expr(Expr::Call { name, args }) = &stmts[0] {
        assert_eq!(name, "qor");
        assert_eq!(args.len(), 1);
    } else {
        panic!("expected qor(...) statement");
    }
}

#[test]
fn test_parse_var_decl() {
    let source = "door x = 42";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 1);
    if let Stmt::VarDecl { name, value, .. } = &stmts[0] {
        assert_eq!(name, "x");
        assert!(matches!(value, Expr::Literal(_)));
    } else {
        panic!("expected var decl");
    }
}

#[test]
fn test_parse_if_then() {
    let source = r#"
        haddii (run) {
            qor(1)
        }
    "#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::If { .. }));
}
