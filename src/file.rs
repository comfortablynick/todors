use crate::{
    config::AppContext,
    prelude::*,
    task::{Task, Tasks},
};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

// TODO: combine get_tasks and get_done since they are 90% the same
/// Load todo.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
pub fn get_tasks(ctx: &mut AppContext) -> Result {
    read_file_to_string(&ctx.todo_file).map(|b| {
        ctx.task_ct = 0;
        ctx.tasks = Tasks(
            b.lines()
                .map(|l| {
                    ctx.task_ct += 1;
                    Task::new(ctx.task_ct, l)
                })
                .collect(),
        );
    })
}

/// Load done.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
pub fn get_done(ctx: &mut AppContext) -> Result {
    read_file_to_string(&ctx.done_file).map(|b| {
        ctx.task_ct = 0;
        ctx.done = Tasks(
            b.lines()
                .map(|l| {
                    ctx.task_ct += 1;
                    Task::new(ctx.task_ct, l)
                })
                .collect(),
        );
    })
}

/// Write tasks to file
pub fn write_buf_to_file<T, P>(buf: T, file_path: P, append: bool) -> Result
where
    T: Into<String>,
    P: AsRef<Path> + std::fmt::Debug,
{
    OpenOptions::new()
        .write(true)
        .truncate(!append)
        .append(append)
        .open(&file_path)
        .and_then(|mut file| write!(file, "{}{}", buf.into(), if append { "\n" } else { "" }))?;
    info!(
        "{} tasks to file {:?}",
        if append { "Appended" } else { "Wrote" },
        file_path
    );
    Ok(())
}

/// Read file to string
pub fn read_file_to_string<P>(file_path: P) -> Result<String>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    OpenOptions::new()
        .read(true)
        .open(&file_path)
        .and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf).map(|_| buf)
        })
        .with_context(|| format!("reading file {:?} to string", file_path))
}
