all: build

build:
	@cargo build

clean:
	@cargo clean

install: build
	@install -sC target/debug/lxp-bridge /usr/local/bin

uninstall:
	@rm -f /usr/local/bin/lxp-bridge
