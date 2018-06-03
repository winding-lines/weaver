// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate actix_web;
extern crate diesel;
#[macro_use]
extern crate error_chain;
extern crate sys_info;

use std::convert;


// `error_chain!` creates.

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Diesel(::diesel::result::Error);
        SysInfo(::sys_info::Error);
        Io(::std::io::Error);
    }
}

// Allow our errors to be easily returned through the web apis.
impl convert::Into<actix_web::Error> for Error {
    fn into(self) -> actix_web::Error {
        actix_web::error::ErrorInternalServerError(format!("{}", self.description()))
    }
}

impl <'a> convert::From<&'a Error> for actix_web::Error {
    fn from(werror: &Error) -> Self {
        actix_web::error::ErrorInternalServerError(format!("{}", werror.description()))
    }
}

