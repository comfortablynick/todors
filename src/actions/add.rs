use crate::{cli::*, task::Task};
use chrono::Utc;
use std::io::{self, Write};

/// Create task from raw input. Print confirmation and return to caller.
pub fn add(task: String, ctx: &mut Context) -> Result<Task> {
    let mut task = task;
    if task == "" {
        io::stdout().write_all(b"Add: ").unwrap();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut task).unwrap();
    }
    ctx.task_ct += 1;
    if ctx.opts.date_on_add {
        let dt = Utc::today().format("%Y-%m-%d");
        task = format!("{} {}", dt, task);
    }
    let new = Task::new(ctx.task_ct, &task);
    println!("{}", new);
    println!("TODO: {} added.", new.id);
    Ok(new)
}
