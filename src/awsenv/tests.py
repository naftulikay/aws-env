#!/usr/bin/env python
# -*- coding: utf-8 -*-

import sys
import tempfile
import mock
import unittest

from unittest import skip

from awsenv import AWSCredentials, AWSProfile, config_to_dict, GnuPG

PYTHON_VERSION = sys.version_info[0]


class GnuPGTestCase(unittest.TestCase):

    def test_load(self):
        """Tests that finding GnuPG works."""
        detected_path = GnuPG.path()

        self.assertIsNotNone(detected_path)
        self.assertIn(detected_path, ['/usr/bin/gpg2', '/usr/bin/gpg'])


class ConfigToDictTestCase(unittest.TestCase):

    def test_config_to_dict_py2(self):
        """Tests that converting a Python 2 ConfigParser object to a dictionary works as expected."""
        if PYTHON_VERSION > 2:
            return

        from ConfigParser import ConfigParser
        fixture = ConfigParser()
        fixture.add_section('something')
        fixture.set('something', 'value', 'stuff')

        self.assertEqual({ 'something': { 'value': 'stuff' } }, config_to_dict(fixture))

    def test_config_to_dict_py3(self):
        """Tests that converting a Python 3 ConfigParser object to a dictionary works as expected."""
        if PYTHON_VERSION < 3:
            return

        from configparser import ConfigParser
        fixture = ConfigParser()
        fixture['something'] = { 'value': 'stuff' }

        self.assertEqual({ 'something': { 'value': 'stuff' } }, config_to_dict(fixture))


class AWSCredentialsTestCase(unittest.TestCase):

    def setUp(self):
        """Sets up fixtures."""
        profiles = {
            'one': {
                'aws_access_key_id': 'key one',
                'aws_secret_access_key': 'key two',
            },
            'two': {
                'aws_access_key_id': 'another',
                'aws_secret_access_key': 'thing',
            },
            'blank_id': {
                'aws_secret_access_key': 'value'
            },
            'blank_secret': {
                'aws_access_key_id': 'eyedee'
            }
        }

        self.credentials_file = tempfile.NamedTemporaryFile(mode='w', delete=True)

        for profile in profiles.keys():
            self.credentials_file.write("[{}]\n".format(profile))

            if profiles.get(profile).get('aws_access_key_id'):
                self.credentials_file.write("aws_access_key_id={}\n".format(profiles.get(profile).get('aws_access_key_id')))

            if profiles.get(profile).get('aws_secret_access_key'):
                self.credentials_file.write('aws_secret_access_key={}\n'.format(profiles.get(profile).get('aws_secret_access_key')))

            self.credentials_file.write("\n")

        self.credentials_file.flush()

    def test_add(self):
        result = AWSCredentials()

        self.assertEqual(0, len(result.profiles.keys()))

        valid = AWSProfile('profile', 'key id', 'key value')
        rc = result.add(valid)

        self.assertTrue(rc)
        self.assertEqual(valid, result.profiles.get('profile'))

        # test null secret key
        rc = result.add(AWSProfile('profile', 'key id', None))

        self.assertFalse(rc)
        self.assertEqual(1, len(result.profiles.keys()))

        # test empty secret key
        rc = result.add(AWSProfile('profile', 'key id', ''))

        self.assertFalse(rc)
        self.assertEqual(1, len(result.profiles.keys()))

        # test null access key
        rc = result.add(AWSProfile('profile', None, 'value'))

        self.assertFalse(rc)
        self.assertEqual(1, len(result.profiles.keys()))

        # test empty access key
        rc = result.add(AWSProfile('profile', '', 'value'))

        self.assertFalse(rc)
        self.assertEqual(1, len(result.profiles.keys()))

        # test null profile name
        rc = result.add(AWSProfile(None, 'key', 'value'))

        self.assertFalse(rc)
        self.assertEqual(1, len(result.profiles.keys()))

        # test empty profile name
        rc = result.add(AWSProfile('', 'key', 'value'))

        self.assertFalse(rc)
        self.assertEqual(1, len(result.profiles.keys()))


    def test_get(self):
        result = AWSCredentials(one=AWSProfile('one', 'key one', 'key two'))
        test = result.get('one')

        self.assertIsNotNone(test)
        self.assertTrue(isinstance(test, AWSProfile))
        self.assertEqual('key one', test.aws_access_key_id)
        self.assertEqual('key two', test.aws_secret_access_key)

    def test_ls(self):
        result = AWSCredentials(one=AWSProfile('one', 'a', 'b'), two=AWSProfile('two', 'a', 'b'))
        self.assertEqual(set(['one', 'two']), set(result.ls()))


class AWSProfileTestCase(unittest.TestCase):

    def test_constructor(self):
        fixture = AWSProfile('profile one', 'access key id', 'secret access key')

        self.assertEqual('profile one', fixture.name)
        self.assertEqual('access key id', fixture.key_id)
        self.assertEqual('secret access key', fixture.secret_key)

    def test_format(self):
        fixture = AWSProfile(None, 'a', 'b')
        result_export = "export AWS_ACCESS_KEY_ID=a\nexport AWS_SECRET_ACCESS_KEY=b"
        result_no_export = "AWS_ACCESS_KEY_ID=a\nAWS_SECRET_ACCESS_KEY=b"

        self.assertEqual(result_export, fixture.format())
        self.assertEqual(result_no_export, fixture.format(export=False))

    def test_access_key_id(self):
        self.assertEqual('access key id', AWSProfile(None, 'access key id', None).aws_access_key_id)

    def test_secret_access_key(self):
        self.assertEqual('secret access key', AWSProfile(None, None, 'secret access key').aws_secret_access_key)
