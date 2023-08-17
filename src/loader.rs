mod credentials_file;
pub(crate) mod profile;

pub use credentials_file::AwsCredentialsFile;
pub use profile::AwsProfile;

use crate::utils;

use indexmap::IndexMap;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::{self as stream, StreamExt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSource {
    pub path: PathBuf,
    pub encrypted: bool,
}

impl FileSource {
    pub fn is_root(&self) -> bool {
        utils::homedir()
            .join(".aws")
            .join("credentials")
            .eq(&self.path)
    }

    pub fn prefix(&self) -> Option<String> {
        if self.is_root() {
            None
        } else {
            if let Some(stem) = self.path.file_stem() {
                Some(stem.to_string_lossy().to_string())
            } else {
                None
            }
        }
    }

    pub fn from_path<P: AsRef<Path>>(p: P) -> Self {
        Self {
            path: p.as_ref().into(),
            encrypted: if let Some(extension) = p.as_ref().extension() {
                let extension = extension.to_str().expect("path is not valid utf-8");

                extension.eq("asc") || extension.eq("gpg") || extension.eq("pgp")
            } else {
                false
            },
        }
    }

    pub fn home_path(&self) -> PathBuf {
        utils::strip_homedir(&self.path)
    }
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type ProfilesParsed = IndexMap<String, AwsProfile>;

pub struct AwsCredentials {
    //
    pub sources: BTreeSet<AwsCredentialsFile>,
}

impl AwsCredentials {
    pub async fn load_all() -> Result<Self, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("unable to get user's home directory")?;
        let aws_config_dir = home.join(".aws");

        if !aws_config_dir.exists() {
            return Err(format!("{} directory does not exist", aws_config_dir.display()).into());
        }

        if !aws_config_dir.is_dir() {
            return Err(format!("{} is not a directory", aws_config_dir.display()).into());
        }

        let creds_d = aws_config_dir.join("credentials.d");

        // NOTE we _could_ abort here if !creds_d.is_dir(), but we are chaining on ~/.aws/credentials

        let (plain_sync, encrypted_sync) =
            (Arc::new(Semaphore::new(32)), Arc::new(Semaphore::new(4)));

        let creds_files: Vec<PathBuf> = match fs::read_dir(&creds_d).await {
            Ok(creds) => {
                ReadDirStream::new(creds)
                    .filter_map(|f| f.ok())
                    .map(|f| f.path())
                    .collect()
                    .await
            }
            Err(_) => {
                log::debug!("No $HOME/.aws/credentials.d directory found");
                vec![]
            }
        };

        let handles = stream::iter(creds_files)
            .chain(stream::iter(vec![aws_config_dir.join("credentials")]))
            .filter(|p| p.is_file())
            .map(|p| {
                let (plain_permit, encrypted_permit) = (plain_sync.clone(), encrypted_sync.clone());

                tokio::spawn(async {
                    let name = p.file_name().unwrap().to_string_lossy();

                    if name.ends_with(".asc") || name.ends_with(".gpg") || name.ends_with(".pgp") {
                        Some({
                            let work = encrypted_permit.acquire_owned().await.unwrap();
                            let r = AwsCredentialsFile::load_encrypted(p).await;
                            drop(work);
                            r
                        })
                    } else if name.ends_with(".ini") || name.ends_with("") {
                        Some({
                            let work = plain_permit.acquire_owned().await.unwrap();
                            let r = AwsCredentialsFile::load(p).await;
                            drop(work);
                            r
                        })
                    } else {
                        log::debug!("Skipping file with unknown extension {}", p.display());
                        None
                    }
                })
            })
            .collect::<Vec<JoinHandle<_>>>();

        let mut credentials = BTreeSet::new();

        for h in handles.await {
            let h = h.await.unwrap();

            if let Some(t) = h {
                if t.is_err() {
                    log::warn!("{}", t.unwrap_err());
                } else {
                    let f = t.unwrap();
                    log::info!("Loaded: {}", utils::strip_homedir(&f.file.path).display());

                    credentials.insert(f);
                }
            }
        }

        Ok(Self {
            sources: credentials,
        })
    }
}
