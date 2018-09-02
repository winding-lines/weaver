use bincode;
use cpython::{PyDict, PyErr, PyList, PyObject, PyResult, PyString, Python, PythonObject};
use error::as_py_err;
use lib_goo::config::db::PasswordSource;
use lib_goo::entities::PageContent;
use lib_index::repo::EncryptedRepo;

/// List all the files in the Repo. For now this is returned in memory.
pub fn repo_list(py: Python, password: String) -> PyResult<PyList> {
    let source = if password.is_empty() {
        PasswordSource::Keyring
    } else {
        PasswordSource::PassIn(password)
    };
    let repo = EncryptedRepo::build(&source).map_err(|e| as_py_err(py, e))?;

    let mut entries: Vec<PyObject> = Vec::new();
    for entry in repo
        .list(&PageContent::collection_name().into())
        .map_err(|e| as_py_err(py, e))?
    {
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
