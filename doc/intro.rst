Introduction
============

Summary
-------

The command-line interface (CLI) is self-documenting. To begin, try::

  rerobots help

which will result in a message similar to the following

.. highlight:: none

::

  USAGE:
      rerobots [FLAGS] [OPTIONS] [SUBCOMMAND]

  FLAGS:
      -h, --help       Prints help information
      -v, --verbose    Increases verboseness level of logs; ignored if RUST_LOG is
		       defined
      -V, --version    Prints version number and exits

  OPTIONS:
      -t <FILE>                plaintext file containing API token; with this
			       flag, the REROBOTS_API_TOKEN environment variable
			       is ignored
	  --format <FORMAT>    output formatting; options: YAML , JSON

  SUBCOMMANDS:
      help         Prints this message or the help of the given subcommand(s)
      info         Print summary about instance
      isready      Indicate whether instance is ready with exit code
      launch       Launch instance from specified workspace deployment or type
      list         List all instances by this user
      search       Search for matching deployments. empty query implies show
		   all existing workspace deployments
      ssh          Connect to instance host via ssh
      terminate    Terminate instance
      version      Prints version number and exits
      wdinfo       Print summary about workspace deployment

Call ``help`` to learn more about commands, e.g., ``rerobots help info`` to
learn usage of ``rerobots info``.

To use an `API token <https://rerobots.net/tokens>`_, assign it to the
environment variable ``REROBOTS_API_TOKEN``, or give it through a file named in
the command-line switch ``-t``.


.. _ssec:cli-example:

Example
-------

The following video demonstrates how to search for types of workspaces, request
an instance, and finally terminate it. The same example is also presented below
in text. (This video can also be watched at https://asciinema.org/a/l0l2yh83JtAM8RjDiOHsk3Q9F)

.. raw:: html

  <script id="asciicast-l0l2yh83JtAM8RjDiOHsk3Q9F" src="https://asciinema.org/a/l0l2yh83JtAM8RjDiOHsk3Q9F.js" async></script>

.. highlight:: none

Before beginning, `get an API token
<https://help.rerobots.net/webui.html#making-and-revoking-api-tokens>`_ (`from
the Web UI <https://rerobots.net/tokens>`_). Now assign it to an environment variable.
For example, if the API token is saved to a local file named ``tok``, then ::

  export REROBOTS_API_TOKEN=$(cat tok)

Search for workspace deployments::

  $ rerobots search misty
  2c0873b5-1da1-46e6-9658-c40379774edf    fixed_misty2

Get more information about one of them::

  $ rerobots wdinfo 2c0873b5-1da1-46e6-9658-c40379774edf
  {
    "cap": {
      "rules": []
    },
    "id": "2c0873b5-1da1-46e6-9658-c40379774edf",
    "type": "fixed_misty2",
    "type_version": 1,
    "supported_addons": [
      "cam",
      "mistyproxy"
    ],
    "desc": "",
    "region": "us:cali",
    "icounter": 641,
    "created": "2019-11-18 22:23:57.433893",
    "queuelen": 0
  }

Notice that ``queuelen = 0``, i.e., this workspace deployment is available, and
requests to instantiate from it now are likely to succeed. To do so, ::

  $ rerobots launch 2c0873b5-1da1-46e6-9658-c40379774edf
  f7856ad4-a9d7-43f5-8420-7073d10bceec

Get information about the new instance::

  $ rerobots info f7856ad4-a9d7-43f5-8420-7073d10bceec
  {
    "id": "f7856ad4-a9d7-43f5-8420-7073d10bceec",
    "deployment": "2c0873b5-1da1-46e6-9658-c40379774edf",
    "type": "fixed_misty2",
    "region": "us:cali",
    "starttime": "2020-05-23 02:05:20.311535",
    "rootuser": "scott",
    "fwd": {
      "ipv4": "147.75.70.51",
      "port": 2210
    },
    "hostkeys": [
      "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBPd5tTJLAksiu3uTbGwkBKXFb00XyTPeef6tn/0AMFiRpomU5bArpJnT3SZKhN3kkdT3HvTQiN5/dexOCFWNGUE= root@newc59"
    ],
    "status": "READY"
  }

When ``READY``, get the SSH secret key created for the instance::

  $ rerobots get-ssh-key f7856ad4-a9d7-43f5-8420-7073d10bceec

The section ``fwd`` contains an IP address and port at which ssh connections can
be established to the instance host. The rerobots CLI provides a convenient
command to do this::

  $ rerobots ssh f7856ad4-a9d7-43f5-8420-7073d10bceec

Finally, ``exit`` the ssh shell, and terminate the instance::

  $ rerobots terminate f7856ad4-a9d7-43f5-8420-7073d10bceec
