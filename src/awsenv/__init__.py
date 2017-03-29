#!/usr/bin/env python
# -*- coding: utf-8 -*-


#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import configparser
import os
import sys

CREDENTIALS_PATH = "~/.aws/credentials"


class AWSCredentials(object):

    @classmethod
    def from_path(cls, path):
        """Load AWS credentials from a path into an AWSCredentials object."""
        config = configparser.ConfigParser()
        config.read(path)

        profiles = {}

        for section in config.sections():
            name = section
            key_id = config[section].get('aws_access_key_id')
            secret_key = config[section].get('aws_secret_access_key')

            if key_id and len(key_id) > 0 and secret_key and len(secret_key) > 0:
                profiles[name] = AWSProfile(name, key_id, secret_key)

        return AWSCredentials(**profiles)

    def __init__(self, **kwargs):
        self.profiles = kwargs

    def add(self, profile):
        if profile.name and profile.aws_access_key_id and profile.aws_secret_access_key:
            self.profiles[profile.name] = profile
            return True
        else:
            return False

    def get(self, profile):
        return self.profiles.get(profile) if profile in self.profiles.keys() else None

    def ls(self):
        return list(self.profiles.keys())


class AWSProfile(object):

    def __init__(self, name, key_id, secret_key):
        self.name = name
        self.key_id = key_id
        self.secret_key = secret_key

    def format(self, export=True):
        """Formats the AWS credentials for the shell."""
        return "\n".join([
            "{}AWS_ACCESS_KEY_ID={}".format("export " if export else "", self.aws_access_key_id),
            "{}AWS_SECRET_ACCESS_KEY={}".format("export " if export else "", self.aws_secret_access_key)
        ])

    @property
    def aws_access_key_id(self):
        return self.key_id

    @property
    def aws_secret_access_key(self):
        return self.secret_key



def main():
    parser = argparse.ArgumentParser(prog="aws-env",
        description="Extract AWS credentials for a given profile as environment variables.")
    parser.add_argument('-n', '--no-export', action="store_true",
        help="Do not use export on the variables.")
    parser.add_argument("profile", help="The profile in ~/.aws/credentials to extract credentials for.")
    args = parser.parse_args()

    config_file_path = os.path.expanduser(CREDENTIALS_PATH)

    if not os.path.isfile(config_file_path):
        fail("Unable to load credentials file from {}".format(config_file_path))

    credentials = AWSCredentials.from_path(config_file_path)

    if args.profile not in credentials.ls():
        fail("Profile {} does not exist in {}".format(args.profile, config_file_path))

    profile = credentials.get(args.profile)

    sys.stdout.write(profile.format(export=not args.no_export) + "\n")
    sys.stdout.flush()


def fail(message):
    sys.stderr.write(message + "\n")
    sys.stderr.flush()
    sys.exit(1)

if __name__ == "__main__":
    main()
