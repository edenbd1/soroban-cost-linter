//! `--clear-cache` exits before any other action runs, so combining it with
//! `--list-lints` or `--explain` must be rejected up front rather than
//! clearing the cache and silently dropping the other flag (issue #1066).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// A workspace whose `target/cost-lint-cache` holds one cached result, so a
/// test can tell whether the cache was cleared.
fn workspace_with_cached_result() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("create temp workspace");
    let cache_dir = dir.path().join("target").join("cost-lint-cache");
    fs::create_dir_all(&cache_dir).expect("create cache dir");
    let entry = cache_dir.join("cached.json");
    fs::write(&entry, "{}").expect("write cache entry");
    (dir, entry)
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-cost-lint"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("Failed to execute cargo-cost-lint")
}

fn assert_usage_error(output: &Output, flag: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    assert!(
        output.stdout.is_empty(),
        "no action may run: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(stderr.contains("cannot be used with"), "stderr: {stderr}");
    assert!(stderr.contains("--clear-cache"), "stderr: {stderr}");
    assert!(stderr.contains(flag), "stderr: {stderr}");
}

#[test]
fn clear_cache_with_list_lints_is_a_usage_error() {
    let (dir, entry) = workspace_with_cached_result();
    let output = run(dir.path(), &["--clear-cache", "--list-lints"]);
    assert_usage_error(&output, "--list-lints");
    assert!(entry.exists(), "the cache must not be cleared");
}

#[test]
fn clear_cache_with_explain_is_a_usage_error() {
    let (dir, entry) = workspace_with_cached_result();
    let output = run(
        dir.path(),
        &["--clear-cache", "--explain", "redundant_env_clone"],
    );
    assert_usage_error(&output, "--explain");
    assert!(entry.exists(), "the cache must not be cleared");
}

#[test]
fn flag_order_does_not_matter() {
    let (dir, entry) = workspace_with_cached_result();
    let output = run(dir.path(), &["--list-lints", "--clear-cache"]);
    assert_usage_error(&output, "--list-lints");
    assert!(entry.exists(), "the cache must not be cleared");
}

#[test]
fn cargo_subcommand_form_is_rejected_too() {
    let (dir, entry) = workspace_with_cached_result();
    let output = run(dir.path(), &["cost-lint", "--clear-cache", "--list-lints"]);
    assert_usage_error(&output, "--list-lints");
    assert!(entry.exists(), "the cache must not be cleared");
}

#[test]
fn standalone_clear_cache_still_clears() {
    let (dir, entry) = workspace_with_cached_result();
    let output = run(dir.path(), &["--clear-cache"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("Cleared 1 cached lint result(s)"),
        "stdout: {stdout}"
    );
    assert!(!entry.exists(), "the cached result must be removed");
}

#[test]
fn standalone_list_lints_still_lists() {
    let (dir, entry) = workspace_with_cached_result();
    let output = run(dir.path(), &["--list-lints"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "stdout: {stdout}");
    assert!(stdout.starts_with("Lint inventory"), "stdout: {stdout}");
    assert!(stdout.contains("redundant_env_clone"), "stdout: {stdout}");
    assert!(entry.exists(), "--list-lints must not touch the cache");
}
