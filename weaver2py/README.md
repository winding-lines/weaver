# Weaver2py

This subproject provides a [binary extension](https://docs.python.org/2/extending/extending.html) to cpython 
in order to read the encrypted file repo. 

## Installation

In order to compile and install this extension:

1. Enable your Python virtual environment in your main project 
2. Install dependencies: `pip install setuptools_rust`
3. Run `python setup.py install` in this folder.

Note: setup.py needs to pass in special 
[linker options](https://github.com/dgrunwald/rust-cpython/issues/87)