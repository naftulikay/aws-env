#[cfg(test)]
mod tests;

use crate::loader::FileSource;

use std::fmt::{Debug, Display, Formatter};

use zeroize::Zeroizing;

pub struct AwsProfile {
    pub name: String,
    pub access_key_id: String,
    pub secret_access_key: Zeroizing<String>,
    pub session_token: Option<Zeroizing<String>>,
    pub file: FileSource,
}

impl AwsProfile {
    pub fn new<S: Into<String>>(
        name: S,
        access_key_id: S,
        secret_access_key: String,
        session_token: Option<String>,
        file: FileSource,
    ) -> Self {
        Self {
            name: name.into(),
            access_key_id: access_key_id.into(),
            secret_access_key: Zeroizing::new(secret_access_key),
            session_token: session_token.map(|t| Zeroizing::new(t)),
            file,
        }
    }

    pub fn is_in_root(&self) -> bool {
        self.file.is_root()
    }

    pub fn prefix(&self) -> Option<String> {
        self.file.prefix()
    }

    pub fn uri(&self) -> String {
        if let Some(prefix) = self.prefix() {
            format!("{prefix}/{name}", prefix = prefix, name = self.name)
        } else {
            format!("/{name}", name = self.name)
        }
    }
}

impl Display for AwsProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AWSProfile{{name={name}, access_key_id={access_key_id}, secret_access_key=***, file={file}}}",
            name=self.name,
            access_key_id=self.access_key_id,
            file=self.file.home_path().display(),
        )
    }
}

impl Debug for AwsProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
