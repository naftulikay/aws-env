use super::{AwsCredentialsFile, AwsProfile, AwsProfileLookup};
use crate::loader::FileSource;
use indexmap::IndexMap;
use zeroize::Zeroizing;

#[test]
fn test_name_lookup() {
    let root_dir = crate::utils::homedir().join(".aws");
    let root_file = root_dir.join("credentials");

    let mut lookup = AwsProfileLookup::new();
    let mut root_file = AwsCredentialsFile {
        file: FileSource::from_path(root_file),
        profiles: IndexMap::new(),
    };

    for (profile_name, access_key_id) in [("a", "1"), ("b", "2"), ("c", "3")] {
        root_file.push(AwsProfile {
            name: profile_name.into(),
            file: root_file.file.clone(),
            access_key_id: access_key_id.into(),
            secret_access_key: Zeroizing::new(String::new()),
            session_token: None,
        });
    }

    lookup.insert(root_file);

    // profile a
    assert!(lookup.by_name("a").is_some());

    {
        let profile = lookup.by_name("a").unwrap();

        assert_eq!("a", profile.name.as_str());
        assert_eq!("1", profile.access_key_id.as_str());
    }

    // profile b
    assert!(lookup.by_name("b").is_some());

    {
        let profile = lookup.by_name("b").unwrap();

        assert_eq!("b", profile.name.as_str());
        assert_eq!("2", profile.access_key_id.as_str());
    }

    // profile c
    assert!(lookup.by_name("c").is_some());

    {
        let profile = lookup.by_name("c").unwrap();

        assert_eq!("c", profile.name.as_str());
        assert_eq!("3", profile.access_key_id.as_str());
    }

    // okay, insert a non-root file
    let creds_d = root_dir.join("credentials.d");

    let mut f99 = AwsCredentialsFile {
        file: FileSource::from_path(creds_d.join("99-something.ini")),
        profiles: IndexMap::new(),
    };

    f99.push(AwsProfile {
        name: "d".into(),
        access_key_id: "4".into(),
        secret_access_key: Zeroizing::new(String::new()),
        session_token: None,
        file: f99.file.clone(),
    });

    lookup.insert(f99);

    // test failing over from the root file to the last file in the credentials.d directory
    assert!(lookup.by_name("d").is_some());

    {
        let profile = lookup.by_name("d").unwrap();

        assert_eq!("d", profile.name);
        assert_eq!("4", profile.access_key_id);
    }

    // test duplicate profiles, i.e. priority
    let mut f98 = AwsCredentialsFile {
        file: FileSource::from_path(creds_d.join("98-other-thing.asc")),
        profiles: IndexMap::new(),
    };

    f98.push(AwsProfile {
        name: "d".into(),
        access_key_id: "5".into(),
        secret_access_key: Zeroizing::new(String::new()),
        session_token: None,
        file: f98.file.clone(),
    });

    assert!(lookup.by_name("d").is_some());

    {
        let profile = lookup.by_name("d").unwrap();

        assert_eq!("d", profile.name.as_str());
        assert_eq!("4", profile.access_key_id.as_str());
        assert_eq!(
            FileSource::from_path(creds_d.join("99-something.ini")),
            profile.file
        );
    }

    // finally, test that encrypted files always win
    let mut f99_enc = AwsCredentialsFile {
        file: FileSource::from_path(creds_d.join("99-something.asc")),
        profiles: IndexMap::new(),
    };

    f99_enc.push(AwsProfile {
        name: "d".into(),
        access_key_id: "100".into(),
        secret_access_key: Zeroizing::new(String::new()),
        session_token: None,
        file: f99_enc.file.clone(),
    });

    // TODO this is where things break; breakage occurs because file path sort incorporates file extension
    lookup.insert(f99_enc);

    assert!(lookup.by_name("d").is_some());

    {
        let profile = lookup.by_name("d").unwrap();

        assert_eq!("d", profile.name.as_str());
        assert_eq!("100", profile.access_key_id.as_str());
        assert_eq!(
            FileSource::from_path(creds_d.join("99-something.asc")),
            profile.file
        );
    }
}

#[test]
fn test_prefix_lookup() {}
