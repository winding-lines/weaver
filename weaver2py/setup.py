from setuptools import setup
from setuptools_rust import Binding, RustExtension

# Build the weaver python package that wraps the binary weaver2py.
setup(name='weaver',
      version='0.1.0',
      rust_extensions=[RustExtension('weaver2py',
                                     'Cargo.toml', binding=Binding.RustCPython)],
      packages=['weaver'],
      # rust extensions are not zip safe, just like C-extensions.
      zip_safe=False
)
