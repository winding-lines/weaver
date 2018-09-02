//! Provide Python access the encrypted repo.
//!
//! Install with `python setup.py install` (after setting up your virtual environment).
//! Use like the following, you should prompt the user for the Repo
//! password unless it's already in the Keyring (Key Chain on OS X).
//!
//! ```python
//!
//! import weaver
//!
//! print(weaver.repo_list(password))
//!
//! ```

#![allow(dead_code)]

extern crate bincode;
#[macro_use]
extern crate cpython;
extern crate lib_error;
extern crate lib_goo;
extern crate lib_index;

mod repo;
mod error;

use repo::repo_list;

// add bindings to the generated python module
py_module_initializer!(weaver2py, initweaver2py, PyInit_weaver2py, |py, m| {
    try!(m.add(py, "__doc__", "Interface to the Weaver encrypted text repo."));
    try!(m.add(py, "repo_list", py_fn!(py, repo_list(password: String))));
    Ok(())
});

