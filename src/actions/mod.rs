pub mod add;
pub mod delete;
pub mod list;

use crate::{
    actions::{
        add::add,
        list::{list, list_test},
    },
    app::SubCommand,
    config::{expand_paths, AppContext},
    file::{get_done, get_tasks, write_buf_to_file},
    prelude::*,
    task::tasks_to_string,
};
use log::{debug, info, trace};
use std::process::exit;

/// Direct the execution of the program based on the Command in the
/// Context object
pub fn handle_command(ctx: &mut AppContext, buf: &mut termcolor::Buffer) -> Result {
    expand_paths(ctx)?;
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
            SubCommand::Add { task } => {
                let new = add(task, ctx)?;
                write_buf_to_file(new.raw, ctx, true)?;
            }
            SubCommand::Addm { tasks } => {
                for task in tasks {
                    let new = add(task, ctx)?;
                    write_buf_to_file(new.raw, ctx, true)?;
                }
            }
            SubCommand::Addto => todo!(),
            SubCommand::Append { item, text } => {
                eprintln!("Appending: {:?} to task {}", text, item);
                todo!()
            }
            SubCommand::Archive => todo!(),
            SubCommand::Deduplicate => todo!(),
            SubCommand::Depri { items } => {
                eprintln!("Deprioritizing item(s): {:?}", items);
                todo!()
            }
            SubCommand::Del { item, term } => {
                if delete::delete(item, &term, ctx)? {
                    write_buf_to_file(tasks_to_string(ctx)?, ctx, false)?;
                    return Ok(());
                }
                exit(1)
            }
            SubCommand::List { terms } => {
                list_test(&terms, buf, ctx, false)?;
            }
            SubCommand::Listall { terms } => {
                get_done(ctx)?;
                list(&terms, buf, ctx, true)?;
            }
            SubCommand::Listpri { priorities } => info!("Listing priorities {:?}", priorities),
        },
        None => match &ctx.settings.default_action {
            Some(cmd) => match cmd.as_str() {
                "ls" | "list" => list(&[], buf, ctx, false)?,
                _ => panic!("Unknown command: {:?}", cmd),
            },
            None => {
                info!("No command supplied; defaulting to List");
                list(&[], buf, ctx, false)?;
            }
        },
    }
    Ok(())
}
