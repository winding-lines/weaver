extern crate actix_web;
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate regex;
extern crate reqwest;
extern crate sys_info;
extern crate tantivy;
use std::fmt::{self, Display};
use std::result;

use std::convert::From;

#[derive(Debug)]
pub struct WeaverError {
    inner: failure::Context<WeaverErrorKind>,
}

#[derive(Clone, Debug, Fail)]
pub enum WeaverErrorKind {
    #[fail(display = "{}", _0)]
    Generic(&'static str),
    #[fail(display = "{}", _0)]
    WithText(String),
    #[fail(display = "sysinfo")]
    SysInfo,
    #[fail(display = "network")]
    Network,
    #[fail(display = "local")]
    Local,
    #[fail(display = "data")]
    DataLayer, 
}

pub type Result<T> = result::Result<T, WeaverError>;

pub use failure::ResultExt;

impl failure::Fail for WeaverError {
    fn cause(&self) -> Option<&failure::Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for WeaverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl WeaverError {
    pub fn kind(&self) -> &WeaverErrorKind {
        self.inner.get_context()
    }

    pub fn display(&self) {
        use failure::Fail;

        for e in Fail::iter_causes(self) {
            eprintln!("caused by: {}", e);
        }

        if let Some(backtrace) = self.backtrace() {
            eprintln!("backtrace: {:?}", backtrace);
        }
    }
}

impl From<WeaverErrorKind> for WeaverError {
    fn from(kind: WeaverErrorKind) -> WeaverError {
        WeaverError {
            inner: failure::Context::new(kind),
        }
    }
}

impl From<failure::Context<WeaverErrorKind>> for WeaverError {
    fn from(inner: failure::Context<WeaverErrorKind>) -> WeaverError {
        WeaverError { inner: inner }
    }
}

impl From<&'static str> for WeaverErrorKind {
    fn from(reason: &'static str) -> Self {
        WeaverErrorKind::Generic(reason)
    }
}

impl From<&'static str> for WeaverError {
    fn from(reason: &'static str) -> Self {
        WeaverErrorKind::Generic(reason).into()
    }
}

impl From<String> for WeaverError {
    fn from(reason: String) -> Self {
        WeaverErrorKind::WithText(reason).into()
    }
}

impl From<::sys_info::Error> for WeaverError {
    fn from(_err: ::sys_info::Error) -> Self {
        WeaverErrorKind::SysInfo.into()
    }
}

impl From<::reqwest::Error> for WeaverError {
    fn from(_err: ::reqwest::Error) -> Self {
        WeaverErrorKind::Network.into()
    }
}

impl From<::std::io::Error> for WeaverError {
    fn from(_err: ::std::io::Error) -> Self {
        WeaverErrorKind::Local.into()
    }
}

impl actix_web::ResponseError for WeaverError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().into()
    }
}

impl From<tantivy::TantivyError> for WeaverError {
    fn from(_err: tantivy::TantivyError) -> Self {
        WeaverErrorKind::DataLayer.into()
    }
}

impl From<::diesel::result::Error> for WeaverError {
    fn from(_err: ::diesel::result::Error) -> Self {
        WeaverErrorKind::DataLayer.into()
    }
}

impl From<::diesel::ConnectionError> for WeaverError {
    fn from(_err: ::diesel::ConnectionError) -> Self {
        WeaverErrorKind::DataLayer.into()
    }
}

impl From<::regex::Error> for WeaverError {
    fn from(_err: ::regex::Error) -> Self {
        WeaverErrorKind::DataLayer.into()
    }
}
