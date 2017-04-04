# aws-env [![Build Status][svg-travis]][travis]

A simple shell utility for exporting a given AWS credentials profile to environment variables. Useful for crossing
machine boundaries with SSH and Vagrant.

## Usage

Shamelessly ripped from `aws-env -h`:

```
usage: aws-env [-h] [-n] [-l] [profile]

Extract AWS credentials for a given profile as environment variables.

positional arguments:
  profile          The profile in ~/.aws/credentials to extract credentials
                   for. Defaults to 'default'.

optional arguments:
  -h, --help       show this help message and exit
  -n, --no-export  Do not use export on the variables.
  -l, --ls         List available profiles.
```

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

## License

Read the file called `LICENSE`, but it's basically MIT. If you want or need a dual-license for some reason that I have
yet to understand, please ask and I can dual-license it as appropriate.

 [travis]: https://travis-ci.org/naftulikay/aws-env
 [svg-travis]: https://travis-ci.org/naftulikay/aws-env.svg?branch=master
 [releases]: https://github.com/naftulikay/aws-env/releases
 [keybase]: https://keybase.io/naftulikay
