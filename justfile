#!/usr/bin/env just --justfile
alias r := run
alias b := build
alias i := install
alias h := help
alias lh := longhelp
alias t := todors
alias q := runq

dev := '0'

# build release binary
build:
	cargo build --release

# build release binary ONLY during dev
# otherwise install
install:
	#!/usr/bin/env bash
	if [[ {{dev}} -eq "1" ]]; then
		cargo run --release
	else
		cargo install --path . -f
	fi #

# build release binary and run
run:
	cargo run --release -- -v

# run with --quiet
runq:
	./target/release/todors -q

# clap short help
help:
    cargo run --release -- -h

# clap long help
longhelp:
    cargo run --release -- --help

# run with verbosity (INFO)
runv:
	RUST_LOG=info cargo run

# run with verbosity (DEBUG)
runvv:
	RUST_LOG=debug cargo run

# run release binary
todors +args='':
	./target/release/todors {{args}}

test:
	cargo test

fix:
	cargo fix
