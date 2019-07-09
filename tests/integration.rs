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
/// Use library functions to compare todors and todo.sh
fn compare_lib_output() -> Result<(), Box<dyn std::error::Error>> {
    // Get output from todors `run()`
    use termcolor::{BufferWriter, ColorChoice};
    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();
    todors::run(&["todors".to_string(), "list".to_string()], &mut buf)?;
    let todors = std::str::from_utf8(buf.as_slice())?;

    // Get output from todo.sh
    let todo_sh_output = todors::get_todo_sh_output(None, Some("sort"))?;
    let todo_sh = std::str::from_utf8(&todo_sh_output.stdout)?;

    assert_eq!(todo_sh, todors);
    Ok(())
}

#[test]
/// Run both todors and todo.sh and compare output
fn compare_bin_output() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::*;
    let todors = Command::new("todors").arg("ls").output()?;

    let todo_sh = Command::new("todo.sh")
        .arg("ls")
        .env("TODOTXT_SORT_COMMAND", "sort")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&todors.stdout)?,
        std::str::from_utf8(&todo_sh.stdout)?
    );
    Ok(())
}
