pub(crate) mod ini;
pub(crate) mod utils;

mod loader;
#[cfg(test)]
mod tests;

use indexmap::IndexMap;

use std::collections::BTreeSet;

pub use loader::{AwsCredentials, AwsCredentialsFile, AwsProfile, Error};

pub struct AwsProfileLookup {
    files: BTreeSet<AwsCredentialsFile>,
}

impl<'a> AwsProfileLookup {
    pub fn new() -> Self {
        Self {
            files: Default::default(),
        }
    }

    pub fn insert(&mut self, value: AwsCredentialsFile) {
        self.files.insert(value);
    }

    pub fn insert_all<I: Iterator<Item = AwsCredentialsFile>>(&mut self, iter: I) {
        for profile in iter {
            self.files.insert(profile);
        }
    }

    /// Lookup the first profile with the given name in priority order.
    ///
    /// `AwsCredentialsFile` instances are sorted in ascending priority order in the underlying
    /// `BTreeSet`, such that the very _last_ item in the set has the highest priority, and the
    /// first item in the set has the lowest priority.
    pub fn by_name<S: AsRef<str>>(&'a self, name: S) -> Option<&'a AwsProfile> {
        for file in self.files.iter().rev() {
            if let Some(profile) = file.profiles.get(name.as_ref()) {
                return Some(profile);
            }
        }

        None
    }

    /// Lookup the first profile using a file stem prefix.
    ///
    /// If the prefix is `/`, only the root file will be used. Otherwise, the prefix will be used to
    /// match a file stem for a credentials file in `~/.aws/credentials.d`, such that a file named
    /// `abc123.ini` will yield a prefix of `abc123`. Note that it is possible that two files can
    /// have the same prefix, e.g. `abc123.ini` and `abc123.asc`; in this case, encrypted files are
    /// preferred over plain-text ones.
    pub fn by_prefix<S: AsRef<str>>(&'a self, prefix: S, name: S) -> Option<&'a AwsProfile> {
        // if we're in the root namespace, find all root files and try to lookup the first one with the given profile
        if prefix.as_ref().eq("/") {
            // NOTE for future compatibility reasons, it will be possible to have multiple root files, for example,
            //      ~/.aws/credentials, ~/.aws/credentials.asc. This is not currently implemented, but may be
            //      implemented later.
            for root_file in self.files.iter().rev().filter(|f| f.is_root()) {
                if let Some(profile) = root_file.profiles.get(name.as_ref()) {
                    return Some(profile);
                }
            }

            // if we can't find a root file, abort
            return None;
        }

        // we're not in the root namespace, so iterate in priority order for each file matching the specified prefix
        for file in self
            .files
            .iter()
            .rev()
            .filter(|f| !f.is_root())
            .filter(|f| f.prefix().unwrap().eq(prefix.as_ref()))
        {
            if let Some(profile) = file.profiles.get(name.as_ref()) {
                return Some(profile);
            }
        }

        // otherwise, we didn't find the profile
        None
    }

    /// Fetches a profile by its "URI," i.e. `/dev` or `something/prod`.
    pub fn by_uri<S: AsRef<str>>(&'a self, uri: S) -> Option<&'a AwsProfile> {
        for file in self.files.iter().rev() {
            for profile in file.profiles.values() {
                if profile.uri().eq(uri.as_ref()) {
                    return Some(profile);
                }
            }
        }

        None
    }

    /// List _all_ profiles regardless of overlapping aliases.
    pub fn list(&'a self) -> Vec<&'a AwsProfile> {
        // reserve at _least_ enough for the amount of files we have
        let mut result = Vec::with_capacity(self.files.len());

        for file in self.files.iter() {
            for profile in file.profiles.values() {
                result.push(profile);
            }
        }

        result
    }

    /// List profiles with only the highest priority profiles being shown.
    pub fn list_aliased(&'a self) -> Vec<&'a AwsProfile> {
        let mut storage = IndexMap::with_capacity(self.files.len());

        for file in self.files.iter() {
            for profile in file.profiles.values() {
                storage.insert(&profile.name, profile);
            }
        }

        storage.values().map(|x| *x).collect()
    }
}
