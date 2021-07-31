Installation
============

Releases
--------

Go to https://github.com/rerobots/cli/releases to find the most recent release
files built for popular targets like macOS or Linux on x86_64. If your preferred
host is not listed there, please `contact us <https://rerobots.net/contact>`_.


Building from Source Code
-------------------------

To build for own computer::

  cargo build --release --locked

Beware that the resulting program might be dynamically linked to libraries and,
therefore, not easily copied to a different host. For cross-compiling and
creating static programs (therefore avoiding linker dependencies at runtime),
releases are made with cross_.
For example, to build for Linux on Raspberry Pi, ::

  cross build --target armv7-unknown-linux-musleabihf --release --locked


.. _cross: https://github.com/rust-embedded/cross
