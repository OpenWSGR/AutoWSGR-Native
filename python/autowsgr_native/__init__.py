"""AutoWSGR native (Rust) backends.

This package holds functionality that is inconvenient to implement in the
pure-Python main AutoWSGR package, such as image recognition and device
control via scrcpy/adb. Public APIs live in feature subpackages such as
:mod:`autowsgr_native.recognition`; the root package itself intentionally
exports nothing, so that additional backends can be added as sibling
subpackages.
"""
