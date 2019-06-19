#!/usr/bin/env just --justfile

test:
	cargo test

build:
	cargo build

build-release:
	cargo build --release

install:
	cargo install --path . -f
	todors

run:
	cargo run --release

# run with --quiet
runq:
	todors -q
