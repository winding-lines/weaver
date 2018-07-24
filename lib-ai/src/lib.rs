//! Hold online AI-like algorithms. 
//! For offline processing see the [weaver-recommend](https://gitlab.com/lab-flow/weaver-recommend) project.

extern crate url;
extern crate lib_error;
extern crate lib_goo;
#[macro_use]
extern crate log;

pub mod recommender;
pub mod normalize;