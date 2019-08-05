use crate::{
    cli::*,
    task::{Task, Tasks},
};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

/// Load todo.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
pub fn get_tasks(ctx: &mut Context) -> Result {
    let mut todo_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&ctx.todo_file)
        .context(format!("file: {:?}", ctx.todo_file))
        .map_err(|_| ErrorType::FileOpenError(format!("{:?}", ctx.todo_file)))?;
    // create string buffer and read file into it
    let mut buf = String::new();
    todo_file
        .read_to_string(&mut buf)
        .map_err(|_| ErrorType::FileReadError)?;
    let mut task_ct = 0;
    ctx.tasks = Tasks(
        buf.lines()
            .map(|l| {
                task_ct += 1;
                Task::new(task_ct, l)
            })
            .collect(),
    );
    ctx.task_ct = task_ct;
    Ok(())
}

/// Load done.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
pub fn get_done(ctx: &mut Context) -> Result {
    let mut done_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&ctx.done_file)
        .context(format!("file: {:?}", ctx.done_file))
        .map_err(|_| ErrorType::FileOpenError(format!("{:?}", ctx.done_file)))?;
    // create string buffer and read file into it
    let mut buf = String::new();
    done_file.read_to_string(&mut buf)?;
    let mut task_ct = 0;
    ctx.done = Tasks(
        buf.lines()
            .map(|l| {
                task_ct += 1;
                Task::new(0, l)
            })
            .collect(),
    );
    ctx.done_ct += task_ct;
    Ok(())
}

/// Write tasks to file
pub fn write_buf_to_file<T>(buf: T, ctx: &Context, append: bool) -> Result
where
    T: Into<String>,
{
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(!append)
        .append(append)
        .open(&ctx.todo_file)
        .context(format!("file: {:?}", ctx.todo_file))
        .map_err(|_| ErrorType::FileOpenError(format!("{:?}", ctx.todo_file)))?;
    write!(file, "{}", buf.into()).map_err(|_| ErrorType::FileWriteError)?;
    if append {
        writeln!(file).map_err(|_| ErrorType::FileWriteError)?; // Add newline at end
    }
    let action = if append { "Appended" } else { "Wrote" };
    info!("{} tasks to file {:?}", action, ctx.todo_file);
    Ok(())
}
