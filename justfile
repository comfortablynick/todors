#!/usr/bin/env just --justfile
alias r := run
alias i := install
alias q := runq
alias h := help

test:
	cargo test

build:
	cargo build

fix:
	cargo fix

build-release:
	cargo build --release

# install and run
install:
	cargo install --path . -f
	todors

run:
	cargo run --release

# run with --quiet
runq:
	todors -q

help:
	todors -h

# run with verbosity 2 (-vv)
runv:
	cargo run --release -- -vv
