use dirs;
use lib_error::*;
use std::env;
use std::path::{Path, PathBuf};

/// Store information needed to move between different shell environments.
/// This will be useful when you use the same server between a desktop and a laptop.
pub struct Environment {
    pub cwd: PathBuf,
    epic: Option<String>,
    pub home_dir: PathBuf,
    /// Hold cwd rebased on home, speeds up some operations,
    pub(crate) cwd_rebased: PathBuf,
}

impl Environment {
    /// Standardize the way to encode paths for this application, at this time
    /// this is a lossy encoding. Building the Environment will ensure that it
    /// makes sense at least for the home and current directory.
    pub fn encode_path(path: &Path) -> String {
        path.to_string_lossy().into()
    }

    pub fn build(epic: Option<String>) -> Result<Environment> {
        let home_dir = match dirs::home_dir() {
            Some(d) => d,
            None => return Err("cannot get home directory location".into()),
        };
        let cwd = env::current_dir()?;
        let cwd_rebased = Self::normalize_base_dir(cwd.clone(), &home_dir, "~")?;
        Ok(Environment {
            cwd,
            epic,
            home_dir,
            cwd_rebased,
        })
    }

    pub fn epic(&self) -> Option<&str> {
        self.epic.as_ref().map(|e| e.as_str())
    }

    /// If path is inside the base then replace the base prefix with the given replacement.
    pub(crate) fn normalize_base_dir(
        path: PathBuf,
        base: &Path,
        replacement: &str,
    ) -> Result<PathBuf> {
        if path.starts_with(base) {
            let relative = path.strip_prefix(base).map_err(|_| "strip prefix")?;
            let mut out = PathBuf::new();
            out.push(replacement);
            if relative.components().next().is_some() {
                out.push(relative);
            }
            Ok(out)
        } else {
            Ok(path)
        }
    }

    /// Check if this path is already rebased on the home folder.
    pub(crate) fn is_rebased_on_home(path: &Path) -> bool {
        match path.components().next() {
            Some(a) => a.as_os_str() == "~",
            None => false,
        }
    }

    /// Rebase the file name in the current environment. Try both the current
    /// working folder and the home directory. This is only valid for display
    /// since it depends on the current home.
    pub fn rebase(&self, path: PathBuf) -> Result<PathBuf> {
        let on_cwd = self.rebase_on_cwd(path)?;
        self.rebase_on_home(on_cwd)
    }

    /// Rebase just on cwd, for testing.
    pub(crate) fn rebase_on_cwd(&self, path: PathBuf) -> Result<PathBuf> {
        // Need to check if the incoming path is already relative to home, cwd may be more specific.
        let matching_cwd = if Self::is_rebased_on_home(&path) {
            // Select the cwd represenation in the same format as the input.
            &self.cwd_rebased
        } else {
            &self.cwd
        };
        if path.starts_with(matching_cwd) {
            Self::normalize_base_dir(path, matching_cwd, ".")
        } else {
            Ok(path)
        }
    }

    /// Rebase just on home, this is suitable to save in the database.
    pub fn rebase_on_home(&self, path: PathBuf) -> Result<PathBuf> {
        Self::normalize_base_dir(path, &self.home_dir, "~")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_home_path() {
        let normalized =
            Environment::normalize_base_dir(Path::new("/haha").into(), Path::new("/haha"), "~")
                .unwrap();
        assert_eq!("~", normalized.to_str().unwrap());
    }

    #[test]
    fn normalize_in_sub_folder() {
        let normalized =
            Environment::normalize_base_dir(Path::new("/haha/one").into(), Path::new("/haha"), "~")
                .unwrap();
        assert_eq!("~/one", normalized.to_str().unwrap());
    }

    #[test]
    fn normalize_tilde_in_sub_folder() {
        let normalized = Environment::normalize_base_dir(
            Path::new("~/haha/one").into(),
            Path::new("~/haha"),
            ".",
        ).unwrap();
        assert_eq!("./one", normalized.to_str().unwrap());
    }

    #[test]
    fn normalize_empty_home() {
        let normalized =
            Environment::normalize_base_dir(Path::new("/haha").into(), Path::new(""), "~").unwrap();
        assert_eq!("/haha", normalized.to_str().unwrap());
    }

    #[test]
    fn test_is_rebased_on_home() {
        assert!(Environment::is_rebased_on_home(&Path::new("~/foo")));
        assert!(!Environment::is_rebased_on_home(&Path::new("/foo")));
    }

    /// Build test Environment.
    fn env(cwd: &str, home_dir: &str) -> Environment {
        let cwd = PathBuf::from(cwd);
        let home_dir = PathBuf::from(home_dir);
        let cwd_rebased = Environment::normalize_base_dir(cwd.clone(), &home_dir, "~").unwrap();
        Environment {
            cwd: cwd,
            epic: None,
            home_dir,
            cwd_rebased,
        }
    }

    #[test]
    fn not_home_path() {
        let e = env("", "/home/username");
        assert_eq!(Path::new("/haha"), e.rebase("/haha".into()).unwrap());
        let normalized = Environment::normalize_base_dir(
            Path::new("/haha").into(),
            Path::new("/home/username"),
            "~",
        ).unwrap();
        assert_eq!("/haha", normalized.to_str().unwrap());
    }

    #[test]
    fn empty_home_path() {
        let e = env("", "");
        assert_eq!(
            Path::new("/home/username/dev"),
            e.rebase("/home/username/dev".into()).unwrap()
        );
        let normalized =
            Environment::normalize_base_dir(Path::new("/haha").into(), Path::new(""), "~").unwrap();
        assert_eq!("/haha", normalized.to_str().unwrap());
    }

    #[test]
    fn rebase_cwd_in_home() {
        let e = env("/home/username/dev", "/home/username");
        assert_eq!(
            Path::new("./foo"),
            e.rebase("/home/username/dev/foo".into()).unwrap()
        );
        assert_eq!(
            Path::new("~/dev/foo"),
            e.rebase_on_home("/home/username/dev/foo".into()).unwrap()
        );
    }

    #[test]
    fn cwd_rebased() {
        // when cwd under home
        let e = env("/home/username/dev", "/home/username");
        assert_eq!(Path::new("~/dev"), e.cwd_rebased);

        // when cwd not under home
        let e = env("/home/username/dev", "/home/foo");
        assert_eq!(Path::new("/home/username/dev"), e.cwd_rebased);
    }

    #[test]
    fn rebase_on_home_tilde() {
        let e = env("/home/username/dev", "/home/username");
        assert_eq!(
            Path::new("~/dev/foo"),
            e.rebase_on_home("/home/username/dev/foo".into()).unwrap()
        );

        // already rebased
        assert_eq!(
            Path::new("~/dev/foo"),
            e.rebase_on_home("~/dev/foo".into()).unwrap()
        );
    }

    #[test]
    fn rebase_on_cwd_tilde() {
        let e = env("/home/username/dev", "/home/username");
        let input = Path::new("~/dev/foo");
        assert!(Environment::is_rebased_on_home(&input));
        assert_eq!(Path::new("./foo"), e.rebase_on_cwd(input.into()).unwrap());
    }

    #[test]
    fn rebase_tilde() {
        let e = env("/home/username/dev", "/home/username");
        let input = Path::new("~/dev/foo");
        assert_eq!(Path::new("./foo"), e.rebase(input.into()).unwrap());
    }
}
