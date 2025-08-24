use assert_cmd::Command;
use insta::assert_snapshot;
use std::path::PathBuf;

fn repo_fixture() -> PathBuf {
    // Point to a sample repo you include in `tests/fixtures`
    PathBuf::from("tests/fixtures/repo")
}

#[test]
fn snapshot_normal() {
    let mut cmd = Command::cargo_bin("tudu").unwrap(); // `tudu` is your package name
    cmd.arg(repo_fixture());

    let output = cmd.unwrap().stdout;
    let stdout = String::from_utf8_lossy(&output);

    assert_snapshot!("run_default", stdout);
}

#[test]
fn snapshot_verbose() {
    let mut cmd = Command::cargo_bin("tudu").unwrap();
    cmd.arg(repo_fixture()).arg("--verbose");

    let output = cmd.unwrap().stdout;
    let stdout = String::from_utf8_lossy(&output);

    assert_snapshot!("run_verbose", stdout);
}
