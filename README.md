rerobots CLI
============

Summary
-------

command-line interface (CLI) for [rerobots](https://rerobots.net/)

Documentation of the current release is at https://docs.rerobots.net/tools/cli

If you want to contribute to development, then read more below, and clone the
repository at https://github.com/rerobots/cli


Getting Started
---------------

Most interesting interactions with rerobots require an API token, which can be
provided through the environment variable `REROBOTS_API_TOKEN` or via the
command-line switch `-t`. Get API tokens at https://rerobots.net/tokens


Building and Testing
--------------------

This tool is implemented in [Rust](https://www.rust-lang.org/), and releases are
posted to the crate registry at <https://crates.io/crates/rerobots-cli>.
To build, clone the repository at https://github.com/rerobots/cli.git and

    cargo build

To perform tests,

    cargo test

To check code style,

    cargo fmt -- --check
    cargo clippy --tests -- -D clippy::all

Current [CI report](https://github.com/rerobots/cli/actions/workflows/main.yml):
![build status from GitHub Actions](https://github.com/rerobots/cli/actions/workflows/main.yml/badge.svg)


Participating
-------------

All participation must follow our code of conduct, elaborated in the file
CODE_OF_CONDUCT.md in the same directory as this README.

### Reporting errors, requesting features

Please first check for prior reports that are similar or related in the issue
tracker at https://github.com/rerobots/cli/issues

Reports of security flaws are given the highest priority and should be sent to
<security@rerobots.net>, optionally encrypted with the public key available at
https://rerobots.net/contact Please do so before opening a public issue to allow
us an opportunity to find a fix.

### Contributing changes or new code

Contributions are welcome! There is no formal declaration of code style. Just
try to follow the style and structure currently in the repository.

Contributors, who are not rerobots employees, must agree to the [Developer
Certificate of Origin](https://developercertificate.org/). Your agreement is
indicated explicitly in commits by adding a Signed-off-by line with your real
name. (This can be done automatically using `git commit --signoff`.)


License
-------

This is free software, released under the Apache License, Version 2.0.
You may obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
