# aws-env [![Build Status][build.svg]][build]

A utility for exporting a given AWS credentials profile to environment variables. Useful for crossing
machine boundaries with SSH and Vagrant.

## Background

The `aws` CLI and other tools such as Terraform can use an INI-format file located at `~/.aws/credentials` to
store different "profiles" and credentials/configuration for each. While this works fairly well, storing all
credentials in a single, unencrypted file is far from ideal.

This utility allows users to store profiles in multiple files, optionally using GnuPG for file encryption so that
secrets are never stored in plaintext when stored. `aws-env` will use an ordered loading system to load from:

 1. the traditional `~/.aws/credentials` file in plaintext.
 2. a GnuPG (`gpp`) encrypted file at `~/.aws/credentials.asc` or `~/.aws/credentials.gpg`.
 3. both encrypted and plaintext profiles within the `~/.aws/credentials.d` directory, either with a suffix of
    `*.gpg`/`*.asc`/`*.ini`, or without a file suffix.

When using multiple files, `aws-env` creates prefixed names for profiles in case of multiple files containing the
same profile id. See the output of `aws-env list` for more information.

Other features, such as the ability to use SSO profiles, are not supported yet, but this work is being tracked
in #19.

## Usage

Shamelessly ripped from `aws-env -h`:

```
aws-env 2.0.0

USAGE:
    aws-env [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --log-level <log-level>    Set the logging level for the utility [default: error]  [possible values: trace,
                                   debug, info, warn, error]

SUBCOMMANDS:
    export    Export the specified profile
    help      Prints this message or the help of the given subcommand(s)
    list      List available profiles

```

### Listing Available Profiles

To list available profiles, use `aws-env list` command:

```text
aws-env-list 2.0.0
List available profiles

USAGE:
    aws-env list [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
        --no-header    Exclude the header when printing to a TTY
    -V, --version      Prints version information

OPTIONS:
    -F, --format <format>    The output format [default: table]  [possible values: table, plain, csv, json]
```

Listing profiles will never expose sensitive data, only the presence of profiles within the configuration files.
By default, the `table` format is used to display the profiles:

```text
profile   prefix/profile priority file
――――――――― ―――――――――――――― ―――――――― ――――――――――――――――――――――――――――
hello     a/hello        00       ~/.aws/credentials.d/a.ini
goodbye   a/goodbye      01       ~/.aws/credentials.d/a.ini
encrypted enc/encrypted  02       ~/.aws/credentials.d/enc.asc
default   /default       03       ~/.aws/credentials
```

The `profile` field is the name of the profile within a file, e.g. `[default]` will yield a name of `default`.
The `prefix/profile` field is a generated, qualified path to a profile, which is useful when multiple profiles
with the same name exist across multiple files. Both the profile name and the `prefix/profile` format are used
during lookup in `aws-env export`. The `priority` field is a generated field showing the load order of profiles,
the larger the value of `priority`, the higher precedence it has when collisions occur.

Finally, the `file` field simply points to the file from which the given profile was found.

### Exporting a Profile

For information on how profiles are loaded, see the previous section.

`aws-env export` will dump the specified profile in shell commands to standard output.

```text
aws-env-export 2.0.0
Export the specified profile

USAGE:
    aws-env export <profile_name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <profile_name>    The profile name to export. This can be either the bare profile name or a URI. See the 'list'
                      command for URI format
```

For example, to export the `default` profile mentioned above, run `aws-env export default`, and you will see output
like:

```shell
export AWS_ACCESS_KEY_ID=YOUR_ACCESS_KEY_ID
export AWS_SECRET_ACCESS_KEY=YOUR_SECRET_KEY
```

Additionally, qualified names can be used to resolve collisions. `aws-env export default` and `aws-env export /default`
refer to the same profile as described above.

#### Directly Exporting to Shell

Simply dumping the profile credentials to standard out does not mean that these are exported to your shell session.
In most shells, to directly export the credentials to the shell session, you can have your shell execute the output
from `aws-env`:

```shell
$(aws-env export default)
```

When you run this in an interactive shell session, you won't see any output from the command, but you should be able
to now see that the environment variables have been set correctly:

```shell
$ env | grep AWS_
AWS_ACCESS_KEY_ID=YOUR_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY=YOUR_SECRET_KEY
```

## Installation

To install, clone the Git repository locally, and run `cargo install --path .` to install `aws-env` to your `PATH`
under `~/.cargo/bin`. You'll need a functional Rust compilation environment to install from source like this.

## License

Licensed at your discretion under either:

 - [Apache Software License, Version 2.0](./LICENSE-APACHE)
 - [MIT License](./LICENSE-MIT)

 [build]: https://github.com/naftulikay/aws-env/actions/workflows/rust.yml
 [build.svg]: https://github.com/naftulikay/aws-env/actions/workflows/rust.yml/badge.svg
