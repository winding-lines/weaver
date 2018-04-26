use std::fmt::{Display, Formatter, Result as FmtResult};
pub mod file_utils;

pub const APP_FOLDER: &'static str = ".weaver";


#[derive(Debug)]
pub enum ServerRun {
    Foreground,
    Daemonize,
}

/// What information to output: just the command or with an additional context.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    PathWithCommand,
    Path,
    Command
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::Content::*;

        let  as_str = match self {
            &PathWithCommand => "Command with Path",
            &Path => "Path",
            &Command => "Command",
        };
        f.write_str(as_str)
    }
}

/// What Channel to output this information on.
#[derive(Clone, Debug, PartialEq)]
pub enum Channel {
    Copy,
    Print,
    Run,
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::Channel::*;

        let  as_str = match self {
            &Print => "Print",
            &Run => "Run",
            &Copy => "Copy",
        };
        f.write_str(as_str)
    }
}

/// Fully specify the output of the command.
#[derive(Clone,Debug)]
pub struct OutputKind {
    pub channel: Channel,
    pub content: Content,
}




