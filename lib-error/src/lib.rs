//! Common error definitions for [Weaver Project](../weaver_project/index.html).

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate actix_web;
extern crate diesel;
#[macro_use]
extern crate error_chain;
extern crate regex;
extern crate reqwest;
extern crate sys_info;

use std::convert;

// `error_chain!` creates.

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Diesel(::diesel::result::Error);
        SysInfo(::sys_info::Error);
        Io(::std::io::Error);
        Reqwest(::reqwest::Error);
        Regex(::regex::Error);
    }
}

impl<'a> convert::From<&'a Error> for actix_web::Error {
    fn from(werror: &Error) -> Self {
        actix_web::error::ErrorInternalServerError(werror.description().to_string())
    }
}

impl convert::From<Error> for actix_web::Error {
    fn from(werror: Error) -> Self {
        actix_web::error::ErrorInternalServerError(werror.description().to_string())
    }
}
