use criterion::{criterion_group, criterion_main, Criterion};
use duct::cmd;

const BIN: &str = env!("CARGO_BIN_EXE_todors");
const CFG: &str = "tests/todo.toml";

fn list() {
    cmd!(BIN, "ls").env("TODORS_CFG_FILE", CFG).read().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("list", |b| b.iter(|| list()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
