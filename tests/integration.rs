#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
/// Test todo_txt library string -> task
fn str_to_task() {
    use std::str::FromStr;
    let line = "x (C) 2019-12-18 Get new +pricing for +item @work due:2019-12-31";
    let task = todo_txt::Task::from_str(line).expect("error parsing task");
    assert_eq!(task.subject, "Get new +pricing for +item @work");
    assert_eq!(task.priority, 2);
    assert_eq!(task.contexts, vec!("work".to_owned()));
    assert_eq!(task.projects, vec!("item".to_owned(), "pricing".to_owned()));
    assert_eq!(task.finish_date, None);
    assert_eq!(task.due_date, Some(todo_txt::Date::from_ymd(2019, 12, 31)));
    assert_eq!(task.threshold_date, None);
}

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
