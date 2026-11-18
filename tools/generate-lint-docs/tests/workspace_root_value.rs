//! `--workspace-root` with no value used to fall back to the current
//! directory and regenerate the docs there (issue #1073). It must be a usage
//! error that writes nothing.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const SINGLE_LINT: &str = r#"
rustc_session::declare_lint! {
    pub SOROBAN_STORAGE_IN_LOOP,
    Deny,
    "storage operations inside a loop"
}
"#;

/// A directory that looks like a workspace root, so a run that wrongly falls
/// back to it as the current directory would succeed and write docs into it.
fn fake_workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp workspace");
    let lib = dir.path().join("soroban_cost_lints/src/lib.rs");
    fs::create_dir_all(lib.parent().unwrap()).unwrap();
    fs::write(&lib, SINGLE_LINT).unwrap();
    dir
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_generate-lint-docs"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run generate-lint-docs")
}

fn assert_usage_error_writes_nothing(cwd: &Path, args: &[&str]) {
    let output = run(cwd, args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "args {args:?}, stderr: {stderr}"
    );
    assert!(
        stderr.contains("--workspace-root requires a path"),
        "args {args:?}, stderr: {stderr}"
    );
    assert!(
        !cwd.join("docs").exists(),
        "args {args:?} must not write any docs"
    );
}

#[test]
fn dangling_workspace_root_is_a_usage_error() {
    let cwd = fake_workspace();
    assert_usage_error_writes_nothing(cwd.path(), &["--workspace-root"]);
}

#[test]
fn dangling_workspace_root_after_check_is_a_usage_error() {
    let cwd = fake_workspace();
    assert_usage_error_writes_nothing(cwd.path(), &["--check", "--workspace-root"]);
}

#[test]
fn flag_in_place_of_the_value_is_a_usage_error() {
    let cwd = fake_workspace();
    assert_usage_error_writes_nothing(cwd.path(), &["--workspace-root", "--check"]);
}

#[test]
fn empty_workspace_root_is_a_usage_error() {
    let cwd = fake_workspace();
    assert_usage_error_writes_nothing(cwd.path(), &["--workspace-root", ""]);
}

#[test]
fn valid_workspace_root_writes_there_and_not_in_the_cwd() {
    let cwd = fake_workspace();
    let root = fake_workspace();
    let output = run(
        cwd.path(),
        &["--workspace-root", root.path().to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.path().join("docs/lints/README.md").exists());
    assert!(root.path().join("docs/lint_catalog.md").exists());
    assert!(!cwd.path().join("docs").exists());
}
