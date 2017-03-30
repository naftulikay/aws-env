#!/usr/bin/env python
# -*- coding: utf-8 -*-

import tempfile
import mock
import unittest

from unittest import skip

from awsenv import AWSCredentials, AWSProfile


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

        self.credentials_file = tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', delete=True)

        for profile in profiles.keys():
            self.credentials_file.write("[{}]\n".format(profile))

            if profiles.get(profile).get('aws_access_key_id'):
                self.credentials_file.write("aws_access_key_id={}\n".format(profiles.get(profile).get('aws_access_key_id')))

            if profiles.get(profile).get('aws_secret_access_key'):
                self.credentials_file.write('aws_secret_access_key={}\n'.format(profiles.get(profile).get('aws_secret_access_key')))

            self.credentials_file.write("\n")

        self.credentials_file.flush()

    def test_from_path(self):
        result = AWSCredentials.from_path(self.credentials_file.name)

        # only two were fully valid
        self.assertEqual(2, len(result.profiles.keys()))
        # validate number one
        self.assertTrue(isinstance(result.profiles.get('one'), AWSProfile))
        self.assertEqual('one', result.profiles['one'].name)
        self.assertEqual('key one', result.profiles['one'].aws_access_key_id)
        self.assertEqual('key two', result.profiles['one'].aws_secret_access_key)
        # validate number two
        self.assertTrue('two', result.profiles['two'].name)
        self.assertEqual('another', result.profiles['two'].aws_access_key_id)
        self.assertEqual('thing', result.profiles['two'].aws_secret_access_key)

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
        result = AWSCredentials.from_path(self.credentials_file.name)
        test = result.get('one')

        self.assertIsNotNone(test)
        self.assertTrue(isinstance(test, AWSProfile))
        self.assertEqual('key one', test.aws_access_key_id)
        self.assertEqual('key two', test.aws_secret_access_key)

    def test_ls(self):
        result = AWSCredentials.from_path(self.credentials_file.name)
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
