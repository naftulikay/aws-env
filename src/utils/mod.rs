use once_cell::sync::OnceCell;

use std::path::{Path, PathBuf};

static HOMEDIR: OnceCell<PathBuf> = OnceCell::new();

/// Get the home directory of the current user, caching the value.
///
/// This internally uses `dirs::home_dir` and a `once_cell::OnceCell`.
pub(crate) fn homedir() -> &'static PathBuf {
    HOMEDIR.get_or_init(|| {
        dirs::home_dir()
            .expect("unable to get home directory from environment")
            .to_path_buf()
    })
}

/// Strips the user's home directory from the path and replaces it with `~`
pub(crate) fn strip_homedir<'a, P: AsRef<Path> + 'a>(p: P) -> PathBuf {
    Path::new("~").join(p.as_ref().strip_prefix(homedir()).unwrap().to_path_buf())
}
