//! Hold online AI-like algorithms.
//! For offline processing see the [weaver-recommend](https://gitlab.com/lab-flow/weaver-recommend) project.

extern crate lib_error;
extern crate lib_goo;
extern crate url;
#[macro_use]
extern crate log;
extern crate chrono;

pub mod normalize;
pub mod recommender;
