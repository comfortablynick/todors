#!/usr/bin/env just --justfile
package_name    := `sed -En 's/name[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
package_version := `sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
build_type      := env_var("CARGO_PROFILE")

alias r := run
alias b := build
alias i := install
alias h := help
alias lh := longhelp
alias q := runq
alias file-run := run
alias file-build := build

_default:
    @just --choose

# automatically build on each change
autobuild:
    cargo watch -x build

# build binary
build *FLAGS:
    cargo build {{FLAGS}}

# benchmark
bench:
    RUST_LOG=off cargo bench

# rebuild docs
doc:
    cargo makedocs -d --root

# rebuild docs and start simple static server
docs +PORT='40000':
    cargo makedocs -d --root && http target/doc -p {{PORT}}

# start server for docs and update upon changes
docslive:
    light-server -c .lightrc

# rebuild docs and start simple static server that watches for changes (in parallel)
docw +PORT='40000':
    parallel --lb ::: "cargo watch -x 'makedocs -d --root'" "http target/doc -p {{PORT}}"

# install binary to ~/.cargo/bin
install:
    cargo install --path . -f

# build release binary and run
run +args='':
    cargo run -- {{args}}

# run with --quiet
runq:
    cargo run -- -q

# run with -v
runv:
    cargo run -- -v

# run with -vv
runvv:
    cargo run -- -vv

# clap short help
help:
    cargo run -- -h

# clap long help
longhelp:
    cargo run -- --help

# run binary
rb *args:
    ./target/{{build_type}}/{{package_name}} {{args}}

test:
    cargo test

lint:
    cargo clippy

fix:
    cargo fix

clean:
    cargo clean
    find . -type f -name "*.orig" -exec rm {} \;
    find . -type f -name "*.bk" -exec rm {} \;
    find . -type f -name ".*~" -exec rm {} \;

version:
    @echo {{package_name}} v{{package_version}}  \({{build_type}}\)
