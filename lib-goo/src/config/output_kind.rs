use std::fmt::{Display, Formatter, Result as FmtResult};

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

        let as_str = match self {
            Print => "Print",
            Run => "Run",
            Copy => "Copy",
        };
        f.write_str(as_str)
    }
}

/// Fully specify the output of the command.
#[derive(Clone, Debug)]
pub struct OutputKind {
    pub channel: Channel,
}
