use assert_cmd::Command;
use insta::assert_snapshot;
use std::env;
use std::path::PathBuf;

fn repo_fixture() -> PathBuf {
    PathBuf::from("tests/fixtures/repo")
}

#[test]
fn snapshot_normal() {
    let fixture_dir = repo_fixture();

    let mut cmd = Command::cargo_bin("tudu").unwrap();
    cmd.arg(&fixture_dir);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("run_default", stdout);
}

#[test]
fn snapshot_verbose() {
    let fixture_dir = repo_fixture();

    let mut cmd = Command::cargo_bin("tudu").unwrap();
    cmd.arg(&fixture_dir).arg("--verbose");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("run_verbose", stdout);
}
