project_name := dunsumday
prefix := /usr/local
exec_prefix := $(prefix)
conf_prefix := $(prefix)/etc

bindir := $(exec_prefix)/bin
confdir := $(conf_prefix)/$(project_name)
datarootdir := $(prefix)/share
datadir := $(datarootdir)/$(project_name)
docdir := $(datarootdir)/doc/$(project_name)

INSTALL_PROGRAM := install
INSTALL_DATA := install -m 644

.PHONY: all dev webui doc dev-doc clean distclean install uninstall \
        uninstall-config

all: doc webui
	cargo build --release

dev: webui
	cargo build

webui:
	make -C webui

doc:
	cargo doc --no-deps

dev-doc:
	cargo doc --no-deps --document-private-items

clean:
	make -C webui clean
	cargo clean

distclean: clean

install:
	@ # webserver
	mkdir -p "$(DESTDIR)$(bindir)/"
	$(INSTALL_PROGRAM) -T "target/release/$(project_name)_webserver" \
	    "$(DESTDIR)$(bindir)/$(project_name)-webserver"
	@ # runtime data
	mkdir -p "$(DESTDIR)$(datadir)/"
	cp -rT lib/runtime-data "$(DESTDIR)$(datadir)/lib"
	cp -rTL webserver/runtime-data "$(DESTDIR)$(datadir)/webserver"
	@ # config
	mkdir -p "$(DESTDIR)$(confdir)/"
	$(INSTALL_DATA) -T default-config.yaml "$(DESTDIR)$(confdir)/config.yaml"
	@ # doc
	mkdir -p "$(DESTDIR)$(docdir)/"
	$(INSTALL_DATA) -t "$(DESTDIR)$(docdir)/" README.md

uninstall:
	@ # webserver
	$(RM) "$(DESTDIR)$(bindir)/$(project_name)-webserver"
	@ # runtime data
	$(RM) -r "$(DESTDIR)$(datadir)/"
	@ # readme
	$(RM) -r "$(DESTDIR)$(docdir)/"

uninstall-config:
	$(RM) -r "$(DESTDIR)$(confdir)/"
