.PHONY: build debug clean py

build:
	cargo build --release

debug:
	cargo build

clean:
	rm -rf target demo.db

py: build
	cd python && python3 demo.py