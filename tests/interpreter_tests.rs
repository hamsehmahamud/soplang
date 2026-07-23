//! Integration tests: run short programs via binary, assert stdout.

use std::process::Command;

/// Run program source with -c and return stdout as string.
fn run_program(source: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .arg("-c")
        .arg(source)
        .output()
        .expect("run soplang");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn test_hello_world() {
    let output = run_program(r#"qor("Salaan, Adduunka!")"#);
    assert_eq!(output, "Salaan, Adduunka!\n");
}

#[test]
fn test_qor_number() {
    let output = run_program("qor(1+2)");
    assert_eq!(output, "3\n");
}

#[test]
fn test_for_loop() {
    let output = run_program(
        r#"
        kuceli (i 1 ilaa 4) {
            qor(i)
        }
    "#,
    );
    assert_eq!(output, "1\n2\n3\n4\n");
}

#[test]
fn test_nooc() {
    let output = run_program("qor(nooc(5))");
    assert_eq!(output, "abn\n");
}

#[test]
fn test_teed_dherer() {
    let output = run_program("qor(teed(1,2,3).dherer())");
    assert_eq!(output, "3\n");
}

#[test]
fn test_oop_class_example() {
    let output = run_program(
        r#"
        qaab Bisad {
            hawl dhaw(magac) { nafta.magac = magac }
            hawl magaca() { celi nafta.magac }
        }
        door b = cusub Bisad("Luna")
        qor(b.magaca())
    "#,
    );
    assert_eq!(output, "Luna\n");
}

#[test]
fn test_try_catch_runtime() {
    let output = run_program(
        r#"
        fasax { qor(1/0) } qabo (e) { qor("ok") }
    "#,
    );
    assert_eq!(output, "ok\n");
}

#[test]
fn test_import_via_file() {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = std::process::Command::new(exe)
        .arg("examples/16_import.sop")
        .output()
        .expect("run import example");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, "6\n");
}
