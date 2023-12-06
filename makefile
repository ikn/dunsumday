project_name := dunsumday_cli
prefix := /usr/local
datarootdir := $(prefix)/share
exec_prefix := $(prefix)
bindir := $(exec_prefix)/bin
docdir := $(datarootdir)/doc/$(project_name)

INSTALL_PROGRAM := install
INSTALL_DATA := install -m 644

.PHONY: all dev clean distclean doc dev-doc install uninstall

all: doc
	cargo build --release

dev:
	cargo build

clean:
	cargo clean

distclean: clean

doc:
	cargo doc --no-deps

dev-doc:
	cargo doc --no-deps --document-private-items

install:
	@ # executable
	mkdir -p "$(DESTDIR)$(bindir)/"
	$(INSTALL_PROGRAM) "target/release/$(project_name)" \
	    "$(DESTDIR)$(bindir)/$(project_name)"
	@ # doc
	mkdir -p "$(DESTDIR)$(docdir)/"
	$(INSTALL_DATA) README.md "$(DESTDIR)$(docdir)/"
	$(INSTALL_DATA) "target/doc/$(project_name)" "$(DESTDIR)$(docdir)/lib/"

uninstall:
	@ # executable
	$(RM) "$(DESTDIR)$(bindir)/$(project_name)"
	@ # readme
	$(RM) -r "$(DESTDIR)$(docdir)/"
