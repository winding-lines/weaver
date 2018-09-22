extern crate actix_web;
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate regex;
extern crate reqwest;
extern crate sys_info;
extern crate tantivy;
use std::result;

use std::convert::From;

#[derive(Debug, Fail)]
pub enum WeaverError {
    #[fail(display = "weaver error {:?}", _0)]
    Generic(String),
    #[fail(display = "error {:?}", _0)]
    Server(actix_web::Error),
    #[fail(display = "error {:?}", _0)]
    Tantivy(tantivy::TantivyError),
    #[fail(display = "error {:?}", _0)]
    Diesel(::diesel::result::Error),
    #[fail(display = "error {:?}", _0)]
    DieselConnection(::diesel::ConnectionError),
    #[fail(display = "error {:?}", _0)]
    SysInfo(::sys_info::Error),
    #[fail(display = "error {:?}", _0)]
    Io(#[cause] ::std::io::Error),
    #[fail(display = "error {:?}", _0)]
    Reqwest(::reqwest::Error),
    #[fail(display = "error {:?}", _0)]
    Regex(::regex::Error),
}

pub type Result<T> = result::Result<T, WeaverError>;

impl<'a> From<&'a str> for WeaverError {
    fn from(reason: &'a str) -> Self {
        WeaverError::Generic(String::from(reason))
    }
}

impl From<String> for WeaverError {
    fn from(reason: String) -> Self {
        WeaverError::Generic(reason)
    }
}

impl From<tantivy::TantivyError> for WeaverError {
    fn from(terror: tantivy::TantivyError) -> Self {
        WeaverError::Tantivy(terror).into()
    }
}

impl From<::diesel::result::Error> for WeaverError {
    fn from(err: ::diesel::result::Error) -> Self {
        WeaverError::Diesel(err).into()
    }
}

impl From<::diesel::ConnectionError> for WeaverError {
    fn from(err: ::diesel::ConnectionError) -> Self {
        WeaverError::DieselConnection(err).into()
    }
}

impl From<::sys_info::Error> for WeaverError {
    fn from(err: ::sys_info::Error) -> Self {
        WeaverError::SysInfo(err).into()
    }
}

impl From<::std::io::Error> for WeaverError {
    fn from(err: ::std::io::Error) -> Self {
        WeaverError::Io(err).into()
    }
}

impl From<::reqwest::Error> for WeaverError {
    fn from(err: ::reqwest::Error) -> Self {
        WeaverError::Reqwest(err).into()
    }
}

impl From<::regex::Error> for WeaverError {
    fn from(err: ::regex::Error) -> Self {
        WeaverError::Regex(err).into()
    }
}

impl WeaverError {
    pub fn display(&self) {
        use failure::Fail;

        for e in Fail::iter_causes(self) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = self.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }
    }
}

impl actix_web::ResponseError for WeaverError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().into()
    }
}
