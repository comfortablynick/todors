#!/usr/bin/env just --justfile
alias r := run
alias i := install
alias q := runq

test:
	cargo test

build:
	cargo build

fix:
	cargo fix

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
