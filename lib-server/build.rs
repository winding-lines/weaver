use std::process::Command;
use std::env;

fn main() {
    let shell = env::var("SHELL").unwrap_or(String::from("/bin/sh"));
    let status = Command::new(shell)
        .current_dir("web")
        .arg("./build.sh")
        .status()
        .expect("build failed");
    if !status.success() {
         ::std::process::exit(1);
    }
}
