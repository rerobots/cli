rerobots CLI
============

This is the command-line interface (CLI) for rerobots_.
The corresponding source code repository is hosted at https://github.com/rerobots/cli

Summary
-------

The command-line interface (CLI) is self-documenting. To begin, try::

  rerobots help

which will result in a message similar to the following

.. highlight:: none

::

  usage: rerobots [-h] [-V] [-t FILE]
		  {info,isready,addon-cam,addon-mistyproxy,addon-drive,list,search,wdinfo,launch,terminate,version,help}
		  ...

  rerobots API command-line client

  positional arguments:
    {info,isready,addon-cam,addon-mistyproxy,addon-drive,list,search,wdinfo,launch,terminate,version,help}
      info                print summary about instance.
      isready             indicate whether instance is ready with exit code.
      addon-cam           get image via add-on `cam`
      addon-mistyproxy    get proxy URL via add-on `mistyproxy`
      addon-drive         send motion commands via add-on `drive`
      list                list all instances owned by this user.
      search              search for matching deployments. empty query implies
			  show all existing workspace deployments.
      wdinfo              print summary about workspace deployment.
      launch              launch instance from specified workspace deployment or
			  type. if none is specified, then randomly select from
			  those available.
      terminate           terminate instance.
      version             print version number and exit.
      help                print this help message and exit

  optional arguments:
    -h, --help            print this help message and exit
    -V, --version         print version number and exit.
    -t FILE, --jwt FILE   plaintext file containing API token; with this flag,
			  the REROBOTS_API_TOKEN environment variable is
			  ignored.

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
in text.

Before beginning, `get an API token
<https://help.rerobots.net/webui.html#making-and-revoking-api-tokens>`_ (`from
the Web UI <https://rerobots.net/tokens>`_). In this example, we assume that it
is saved to a file named ``jwt.txt``.

.. original video is hosted at https://asciinema.org/a/l0l2yh83JtAM8RjDiOHsk3Q9F

.. raw:: html

  <script id="asciicast-l0l2yh83JtAM8RjDiOHsk3Q9F" src="https://asciinema.org/a/l0l2yh83JtAM8RjDiOHsk3Q9F.js" async></script>

.. highlight:: none

Search for workspace deployments::

  $ rerobots search misty
  2c0873b5-1da1-46e6-9658-c40379774edf    fixed_misty2

Get more information about one of them::

  $ rerobots wdinfo 2c0873b5-1da1-46e6-9658-c40379774edf
  {
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

which will result in a secret key being written locally to the file ``key.pem``.
This key should be used for ssh connections, e.g., with commands of the form
``ssh -i key.pem``. Get information about the new instance::

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

Finally, terminate the instance::

  $ rerobots terminate f7856ad4-a9d7-43f5-8420-7073d10bceec


.. _rerobots: https://rerobots.net/
