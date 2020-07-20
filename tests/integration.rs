#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
/// Run both todors and todo.sh and compare output
fn compare_bin_output() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::*;
    let todors_bin = shellexpand::env("$HOME/git/todors/target/release/todors")?;
    let todors = Command::new(todors_bin.as_ref()).args(&["-q"]).output()?;
    let cfg_file = shellexpand::env("$HOME/git/todors/tests/todo.cfg")?;
    let todo_sh = Command::new("todo.sh")
        .args(&["-d", cfg_file.as_ref(), "ls"])
        .env("TODOTXT_SORT_COMMAND", "sort")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&todors.stdout)?,
        std::str::from_utf8(&todo_sh.stdout)?
    );
    Ok(())
}

#[test]
/// Run both todors and todo.sh and compare output
fn compare_bin_test_output() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::*;
    let todors_bin = shellexpand::env("$HOME/git/todors/target/release/todors")?;
    let todors = Command::new(todors_bin.as_ref())
        .args(&["-q", "ls"])
        .output()?;
    let cfg_file = shellexpand::env("$HOME/git/todors/tests/todo.cfg")?;
    let todo_sh = Command::new("todo.sh")
        .args(&["-d", cfg_file.as_ref(), "ls"])
        .env("TODOTXT_SORT_COMMAND", "sort")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&todors.stdout)?,
        std::str::from_utf8(&todo_sh.stdout)?
    );
    Ok(())
}

#[test]
/// Run both todors and todo.sh and compare output
fn compare_bin_plain_output() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::*;
    let todors_bin = shellexpand::env("$HOME/git/todors/target/release/todors")?;
    let todors = Command::new(todors_bin.as_ref())
        .args(&["-p", "ls"])
        .output()?;
    let cfg_file = shellexpand::env("$HOME/git/todors/tests/todo.cfg")?;
    let todo_sh = Command::new("todo.sh")
        .args(&["-p", "-d", cfg_file.as_ref(), "ls"])
        .env("TODOTXT_SORT_COMMAND", "sort")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&todors.stdout)?,
        std::str::from_utf8(&todo_sh.stdout)?
    );
    Ok(())
}
