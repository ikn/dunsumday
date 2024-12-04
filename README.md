dunsumday 0.0.0-next.

Track completion of regular tasks.

# License

Distributed under the terms of the
[GNU General Public License, version 3](http://www.gnu.org/licenses/gpl-3.0.txt).

# Installation

Build dependencies:
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Make](https://www.gnu.org/software/make/)

Run `make`, `make install`.  The makefile respects the `prefix`, `DESTDIR`, etc.
arguments.  To uninstall, run `make uninstall`.  `make clean` and
`make distclean` are also supported.

# Development

- `make dev` to build a development build
- `make dev-doc` to build library docs
