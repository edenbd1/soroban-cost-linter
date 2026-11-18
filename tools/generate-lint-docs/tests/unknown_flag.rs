//! An unrecognised argument such as `--chek` used to be ignored, so the
//! generator took the write path and rewrote the docs instead of failing the
//! gate (issue #1072). It must exit 2 with a usage summary and write nothing.

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

const STALE: &str = "stale docs that a write would replace\n";

/// A workspace whose generated docs are stale, so any run that reaches the
/// write path visibly rewrites them.
fn stale_workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp workspace");
    let root = dir.path();
    fs::create_dir_all(root.join("soroban_cost_lints/src")).unwrap();
    fs::write(root.join("soroban_cost_lints/src/lib.rs"), SINGLE_LINT).unwrap();
    fs::create_dir_all(root.join("docs/lints")).unwrap();
    fs::write(root.join("docs/lints/README.md"), STALE).unwrap();
    fs::write(root.join("docs/lint_catalog.md"), STALE).unwrap();
    dir
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_generate-lint-docs"))
        .args(args)
        .args(["--workspace-root", root.to_str().unwrap()])
        .output()
        .expect("failed to run generate-lint-docs")
}

fn assert_docs_untouched(root: &Path) {
    for file in ["docs/lints/README.md", "docs/lint_catalog.md"] {
        assert_eq!(
            fs::read_to_string(root.join(file)).unwrap(),
            STALE,
            "{file} must not be rewritten"
        );
    }
}

fn assert_rejected(args: &[&str], offending: &str) {
    let ws = stale_workspace();
    let output = run(ws.path(), args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "args {args:?}, stderr: {stderr}"
    );
    assert!(
        stderr.contains(&format!("unrecognised argument `{offending}`")),
        "stderr should name the bad argument: {stderr}"
    );
    assert!(
        stderr.contains("Usage: generate-lint-docs"),
        "stderr should carry the usage summary: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "nothing should be reported as written"
    );
    assert_docs_untouched(ws.path());
}

#[test]
fn misspelt_check_flag_exits_2_without_writing() {
    assert_rejected(&["--chek"], "--chek");
}

#[test]
fn unknown_flag_next_to_check_is_still_rejected() {
    assert_rejected(&["--check", "--verbose"], "--verbose");
}

#[test]
fn stray_positional_argument_is_rejected() {
    assert_rejected(&["check"], "check");
}

#[test]
fn check_mode_reports_drift_without_writing() {
    let ws = stale_workspace();
    let output = run(ws.path(), &["--check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("out of date"));
    assert_docs_untouched(ws.path());
}

#[test]
fn no_flag_still_regenerates_the_docs() {
    let ws = stale_workspace();
    let output = run(ws.path(), &[]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let readme = fs::read_to_string(ws.path().join("docs/lints/README.md")).unwrap();
    assert!(
        readme.contains("soroban_storage_in_loop"),
        "README: {readme}"
    );
}
