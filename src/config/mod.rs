pub mod file_utils;

#[derive(Debug)]
pub enum ServerRun {
    Foreground,
    Daemonize,
}

