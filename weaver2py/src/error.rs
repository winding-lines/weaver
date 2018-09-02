use cpython::{PyErr, PyString, Python};
use std::error::Error;

/// Translate any Error into a Py Error.
pub(crate) fn as_py_err<T:Error>(py: Python, werr: T) -> PyErr {
    PyErr::new::<PyString, _>(py,  werr.description())
}