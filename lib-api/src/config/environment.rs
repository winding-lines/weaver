use std::borrow::Cow;
use std::path::Path;
use std::env;
use weaver_error::{Result, ResultExt};

/// Store information needed to move between different environments.
pub struct Environment {
    cwd: String,
    epic: Option<String>,
    pub home_dir: String,
}

impl Environment {

    /// Standardize the way to encode paths for this application, at this time
    /// this is a lossy encoding. Building the Environment will ensure that it
    /// makes sense at least for the home and current directory.
    pub fn encode_path(path: &Path) -> String {
        path.to_string_lossy().into()
    }

    pub fn build(epic: Option<String>) -> Result<Environment> {

        let home_dir: String = match env::home_dir() {
            Some(p) => {
                match p.as_path().to_str() {
                    Some(s) => s.into(),
                    None =>  return Err("home directory is not utf-8".into())
                }
            },
            None => return Err("cannot get home directory location".into())
        };
        let cwd_path = env::current_dir()
            .chain_err(|| "getting current directory")?;
        let cwd: String = match cwd_path.to_str() {
            Some(s) => s.into(),
            None => return Err("cannot utf-8 encode the current directory".into())
        };

        Ok(Environment{
            cwd,
            epic,
            home_dir,
        })
    }

    pub fn cwd(&self) -> &str {
       self.cwd.as_ref()
    }

    pub fn epic(&self) -> Option<&str> {
        self.epic.as_ref().map(|e| e.as_str())
    }

    pub fn localized_path<'a>(&self, path: &'a str) -> Cow<'a, str> {
        if path.starts_with(&self.home_dir) {
            Cow::Borrowed(path)
        } else if path.starts_with("/home") || path.starts_with("/Users") {
            let mut buf = String::with_capacity(path.len());

            let mut hd_pos = 0;
            let mut in_pos = 0;
            let hd_len = self.home_dir.len();
            let in_len = path.len();

            // invariant going in the loop: `buf` has the processed data up to hd_pos & in_pos
            loop {
                // no more to replace, copy the rest of the input
                if hd_pos == hd_len {
                    // the home_dir may be missing the final '/'
                    if hd_len > 0 && !buf.ends_with('/') {
                        buf.push('/');
                    };
                    buf.push_str(&path[in_pos..in_len]);
                    in_pos = in_len;
                }

                // no more input, return what we have so far
                if in_pos == in_len {
                    return Cow::Owned(buf);
                }

                // use the next component from the home_directory;
                let hd_next = self.home_dir[hd_pos..].find('/')
                    .map(|i| i + hd_pos + 1).unwrap_or(hd_len);
                buf.push_str(&self.home_dir[hd_pos..hd_next]);
                hd_pos = hd_next;

                // skip the next component in the input
                in_pos = path[in_pos..].find('/')
                    .map(|i| i + in_pos + 1).unwrap_or(in_len);
            }
        } else {
            Cow::Borrowed(path)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_home_path() {
        let e = Environment {
            cwd: String::new(),
            epic: None,
            home_dir: "/home/username".into(),
        };
        assert_eq!("/haha", e.localized_path("/haha"))
    }

    #[test]
    fn empty_home_path() {
        let e = Environment {
            cwd: String::new(),
            epic: None,
            home_dir: "".into(),
        };
        assert_eq!("/home/username/dev", e.localized_path("/home/username/dev"))
    }

    #[test]
    fn already_home_path() {
        let e = Environment {
            cwd: String::new(),
            epic: None,
            home_dir: "/home/username".into(),
        };
        assert_eq!("/home/username/dev", e.localized_path("/home/username/dev"))
    }

    #[test]
    fn other_home_path() {
        let e = Environment {
            cwd: String::new(),
            epic: None,
            home_dir: "/home/username".into(),
        };
        assert_eq!("/home/username/dev", e.localized_path("/Users/other/dev"));
    }

    #[test]
    fn other_short() {
        let e = Environment {
            cwd: String::new(),
            epic: None,
            home_dir: "/home/username".into(),
        };
        assert_eq!("/home/", e.localized_path("/Users"));
    }
}
