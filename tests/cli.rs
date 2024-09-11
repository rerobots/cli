// Copyright (C) 2021 rerobots, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap());

    // Alternative style: -h
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("-h").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!("prints_help", String::from_utf8(output.stdout).unwrap());

    // Alternative style: --help
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("--help").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!("prints_help", String::from_utf8(output.stdout).unwrap());
}

#[test]
fn prints_help_search() {
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("help").arg("search").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!(
        "prints_help_search",
        String::from_utf8(output.stdout).unwrap()
    );

    // Alternative style: -h
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("search").arg("-h").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!(
        "prints_help_search",
        String::from_utf8(output.stdout).unwrap()
    );

    // Alternative style: --help
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("search").arg("--help").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!(
        "prints_help_search",
        String::from_utf8(output.stdout).unwrap()
    );
}

#[test]
fn prints_help_launch() {
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("help").arg("launch").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!(
        "prints_help_launch",
        String::from_utf8(output.stdout).unwrap()
    );

    // Alternative style: -h
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("launch").arg("-h").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!(
        "prints_help_launch",
        String::from_utf8(output.stdout).unwrap()
    );

    // Alternative style: --help
    let mut cmd = Command::cargo_bin("rerobots").unwrap();
    let assert = cmd.arg("launch").arg("--help").assert();
    let output = assert.get_output().clone();
    assert.success();
    insta::assert_snapshot!(
        "prints_help_launch",
        String::from_utf8(output.stdout).unwrap()
    );
}
