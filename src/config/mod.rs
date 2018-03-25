pub mod file_utils;

#[derive(Debug)]
pub enum ServerRun {
    Foreground,
    Daemonize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutputKind {
    Copy,
    Print,
    Run,
}
