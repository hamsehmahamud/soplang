mod shell;

use std::fs;
use std::path::PathBuf;
use std::process;

use clap::Parser as ClapParser;

use soplang::{format_error_with_source, run_source, Interpreter};

#[derive(ClapParser)]
#[command(name = "soplang", about = "The Somali Programming Language", version)]
struct Cli {
    #[arg(short, long, help = "Execute code snippet and exit")]
    command: Option<String>,

    #[arg(short, long, help = "Execute file and exit")]
    file: Option<PathBuf>,

    #[arg(short, long, help = "Run example N from examples/ (1-based)")]
    example: Option<usize>,

    #[arg(short, long, help = "Open interactive shell after running file/example")]
    interactive: bool,

    #[arg(help = "Path to .sop file")]
    filename: Option<PathBuf>,

    #[arg(long, help = "Print AST instead of executing")]
    ast: bool,

    #[arg(long, help = "Dump HIR (High-Level IR) instead of executing")]
    hir: bool,

    #[arg(long, help = "Run via Cranelift JIT (compiled) instead of interpreter")]
    jit: bool,
}

fn main() {
    let cli = Cli::parse();

    let run_then_maybe_shell = |path: PathBuf, source: String| {
        let mut interp = Interpreter::new();
        match run_source(&mut interp, &source, Some(&path), cli.ast, cli.hir, cli.jit) {
            Ok(()) => {
                if cli.interactive {
                    run_shell();
                }
            }
            Err(e) => {
                eprintln!("{}", format_error_with_source(&e, Some(&source)));
                process::exit(1);
            }
        }
    };

    if let Some(code) = &cli.command {
        let mut interp = Interpreter::new();
        match run_source(&mut interp, code, None, false, false, false) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("{}", format_error_with_source(&e, Some(code)));
                process::exit(1);
            }
        }
    }

    if let Some(ref path) = cli.file {
        match fs::read_to_string(path) {
            Ok(source) => run_then_maybe_shell(path.clone(), source),
            Err(e) => {
                eprintln!("Khalad: Ma akhriyin faylka '{}': {}", path.display(), e);
                process::exit(1);
            }
        }
        return;
    }

    if let Some(n) = cli.example {
        let path = match example_path(n) {
            Some(p) => p,
            None => {
                eprintln!("Khalad: Tusaale {} ma jiro (examples/)", n);
                process::exit(1);
            }
        };
        match fs::read_to_string(&path) {
            Ok(source) => run_then_maybe_shell(path, source),
            Err(e) => {
                eprintln!("Khalad: Ma akhriyin faylka: {}", e);
                process::exit(1);
            }
        }
        return;
    }

    if let Some(ref path) = cli.filename {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Khalad: Ma akhriyin faylka '{}': {}", path.display(), e);
                process::exit(1);
            }
        };
        let mut interp = Interpreter::new();
        match run_source(&mut interp, &source, Some(path.as_path()), cli.ast, cli.hir, cli.jit) {
            Ok(()) => {
                if cli.interactive {
                    run_shell();
                }
            }
            Err(e) => {
                eprintln!("{}", format_error_with_source(&e, Some(&source)));
                process::exit(1);
            }
        }
        return;
    }

    run_shell();
}

fn run_shell() {
    let mut sh = shell::Shell::new();
    sh.run();
}

/// Resolve example N (1-based) to PathBuf. Sorts .sop files in examples/ by name.
fn example_path(n: usize) -> Option<PathBuf> {
    if n == 0 {
        return None;
    }
    let mut paths: Vec<PathBuf> = fs::read_dir("examples")
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "sop").unwrap_or(false))
        .collect();
    paths.sort();
    paths.into_iter().nth(n - 1)
}
