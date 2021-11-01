use super::{AwsCredentialsFile, FileSource};

use std::collections::BTreeSet;

#[test]
fn test_ordering() {
    let home = dirs::home_dir().unwrap();
    let aws = home.join(".aws");
    let credentials_d = aws.join("credentials.d");

    let mut sorted = BTreeSet::new();

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: false,
            path: credentials_d.join("50-a"),
        },
    });

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: false,
            path: credentials_d.join("90-b"),
        },
    });

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: false,
            path: credentials_d.join("00-c"),
        },
    });

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: false,
            path: aws.join("credentials"),
        },
    });

    let mut iter = sorted.iter();

    assert_eq!(credentials_d.join("00-c"), iter.next().unwrap().file.path);
    assert_eq!(credentials_d.join("50-a"), iter.next().unwrap().file.path);
    assert_eq!(credentials_d.join("90-b"), iter.next().unwrap().file.path);
    assert_eq!(aws.join("credentials"), iter.next().unwrap().file.path);
}

#[test]
fn test_encrypted_ordering() {
    let home = dirs::home_dir().unwrap();
    let aws = home.join(".aws");
    let credentials_d = aws.join("credentials.d");

    let mut sorted = BTreeSet::new();

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: false,
            path: credentials_d.join("50-a"),
        },
    });

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: true,
            path: credentials_d.join("50-a"),
        },
    });

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: true,
            path: credentials_d.join("00-b"),
        },
    });

    sorted.insert(AwsCredentialsFile {
        profiles: Default::default(),
        file: FileSource {
            encrypted: false,
            path: credentials_d.join("00-b"),
        },
    });

    let mut iter = sorted.iter();
    let mut current = iter.next().unwrap();

    assert_eq!(credentials_d.join("00-b"), current.file.path);
    assert!(!current.file.encrypted);

    current = iter.next().unwrap();

    assert_eq!(credentials_d.join("00-b"), current.file.path);
    assert!(current.file.encrypted);

    current = iter.next().unwrap();

    assert_eq!(credentials_d.join("50-a"), current.file.path);
    assert!(!current.file.encrypted);

    current = iter.next().unwrap();

    assert_eq!(credentials_d.join("50-a"), current.file.path);
    assert!(current.file.encrypted);
}
