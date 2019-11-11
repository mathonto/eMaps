build:
	cargo build --release

lint:
	cargo clippy

lint-all:
	cargo clippy -- -W clippy::pedantic

test:
	cargo test --release

all: build lint test
