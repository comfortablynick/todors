use crate::{
    config::Context,
    task::{Task, Tasks},
};
use anyhow::{anyhow, Context as ErrContext};
use log::info;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

type Result<T = ()> = anyhow::Result<T>;

// TODO: combine get_tasks and get_done since they are 90% the same
/// Load todo.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
pub fn get_tasks(ctx: &mut Context) -> Result {
    // let mut task_ct = 0;
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&ctx.todo_file)
        .and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf).map(|_| buf)
        })
        .and_then(|b| {
            ctx.task_ct = 0;
            ctx.tasks = Tasks(
                b.lines()
                    .map(|l| {
                        ctx.task_ct += 1;
                        Task::new(ctx.task_ct, l)
                    })
                    .collect(),
            );
            Ok(())
        })
        .map_err(|e| anyhow!(e))
}

/// Load done.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
pub fn get_done(ctx: &mut Context) -> Result {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&ctx.done_file)
        .and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf).map(|_| buf)
        })
        .and_then(|b| {
            ctx.task_ct = 0;
            ctx.done = Tasks(
                b.lines()
                    .map(|l| {
                        ctx.task_ct += 1;
                        Task::new(ctx.task_ct, l)
                    })
                    .collect(),
            );
            Ok(())
        })
        .map_err(|e| anyhow!(e))
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
        .with_context(|| format!("file: {:?}", ctx.todo_file))?;
    write!(file, "{}", buf.into())?;
    if append {
        writeln!(file)?;
    }
    let action = if append { "Appended" } else { "Wrote" };
    info!("{} tasks to file {:?}", action, ctx.todo_file);
    Ok(())
}
