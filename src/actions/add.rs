use crate::{cli::*, errors::Result, task::Task};
use chrono::Utc;
use std::io::{self, Write};

pub fn command_add(cmds: &mut Vec<App>) {
    const ABOUT: &str = "Add a line to your todo.txt file.";
    cmds.push(App::command("add").alias("a").about(ABOUT).arg(arg_task()));

    // local args
    fn arg_task() -> Arg {
        const SHORT: &str = "Todo item";
        const LONG: &str = long!(
            "\
THING I NEED TO DO +project @context

Adds THING I NEED TO DO to your todo.txt file on its own line.
Project and context notation optional.
Quotes optional."
        );
        Arg::positional("task", "TASK")
            .about(SHORT)
            .long_about(LONG)
            .required(true)
    }
}

pub fn command_addm(cmds: &mut Vec<App>) {
    const ABOUT: &str = "Add multiple lines to todo.txt file";
    cmds.push(App::command("addm").about(ABOUT).arg(arg_tasks()));

    fn arg_tasks() -> Arg {
        const SHORT: &str = "Todo items (line separated)";
        const LONG: &str = long!(
            "
\"FIRST THING I NEED TO DO +project1 @context
SECOND THING I NEED TO DO +project2 @context\"

Adds FIRST THING I NEED TO DO on its own line and SECOND THING I NEED TO DO on its own line.
Project and context notation optional.
Quotes required."
        );
        Arg::positional("tasks", "TASKS")
            .about(SHORT)
            .long_about(LONG)
            .value_delimiter("\n")
            .required(true)
    }
}

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
