//! Phase 5 AOT backend entrypoint.
//!
//! This backend builds a standalone native executable by generating a small
//! Rust runner that embeds Soplang source, then compiling it in release mode.
//! The produced executable is native (AOT) and can run independently.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{runtime_error, SoplangError};

/// Fixed package name and workspace dir so Cargo can reuse the soplang dependency
/// and only rebuild the runner when the generated main.rs changes.
const AOT_PKG_NAME: &str = "soplang_aot_runner";

pub struct LlvmBackend;

impl LlvmBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn build_executable(
        &self,
        source: &str,
        out_path: &Path,
        opt_level: u8,
        strict: bool,
    ) -> Result<(), SoplangError> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let aot_root = manifest_dir.join("target").join(AOT_PKG_NAME);
        let src_dir = aot_root.join("src");
        fs::create_dir_all(&src_dir).map_err(|e| runtime_error(e.to_string(), 0, 0))?;

        let cargo_toml_path = aot_root.join("Cargo.toml");
        let cargo_toml = format!(
            "[package]\nname = \"{pkg}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nsoplang = {{ path = \"{path}\" }}\n",
            pkg = AOT_PKG_NAME,
            path = manifest_dir.display()
        );
        fs::write(&cargo_toml_path, cargo_toml)
            .map_err(|e| runtime_error(e.to_string(), 0, 0))?;

        let escaped_source = format!("{source:?}");
        let runner = format!(
            "use std::process;\nuse soplang::{{format_error_with_source, run_source}};\n\nconst SOURCE: &str = {src};\n\nfn main() {{\n    match run_source(SOURCE, None, false, false, {strict}) {{\n        Ok(()) => {{}}\n        Err(e) => {{\n            eprintln!(\"{{}}\", format_error_with_source(&e, Some(SOURCE)));\n            process::exit(1);\n        }}\n    }}\n}}\n",
            src = escaped_source,
            strict = if strict { "true" } else { "false" }
        );
        fs::write(src_dir.join("main.rs"), runner).map_err(|e| runtime_error(e.to_string(), 0, 0))?;

        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--manifest-path").arg(&cargo_toml_path);
        if opt_level > 0 {
            cmd.arg("--release")
                .env("CARGO_PROFILE_RELEASE_OPT_LEVEL", format!("{}", opt_level.min(3)));
        }
        let status = cmd
            .status()
            .map_err(|e| runtime_error(format!("AOT build failed to start: {}", e), 0, 0))?;
        if !status.success() {
            return Err(runtime_error("AOT build failed", 0, 0));
        }

        let exe_name = if cfg!(windows) {
            format!("{}.exe", AOT_PKG_NAME)
        } else {
            AOT_PKG_NAME.to_string()
        };
        let profile = if opt_level == 0 { "debug" } else { "release" };
        let built = aot_root.join("target").join(profile).join(&exe_name);
        if !built.exists() {
            return Err(runtime_error("AOT binary not found after build", 0, 0));
        }

        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
            }
        }
        fs::copy(&built, out_path).map_err(|e| runtime_error(e.to_string(), 0, 0))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(out_path)
                .map_err(|e| runtime_error(e.to_string(), 0, 0))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(out_path, perms).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
        }

        Ok(())
    }
}
