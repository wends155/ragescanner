.PHONY: check build run test verify

check:
	cargo check

build:
	cargo build --release

run:
	cargo run

test:
	cargo test

verify:
	sh scripts/verify.sh
