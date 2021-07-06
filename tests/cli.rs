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
