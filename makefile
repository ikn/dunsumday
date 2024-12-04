project_name := dunsumday_cli
prefix := /usr/local
datarootdir := $(prefix)/share
exec_prefix := $(prefix)
bindir := $(exec_prefix)/bin
datadir := $(datarootdir)
sysconfdir := $(prefix)/etc
project_conf_dir := $(sysconfdir)/$(project_name)
docdir := $(datarootdir)/doc/$(project_name)

INSTALL_PROGRAM := install
INSTALL_DATA := install -m 644

.PHONY: all webui dev clean distclean doc dev-doc install uninstall

webui:
	make -C webui

all: doc webui
	cargo build --release

dev: webui
	cargo build

clean:
	make -C webui clean
	cargo clean

distclean: clean

doc:
	cargo doc --no-deps

dev-doc:
	cargo doc --no-deps --document-private-items

install:
	@ # webserver
	mkdir -p "$(DESTDIR)$(bindir)/"
	$(INSTALL_PROGRAM) -T "target/release/$(project_name)_webserver" \
	    "$(DESTDIR)$(bindir)/$(project_name)-webserver"
	@ # runtime data
	mkdir -p "$(DESTDIR)$(datarootdir)/"
	$(INSTALL_DATA) -T lib/runtime-data "$(DESTDIR)$(datarootdir)/lib"
	$(INSTALL_DATA) -T webserver/runtime-data \
		"$(DESTDIR)$(datarootdir)/webserver"
	@ # config
	mkdir -p "$(DESTDIR)$(project_conf_dir)/"
	$(INSTALL_DATA) -T default-config.yaml \
		"$(DESTDIR)$(project_conf_dir)/config.yaml"
	@ # doc
	mkdir -p "$(DESTDIR)$(docdir)/"
	$(INSTALL_DATA) -t "$(DESTDIR)$(docdir)/" README.md

uninstall:
	@ # webserver
	$(RM) "$(DESTDIR)$(bindir)/$(project_name)-webserver"
	@ # runtime data
	$(RM) -r "$(DESTDIR)$(datarootdir)/"
	@ # readme
	$(RM) -r "$(DESTDIR)$(docdir)/"
