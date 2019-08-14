#!/usr/bin/env python
# -*- coding: utf-8 -*-


#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import os
import subprocess
import sys

PYTHON_VERSION = sys.version_info[0]

if PYTHON_VERSION < 3:
    from ConfigParser import ConfigParser
else:
    from configparser import ConfigParser

CREDENTIALS_ROOT = os.path.expanduser("~/.aws")
CREDENTIALS_PATH = os.path.join(CREDENTIALS_ROOT, 'credentials')
CREDENTIALS_D_PATH  = os.path.join(CREDENTIALS_ROOT, 'credentials.d')




def config_to_dict(config):
    """Takes a configparser instance and converts it into a sensible dictionary."""
    result = {}

    sections = config.sections()
    for section in sections:
        result[str(section)] = {
            i[0]: i[1] for i in config.items(section)
        }

    return result


class GnuPG(object):
    """GnuPG utility class."""

    __GNUPG_PATH = None

    @classmethod
    def path(cls):
        """Returns the absolute path to (in order) GnuPG 2 or GnuPG 1."""
        if not cls.__GNUPG_PATH:
            for trial in ['gpg2', 'gpg']:
                p = subprocess.Popen(['which', trial], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                stdout, _ = p.communicate()

                if p.returncode == 0:
                    cls.__GNUPG_PATH = stdout.strip().decode('utf-8')
                    return cls.__GNUPG_PATH

        return cls.__GNUPG_PATH


class AWSCredentials(object):

    @classmethod
    def find_files(cls):
        """Find credentials files and return a list."""
        if not os.path.isdir(CREDENTIALS_D_PATH):
            return [CREDENTIALS_PATH]

        credentials_d_files = list(
            filter(lambda p: os.path.isfile(p), sorted(map(lambda p: os.path.join(CREDENTIALS_D_PATH, p),
                os.listdir(CREDENTIALS_D_PATH))))
        )

        return [CREDENTIALS_PATH] + credentials_d_files

    @classmethod
    def __load_encrypted(cls, p):
        """Loads an encrypted file into a dictionary of profile names to access and secret keys."""
        decrypter = subprocess.Popen([GnuPG.path(), '-d', p], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        stdout, _ = decrypter.communicate()

        if not decrypter.returncode == 0:
            fail("ERROR: Unable to decrypt {}".format(p))

        contents = stdout.strip().decode('utf-8')

        config = ConfigParser()

        try:
            config.read_string(contents)
        except:
            fail("ERROR: Unable to parse {} as an INI-structured file.".format(p))

        return config_to_dict(config)

    @classmethod
    def __load_plaintext(cls, p):
        """Loads a plaintext file into a dictionary of profile names to access and secret keys."""
        config = ConfigParser()

        try:
            config.read(p)
        except:
            fail("ERROR: Unable to load {} as an INI-structured file.".format(p))

        return config_to_dict(config)

    @classmethod
    def load(cls):
        """Load all credentials into a dictionary."""
        result = {}

        for p in cls.find_files():
            if p.lower().endswith('.asc') or p.lower().endswith('.gpg') or p.lower().endswith('.pgp'):
                data = cls.__load_encrypted(p)
            else:
                data = cls.__load_plaintext(p)

            data.update(result)
            result = data

        profile_map = {}

        for name in result.keys():
            profile = result[name]
            key_id, secret_key, session_token = profile.get('aws_access_key_id'), profile.get('aws_secret_access_key'), profile.get('aws_session_token')

            if len(key_id or '') > 0 and len(secret_key or '') > 0 and len(session_token or '') > 0:
                profile_map[name] = AWSProfile(name=name, key_id=key_id, secret_key=secret_key, session_token=session_token)
            elif len(key_id or '') > 0 and len(secret_key or ''):
                profile_map[name] = AWSProfile(name=name, key_id=key_id, secret_key=secret_key)

        return AWSCredentials(**profile_map)

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

    def __init__(self, name, key_id, secret_key, session_token=None):
        self.name = name
        self.key_id = key_id
        self.secret_key = secret_key
        self.session_token = session_token

    def format(self, export=True):
        """Formats the AWS credentials for the shell."""
        if self.aws_session_token:
            return "\n".join([
                "{}AWS_ACCESS_KEY_ID={}".format("export " if export else "", self.aws_access_key_id),
                "{}AWS_SECRET_ACCESS_KEY={}".format("export " if export else "", self.aws_secret_access_key),
                "{}AWS_SESSION_TOKEN={}".format("export " if export else "", self.aws_session_token),
                "{}AWS_PROFILE={}".format("export " if export else "", self.name)
            ])
        else:
            return "\n".join([
                "{}AWS_ACCESS_KEY_ID={}".format("export " if export else "", self.aws_access_key_id),
                "{}AWS_SECRET_ACCESS_KEY={}".format("export " if export else "", self.aws_secret_access_key),
                "{}AWS_PROFILE={}".format("export " if export else "", self.name)
            ])
    @property
    def aws_access_key_id(self):
        return self.key_id

    @property
    def aws_secret_access_key(self):
        return self.secret_key

    @property
    def aws_session_token(self):
        return self.session_token



def main():
    parser = argparse.ArgumentParser(prog="aws-env",
        description="Extract AWS credentials for a given profile as environment variables.")
    parser.add_argument('-n', '--no-export', action="store_true",
        help="Do not use export on the variables.")
    parser.add_argument('-l', '--ls', dest="list", action="store_true", help="List available profiles.")
    parser.add_argument("profile", nargs="?", default="default",
        help="The profile in ~/.aws/credentials or ~/.aws/credentials.d/ to extract credentials for. Defaults to 'default'.")
    args = parser.parse_args()

    credentials = AWSCredentials.load()

    user_cred_path = CREDENTIALS_PATH.replace(os.environ.get('HOME'), '~')
    user_cred_d_path = CREDENTIALS_D_PATH.replace(os.environ.get('HOME'), '~') + os.path.sep

    if args.list:
        if len(credentials.ls()) < 1:
            fail("ERROR: No profiles found in {}, {}".format(user_cred_path, user_cred_d_path))

        # just list the profiles and get out
        print('\n'.join(sorted(credentials.ls())))
        return 0

    if args.profile not in credentials.ls():
        fail("Profile '{}' not found in {}, {}/".format(args.profile, user_cred_path, user_cred_d_path))

    profile = credentials.get(args.profile)

    sys.stdout.write(profile.format(export=not args.no_export) + "\n")
    sys.stdout.flush()


def fail(message):
    sys.stderr.write(message + "\n")
    sys.stderr.flush()
    sys.exit(1)


if __name__ == "__main__":
    main()
