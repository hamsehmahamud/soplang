//! Phase 5 AOT backend entrypoint.
//!
//! This backend builds a standalone native executable by generating a small
//! Rust runner that embeds Soplang source, then compiling it in release mode.
//! The produced executable is native (AOT) and can run independently.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{runtime_error, SoplangError};

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
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| runtime_error(e.to_string(), 0, 0))?
            .as_millis();
        let pkg_name = format!("soplang_aot_{}_{}", std::process::id(), now);
        let temp_root = std::env::temp_dir().join(&pkg_name);
        let src_dir = temp_root.join("src");
        fs::create_dir_all(&src_dir).map_err(|e| runtime_error(e.to_string(), 0, 0))?;

        let cargo_toml = format!(
            "[package]\nname = \"{pkg}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nsoplang = {{ path = \"{path}\" }}\n",
            pkg = pkg_name,
            path = manifest_dir.display()
        );
        fs::write(temp_root.join("Cargo.toml"), cargo_toml)
            .map_err(|e| runtime_error(e.to_string(), 0, 0))?;

        let escaped_source = format!("{source:?}");
        let runner = format!(
            "use std::process;\nuse soplang::{{format_error_with_source, run_source}};\n\nconst SOURCE: &str = {src};\n\nfn main() {{\n    match run_source(SOURCE, None, false, false, {strict}) {{\n        Ok(()) => {{}}\n        Err(e) => {{\n            eprintln!(\"{{}}\", format_error_with_source(&e, Some(SOURCE)));\n            process::exit(1);\n        }}\n    }}\n}}\n",
            src = escaped_source,
            strict = if strict { "true" } else { "false" }
        );
        fs::write(src_dir.join("main.rs"), runner).map_err(|e| runtime_error(e.to_string(), 0, 0))?;

        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--manifest-path").arg(temp_root.join("Cargo.toml"));
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
            format!("{}.exe", pkg_name)
        } else {
            pkg_name.clone()
        };
        let profile = if opt_level == 0 { "debug" } else { "release" };
        let built = temp_root.join("target").join(profile).join(exe_name);
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
