project_name := dunsumday
prefix := /usr/local
exec_prefix := $(prefix)
bindir := $(exec_prefix)/bin
sysconfdir := $(prefix)/etc
datarootdir := $(prefix)/share
localstatedir := $(prefix)/var

confdir := $(sysconfdir)/$(project_name)
datadir := $(datarootdir)/$(project_name)
docdir := $(datarootdir)/doc/$(project_name)
statedir := $(localstatedir)/lib/$(project_name)

INSTALL_PROGRAM := install
INSTALL_DATA := install -m 644

.PHONY: all dev run-dev webui doc dev-doc clean distclean install uninstall \
		uninstall-config

all: doc webui
	cargo build --release

dev: webui
	cargo build

run-dev:
	cargo run -- --config dev-config.yaml

webui:
	make -C webui

doc:
	cargo doc --no-deps

dev-doc:
	cargo doc --no-deps --document-private-items

clean:
	cargo clean
	$(RM) default-config.yaml
	make -C webui clean

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
	./generate-default-config "$(DESTDIR)$(datadir)" \
		"$(DESTDIR)$(statedir)/lib" > default-config.yaml
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
