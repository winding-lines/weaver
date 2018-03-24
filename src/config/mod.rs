pub mod file_utils;

#[derive(Debug)]
pub enum ServerRun {
    Foreground,
    Daemonize,
}

#[derive(Debug)]
pub enum ActionKind {
    Copy,
    Print,
    Run,
}
