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

use cpython::{PyDict, PyErr, PyList, PyObject, PyResult, PyString, Python, PythonObject};
use lib_goo::config::db::PasswordSource;
use lib_goo::entities::PageContent;
use std::error::Error;

// add bindings to the generated python module
py_module_initializer!(weaver2py, initweaver2py, PyInit_weaver2py, |py, m| {
    try!(m.add(py, "__doc__", "Interface to the Weaver encrypted text repo."));
    try!(m.add(py, "repo_list", py_fn!(py, repo_list(password: String))));
    Ok(())
});

/// Translate any Error into a Py Error.
fn as_py_err<T:Error>(py: Python, werr: T) -> PyErr {
    PyErr::new::<PyString, _>(py,  werr.description())
}

/// List all the files in the Repo. For now this is returned in memory.
fn repo_list(py: Python, password: String) -> PyResult<PyList> {
    let source = if password.is_empty() {
        PasswordSource::Keyring
    } else {
        PasswordSource::PassIn(password)
    };
    let repo = lib_index::Repo::build(&source)
        .map_err(|e| as_py_err(py, e))?;

    let mut entries: Vec<PyObject> = Vec::new();
    for entry in repo.list().map_err(|e| as_py_err(py, e))? {
        let decrypted = entry.map_err(|e| as_py_err(py, e))?;
        let page_content = bincode::deserialize::<PageContent>(decrypted.as_slice())
            .map_err(|e| as_py_err(py, e))?;
        let pyo = page_content_to_py(py, page_content)?;
        entries.push(pyo);
    }
    let out = PyList::new(py, entries.as_slice());
    Ok(out)
}

/// Create a Python Object from a Page Content.
fn page_content_to_py(py: Python, page_content: PageContent) -> PyResult<PyObject> {
    let one = PyDict::new(py);
    one.set_item(py, "url", page_content.url)?;
    one.set_item(py, "title", page_content.title)?;
    one.set_item(py, "body", page_content.body)?;
    Ok(one.into_object())
}
