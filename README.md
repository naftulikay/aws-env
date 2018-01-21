# aws-env [![Build Status][svg-travis]][travis]

A simple shell utility for exporting a given AWS credentials profile to environment variables. Useful for crossing
machine boundaries with SSH and Vagrant.

## Usage

Shamelessly ripped from `aws-env -h`:

```
usage: aws-env [-h] [-n] [-l] [profile]

Extract AWS credentials for a given profile as environment variables.

positional arguments:
  profile          The profile in ~/.aws/credentials or ~/.aws/credentials.d/
                   to extract credentials for. Defaults to 'default'.

optional arguments:
  -h, --help       show this help message and exit
  -n, --no-export  Do not use export on the variables.
  -l, --ls         List available profiles.
```

`aws-env` looks first at `~/.aws/credentials` and then at all files in `~/.aws/credentials.d/` if found and merges all
of them together in a dictionary of profiles. `~/.aws/credentials` is loaded first and everything loaded from
`~/.aws/credentials.d/` are loaded in alphabetically sorted order and merged in. (See
[Encrypted Credential Files](#Encrypted Credential Files) for instructions on using encrypted credential files.)

If you have a profile named `brangus`, you can extract environment variables like so:

```shell
$ aws-env brangus
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

As a shortcut, you can directly source the output of this command to export the variables into your shell session:

```shell
$ $(aws-env brangus)
```

This will cause your shell to execute the output of `aws-env`, exporting these environment variables.

> **WARNING:** This is _potentially very dangerous_ if you don't trust the script you're executing, so use with care and
> establish trust. All commits and releases here are [PGP signed][keybase] with my key, so if you know me and trust me,
> you should be able to use this, but as always, _read the source code_ and check it before you blindly pipe code into
> your shell session.

### Encrypted Credential Files

`aws-env` now supports encrypted credential files! Stash files ending in `.asc`, `.gpg`, or `.pgp` in
`~/.aws/credentials.d/` and `aws-env` will attempt to decrypt these files using GnuPG. `gpg2` is preferred but `gpg`
will be used as a backup option.

`aws-env` will decrypt files directly into memory. File format should be the same as `~/.aws/credentials`. Here's a
sample `tree` output detailing the directory layout:

```
/home/naftuli/.aws
├── config
├── credentials
└── credentials.d
    └── naftulikay.asc

1 directory, 3 files
```

## Installation

There already exists an `awsenv` package on PyPI, so this is not published to PyPI. I have a personal frustration with
PyPI in a number of respects, so this module is best installed via pip directly.

Please visit the [releases page][releases] for a listing of releases and tag names, and use those to install a given
version of the software. Release `1.0.0` is going to have a tag named `v1.0.0`, etc.

##### User Install

To install `aws-env` as an ordinary user:

```shell
pip install --user git+https://github.com/naftulikay/aws-env@v1.0.0
```

##### System Install

To install `aws-env` system-wide:

```shell
sudo pip install git+https://github.com/naftulikay/aws-env@v1.0.0
```

## Building

This project uses [`buildout`][buildout] to manage the project. Simply put, to get started you can use a `virtualenv`
if you'd like, then install the requirements:

```shell
$ pip install --user -r requirments.txt
```

Next, build the project:

```shell
$ buildout
```

You should now have all dependencies and some scripts in `bin/` such as `bin/test`, `bin/python`, `bin/ipython`, and
`bin/aws-env`. Buildout is rad.

## License

Read the file called `LICENSE`, but it's basically MIT. If you want or need a dual-license for some reason that I have
yet to understand, please ask and I can dual-license it as appropriate.

 [travis]: https://travis-ci.org/naftulikay/aws-env
 [svg-travis]: https://travis-ci.org/naftulikay/aws-env.svg?branch=master
 [releases]: https://github.com/naftulikay/aws-env/releases
 [keybase]: https://keybase.io/naftulikay
 [buildout]: https://github.com/buildout/buildout
