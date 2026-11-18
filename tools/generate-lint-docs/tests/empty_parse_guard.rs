//! A `lib.rs` that yields no `declare_lint!` blocks (a macro-format change, a
//! partial checkout) used to regenerate the docs as empty pages (issue #1076).
//! The generator must refuse unless `--allow-empty` is passed.

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

/// The same lint written in a shape the parser does not recognise, as after
/// an upstream change to the `declare_lint!` macro syntax.
const UNRECOGNISED_FORMAT: &str = r#"
rustc_session::declare_lint!(SOROBAN_STORAGE_IN_LOOP, Deny, "storage operations inside a loop");
"#;

const EXISTING: &str = "existing generated docs\n";

fn workspace(lib_rs: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp workspace");
    let root = dir.path();
    fs::create_dir_all(root.join("soroban_cost_lints/src")).unwrap();
    fs::write(root.join("soroban_cost_lints/src/lib.rs"), lib_rs).unwrap();
    fs::create_dir_all(root.join("docs/lints")).unwrap();
    fs::write(root.join("docs/lints/README.md"), EXISTING).unwrap();
    fs::write(root.join("docs/lint_catalog.md"), EXISTING).unwrap();
    dir
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_generate-lint-docs"))
        .args(args)
        .args(["--workspace-root", root.to_str().unwrap()])
        .output()
        .expect("failed to run generate-lint-docs")
}

fn read(root: &Path, file: &str) -> String {
    fs::read_to_string(root.join(file)).unwrap()
}

fn assert_refused(lib_rs: &str, args: &[&str]) {
    let ws = workspace(lib_rs);
    let output = run(ws.path(), args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "args {args:?}, stderr: {stderr}"
    );
    assert!(
        stderr.contains("no `declare_lint!` blocks"),
        "args {args:?}, stderr: {stderr}"
    );
    assert!(
        stderr.contains(
            &ws.path()
                .join("soroban_cost_lints/src/lib.rs")
                .display()
                .to_string()
        ),
        "the diagnostic should point at the parsed file: {stderr}"
    );
    assert!(stderr.contains("--allow-empty"), "stderr: {stderr}");
    assert!(
        output.stdout.is_empty(),
        "nothing should be reported as written"
    );
    assert_eq!(read(ws.path(), "docs/lints/README.md"), EXISTING);
    assert_eq!(read(ws.path(), "docs/lint_catalog.md"), EXISTING);
}

#[test]
fn empty_lib_rs_aborts_without_overwriting_docs() {
    assert_refused("", &[]);
}

#[test]
fn unrecognised_macro_format_aborts_without_overwriting_docs() {
    assert_refused(UNRECOGNISED_FORMAT, &[]);
}

#[test]
fn check_mode_reports_an_empty_parse_too() {
    assert_refused(UNRECOGNISED_FORMAT, &["--check"]);
}

#[test]
fn allow_empty_overrides_the_guard() {
    let ws = workspace("");
    let output = run(ws.path(), &["--allow-empty"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(read(ws.path(), "docs/lints/README.md"), EXISTING);
    assert_ne!(read(ws.path(), "docs/lint_catalog.md"), EXISTING);
}

#[test]
fn non_empty_parse_still_regenerates_the_docs() {
    let ws = workspace(SINGLE_LINT);
    let output = run(ws.path(), &[]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(read(ws.path(), "docs/lints/README.md").contains("soroban_storage_in_loop"));
    assert!(read(ws.path(), "docs/lint_catalog.md").contains("soroban_storage_in_loop"));
}
