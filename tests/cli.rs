// SCL <scott@rerobots.net>
// Copyright (C) 2021 rerobots, Inc.

use assert_cmd::Command;


#[test]
fn prints_version() {
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("-V").assert();
    assert
        .stdout(format!("{}\n", env!("CARGO_PKG_VERSION")))
        .success();
}


#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("help").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_display_snapshot!(String::from_utf8(output.stdout).unwrap());

    // Alternative style: -h
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("-h").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_display_snapshot!("prints_help", String::from_utf8(output.stdout).unwrap());

    // Alternative style: --help
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("--help").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_display_snapshot!("prints_help", String::from_utf8(output.stdout).unwrap());
}


#[test]
fn prints_help_search() {
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("help").arg("search").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_display_snapshot!("prints_help_search", String::from_utf8(output.stdout).unwrap());

    // Alternative style: -h
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("search").arg("-h").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_display_snapshot!("prints_help_search", String::from_utf8(output.stdout).unwrap());

    // Alternative style: --help
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("search").arg("--help").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_display_snapshot!("prints_help_search", String::from_utf8(output.stdout).unwrap());
}
