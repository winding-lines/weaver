// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate diesel;
#[macro_use]
extern crate error_chain;
extern crate sys_info;

// `error_chain!` creates.

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Diesel(::diesel::result::Error);
        SysInfo(::sys_info::Error);
    }
}


