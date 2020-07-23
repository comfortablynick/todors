use duct::cmd;
use pretty_assertions::assert_eq;
use todors::prelude::*;

const BIN: &str = env!("CARGO_BIN_EXE_todors");
const CFG: &str = "tests/todo.toml";
const TODO_BIN: &str = "todo.sh";
const TODO_CFG: &str = "tests/todo.cfg";

#[test]
/// Run both todors and todo.sh and compare output
fn compare_bin_defaults() -> Result {
    let todors = cmd!(BIN).env("TODORS_CFG_FILE", CFG).read()?;
    let todo_sh = cmd!(TODO_BIN)
        .env("TODOTXT_CFG_FILE", TODO_CFG)
        .env("TODOTXT_SORT_COMMAND", "sort")
        .read()?;
    assert_eq!(todo_sh, todors);
    Ok(())
}

#[test]
/// Compare `ls` command
fn compare_bin_ls() -> Result {
    let todors = cmd!(BIN, "ls").env("TODORS_CFG_FILE", CFG).read()?;
    let todo_sh = cmd!(TODO_BIN, "ls")
        .env("TODOTXT_CFG_FILE", TODO_CFG)
        .env("TODOTXT_SORT_COMMAND", "sort")
        .read()?;
    assert_eq!(todo_sh, todors);
    Ok(())
}

#[test]
/// Compare `ls` command with plain output
fn compare_bin_ls_plain() -> Result {
    let todors = cmd!(BIN, "-p", "ls").env("TODORS_CFG_FILE", CFG).read()?;
    let todo_sh = cmd!(TODO_BIN, "-p", "ls")
        .env("TODOTXT_CFG_FILE", TODO_CFG)
        .env("TODOTXT_SORT_COMMAND", "sort")
        .read()?;
    assert_eq!(todo_sh, todors);
    Ok(())
}

#[test]
/// Compare `lsa` command
fn compare_bin_lsa() -> Result {
    let todo_sh = cmd!(TODO_BIN, "lsa")
        .env("TODOTXT_CFG_FILE", TODO_CFG)
        // .env("TODOTXT_SORT_COMMAND", "sort")
        .read()?;
    let todors = cmd!(BIN, "lsa").env("TODORS_CFG_FILE", CFG).read()?;
    assert_eq!(todo_sh, todors);
    Ok(())
}

#[test]
/// Compare `lsa` command
fn compare_bin_lsa_plain() -> Result {
    let todo_sh = cmd!(TODO_BIN, "-p", "lsa")
        .env("TODOTXT_CFG_FILE", TODO_CFG)
        // .env("TODOTXT_SORT_COMMAND", "sort")
        .read()?;
    let todors = cmd!(BIN, "-p", "lsa").env("TODORS_CFG_FILE", CFG).read()?;
    assert_eq!(todo_sh, todors);
    Ok(())
}
