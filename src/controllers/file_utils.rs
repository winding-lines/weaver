use ::cli;
use ::errors::*;
use std::fs;
use std::fs::File;
use std::io::{Read, Write, stdin};
use std::path::{Path,PathBuf};

/// Load the content of the given file.
pub fn read_content(path: &Path) -> Result<String> {
    File::open(path)
        .and_then(|mut file| {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map(|_| contents)
        }).chain_err(|| "read from file")
}

/// Write the content to the given file.
pub fn write_content<T: AsRef<str>>(path: &Path, content: T) -> Result<()> {
    File::create(path)
        .and_then(|mut f|
            f.write_all(content.as_ref().as_bytes()))
        .chain_err(|| "writing entity")
}

/// Create if needed and then build a PathBuf to the global application folder.
pub fn app_folder() -> Result<PathBuf> {
    use std::env;
    if let Some(home) = env::home_dir() {
        let mut path = PathBuf::new();
        path.push(home);
        path.push(cli::APP_FOLDER);
        if !path.exists() {
            fs::create_dir(&path).chain_err(|| "create weaver folder")?;
        }
        Ok(path)
    } else {
        Err(Error::from_kind(ErrorKind::from("cannot get home folder")))
    }
}

/// Read up to limit lines from stdin.
pub fn read_stdin(limit: usize) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut count = 0;
    while count < limit {
        count += 1;
        let mut input = String::new();
        if stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim();
            if !trimmed.is_empty() {
                out.push(String::from(trimmed));
            }
        } else {
            break;
        }
    }
    Ok(out)
}

