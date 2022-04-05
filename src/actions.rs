//! # Interact and opionally edit the todo.txt file.
pub mod add;
pub mod delete;
pub mod list;

use crate::{
    actions::{add::add, list::list},
    app::Commands,
    config::AppContext,
    file::{get_done, get_tasks, write_buf_to_file},
    prelude::*,
    task::tasks_to_string,
};

/// Direct the execution of the program based on the Command in the
/// Context object
pub fn handle_command<W>(ctx: &mut AppContext, buf: &mut W) -> Result
where
    W: std::io::Write + termcolor::WriteColor,
{
    ctx.expand_paths()?;
    get_tasks(ctx)?;

    // Debug print of all settings
    debug!("{:#?}", ctx.opts);
    debug!("{:#?}", ctx.settings);
    debug!("Todo file: {:?}", ctx.todo_file);
    debug!("Done file: {:?}", ctx.done_file);
    debug!("Rept file: {:?}", ctx.report_file);
    trace!("{:#?}", ctx.styles);
    trace!("{:#?}", ctx.tasks);

    match ctx.opts.cmd.clone() {
        Some(command) => match command {
            Commands::Add { task } => {
                let new = add(task, ctx)?;
                write_buf_to_file(new.raw, &ctx.todo_file, true)?;
            }
            Commands::Addm { tasks } => {
                for task in tasks {
                    let new = add(task, ctx)?;
                    write_buf_to_file(new.raw, &ctx.todo_file, true)?;
                }
            }
            Commands::Addto => todo!(),
            Commands::Append { item, text } => {
                eprintln!("Appending: {:?} to task {}", text, item);
                todo!()
            }
            Commands::Archive => todo!(),
            Commands::Complete { shell } => shell.generate(),
            Commands::Deduplicate => todo!(),
            Commands::Depri { items } => {
                eprintln!("Deprioritizing item(s): {:?}", items);
                todo!()
            }
            Commands::Del { item, term } => {
                if delete::delete(item, &term, ctx)? {
                    write_buf_to_file(tasks_to_string(ctx)?, &ctx.todo_file, false)?;
                    return Ok(());
                }
                std::process::exit(1)
            }
            Commands::List { terms } => {
                list(&terms, buf, ctx, false)?;
            }
            Commands::Listall { terms } => {
                get_done(ctx)?;
                list(&terms, buf, ctx, true)?;
            }
            Commands::Listpri { priorities } => info!("Listing priorities {:?}", priorities),
        },
        None => match &ctx.settings.default_action {
            Some(cmd) => match cmd.as_str() {
                "ls" | "list" => list(&[], buf, ctx, false)?,
                _ => bail!("Unknown command: {:?}", cmd),
            },
            None => {
                info!("No command supplied; defaulting to List");
                list(&[], buf, ctx, false)?;
            }
        },
    }
    Ok(())
}
