use lib_error::*;
use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::process::Command;

pub fn run<T>(command: T) -> Result<i32>
    where T: AsRef<OsStr> + Debug {
    let shell = env::var("SHELL").chain_err(|| "missing shell")?;
    debug!("using shell {} to run {:?}", shell, command.as_ref());
    let mut cmd = Command::new(shell);
    cmd.arg("-c");
    cmd.arg(command.as_ref());
    let status = cmd.status()
        .chain_err(|| "shell_proxy")?;
    Ok(status.code().unwrap_or(-127))
}