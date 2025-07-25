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

extern crate log;

#[macro_use]
extern crate clap;

extern crate serde_json;
extern crate serde_yaml;

mod cli;

fn main() {
    rerobots::init();
    match cli::main() {
        Ok(_) => std::process::exit(0),
        Err(err) => {
            if err.msg.is_some() {
                eprintln!("{err}");
            }
            std::process::exit(err.exitcode);
        }
    }
}
