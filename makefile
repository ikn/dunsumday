project_name := dunsumday_cli
prefix := /usr/local
datarootdir := $(prefix)/share
exec_prefix := $(prefix)
bindir := $(exec_prefix)/bin
docdir := $(datarootdir)/doc/$(project_name)

INSTALL_PROGRAM := install
INSTALL_DATA := install -m 644

.PHONY: all dev clean distclean install uninstall

all:
	cargo build --release

clean:
	cargo clean

distclean: clean

install:
	@ # executable
	mkdir -p "$(DESTDIR)$(bindir)/"
	$(INSTALL_PROGRAM) "target/release/$(project_name)" \
	    "$(DESTDIR)$(bindir)/$(project_name)"
	@ # readme
	mkdir -p "$(DESTDIR)$(docdir)/"
	$(INSTALL_DATA) README.md "$(DESTDIR)$(docdir)/"

uninstall:
	@ # executable
	$(RM) "$(DESTDIR)$(bindir)/$(project_name)"
	@ # readme
	$(RM) -r "$(DESTDIR)$(docdir)/"
