#[cfg(test)]
mod tests;

use super::{AwsProfile, Error, FileSource, ProfilesParsed};

use crate::ini;
use crate::utils;

use indexmap::IndexMap;

use std::cmp::Ordering;
use std::convert::{AsRef, Into};
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::process::Stdio;

use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use zeroize::Zeroizing;

const INI_ACCESS_KEY_ID_TAG: &'static str = "aws_access_key_id";
const INI_SECRET_ACCESS_KEY_TAG: &'static str = "aws_secret_access_key";
const INI_SESSION_TOKEN_TAG: &'static str = "aws_session_token";

pub struct AwsCredentialsFile {
    pub profiles: IndexMap<String, AwsProfile>,
    pub file: FileSource,
}

impl AwsCredentialsFile {
    pub fn push(&mut self, profile: AwsProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    pub fn prefix(&self) -> Option<String> {
        self.file.prefix()
    }

    pub fn is_root(&self) -> bool {
        self.file.is_root()
    }
}

impl PartialEq for AwsCredentialsFile {
    fn eq(&self, other: &Self) -> bool {
        // a file is equivalent if it has the same path and the same status of encryption
        self.file.path.eq(&other.file.path) && self.file.encrypted && other.file.encrypted
    }
}

impl Eq for AwsCredentialsFile {}

impl Ord for AwsCredentialsFile {
    fn cmp(&self, other: &Self) -> Ordering {
        let master = dirs::home_dir()
            .expect("unable to get user home directory")
            .join(".aws")
            .join("credentials");

        if self.file.path.eq(&master) {
            // if I'm the root, greater
            Ordering::Greater
        } else if other.file.path.eq(&master) {
            // if the other is the root, less
            Ordering::Less
        } else {
            // otherwise, it's a bit more complicated
            if let (Some(stem), Some(other_stem)) =
                (self.file.path.file_stem(), other.file.path.file_stem())
            {
                if stem.eq(other_stem) {
                    // if both files have the same file stem
                    if self.file.encrypted && !other.file.encrypted {
                        // if I'm encrypted and the other is not, greater
                        return Ordering::Greater;
                    } else if !self.file.encrypted && other.file.encrypted {
                        // if I'm not encrypted and the other is, less
                        return Ordering::Less;
                    }
                }
            }

            // otherwise, regular path comparison
            self.file.path.cmp(&other.file.path)
        }
    }
}

impl PartialOrd for AwsCredentialsFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Display for AwsCredentialsFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AWSCredentialsFile{{profiles={profiles}, encrypted={encrypted}, file={file}}}",
            file = self.file.home_path().display(),
            encrypted = self.file.encrypted,
            profiles = self
                .profiles
                .keys()
                .into_iter()
                .map(|k| k.clone())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Debug for AwsCredentialsFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl AwsCredentialsFile {
    fn read_profiles<P: AsRef<Path>>(data: String, path: P) -> Result<ProfilesParsed, Error> {
        // when data is dropped, it will be securely erased
        let data = Zeroizing::new(data);

        // owns no values except the containers, all strings are references to sections of `data`
        let config = ini::parse(data.as_str());

        // storage for the profiles we have loaded
        let mut profiles = IndexMap::with_capacity(config.len());

        for (section_name, section) in config {
            if let (Some(Some(access_key_id)), Some(Some(secret_access_key))) = (
                section.get(INI_ACCESS_KEY_ID_TAG),
                section.get(INI_SECRET_ACCESS_KEY_TAG),
            ) {
                // if it has both an access key id and a secret access key, add it
                profiles.insert(
                    section_name.to_string(),
                    AwsProfile {
                        name: section_name.to_string(),
                        access_key_id: access_key_id.to_string(),
                        secret_access_key: Zeroizing::from(secret_access_key.to_string()),
                        // optionally add the session token if present
                        session_token: if let Some(Some(session_token)) =
                            section.get(INI_SESSION_TOKEN_TAG)
                        {
                            Some(Zeroizing::from(session_token.to_string()))
                        } else {
                            None
                        },
                        file: FileSource {
                            path: path.as_ref().to_path_buf(),
                            encrypted: false,
                        },
                    },
                );
            }
        }

        if profiles.len() > 0 {
            Ok(profiles)
        } else {
            Err("file contains no profiles".into())
        }
    }

    pub async fn load<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
        // read the file into memory asynchronously
        let (mut file, mut storage) = (fs::File::open(p.as_ref()).await?, String::new());
        file.read_to_string(&mut storage).await?;

        // load the INI from the string in memory
        let result = Self::read_profiles(storage, p.as_ref()).map_err(|e| {
            Into::<Error>::into(format!(
                "Unable to read profiles from {}: {}",
                utils::strip_homedir(p.as_ref()).display(),
                e
            ))
        })?;

        Ok(AwsCredentialsFile {
            file: FileSource {
                encrypted: false,
                path: p.as_ref().to_path_buf(),
            },
            profiles: result,
        })
    }

    pub async fn load_encrypted<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
        // gpg --batch -d {file}
        let mut decrypt = Command::new("gpg")
            .arg("--batch")
            .arg("-d")
            .arg(p.as_ref().to_str().expect("invalid utf-8 in filename"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let rc = decrypt.wait().await?;

        if !rc.success() {
            let stderr = decrypt.stderr;

            Err(format!(
                "Unable to decrypt file with gpg (rc={rc}): {p}{stderr}",
                p = p.as_ref().display(),
                rc = rc.code().unwrap(),
                stderr = if let Some(mut stderr) = stderr {
                    let mut output = String::new();

                    let _ = stderr.read_to_string(&mut output).await.unwrap();

                    format!(
                        "\n{}",
                        output
                            .lines()
                            .map(|line| format!("    {}", line))
                            .collect::<Vec<String>>()
                            .join("\n")
                    )
                } else {
                    "".to_string()
                }
            )
            .into())
        } else {
            let mut storage = String::new();

            if let Ok(_) = decrypt.stdout.unwrap().read_to_string(&mut storage).await {
                let result = Self::read_profiles(storage, p.as_ref()).map_err(|e| {
                    Into::<Error>::into(format!(
                        "Unable to read profiles from encrypted file {}: {}",
                        p.as_ref().display(),
                        e
                    ))
                })?;

                Ok(AwsCredentialsFile {
                    file: FileSource {
                        encrypted: true,
                        path: p.as_ref().to_path_buf(),
                    },
                    profiles: result,
                })
            } else {
                Err(format!("unable to read gpg stdout").into())
            }
        }
    }
}
