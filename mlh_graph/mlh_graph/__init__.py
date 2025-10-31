# import the contents of the Rust library into the Python extension
from .mlh_graph import *
from .mlh_graph import __all__

# optional: include the documentation from the Rust module
from .mlh_graph import __doc__  # noqa: F401

# __all__ = __all__ + ["ExtensionClass"]
#
#
# class ExtensionClass:
#     def __init__(self, value: int) -> None:
#         self.value = value
