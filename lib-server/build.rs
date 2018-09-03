use std::process::Command;
use std::env;

fn main() {
    let shell = env::var("SHELL").expect("Shell env");
    Command::new(shell)
        .current_dir("web")
        .arg("./build.sh")
        .spawn()
        .expect("build failed");
}
