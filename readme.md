This crate is work in progress, full of unsafe and mostly untested.

German/Umbra-style strings are strings which inline small strings and store a short inline prefix and a pointer for larger strings.
This crate implements three storage classes for strings stored out of line similar to Umbra:
- static (what Umbra calls persistent)
- borrowed, with borrow checking (transient)
- heap allocated (temporary)

See [this blogpost](https://cedardb.com/blog/german_strings/) and [the Umbra paper](https://db.in.tum.de/~freitag/papers/p29-neumann-cidr20.pdf) for an explanation of this string format.

This crate currently does not provide `str` based types, only `[u8]`.