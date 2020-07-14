pub mod add;
pub mod delete;
pub mod list;

pub use crate::{
    actions::{add::add, list::list},
    cli::*,
    config::expand_paths,
    file::{get_done, get_tasks, write_buf_to_file},
    task::tasks_to_string,
};

/// Direct the execution of the program based on the Command in the
/// Context object
pub fn handle_command(ctx: &mut Context, buf: &mut termcolor::Buffer) -> Result {
    expand_paths(ctx)?;
    get_tasks(ctx).map_err(|e| err_msg(e))?;

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
            Command::Add { task } => {
                let new = add(task, ctx)?;
                write_buf_to_file(new.raw, ctx, true).map_err(|e| err_msg(e))?;
            }
            Command::Addm { tasks } => {
                for task in tasks {
                    let new = add(task, ctx)?;
                    write_buf_to_file(new.raw, ctx, true).map_err(|e| err_msg(e))?;
                }
            }
            Command::Delete { item, term } => {
                if delete::delete(item, &term, ctx)? {
                    write_buf_to_file(tasks_to_string(ctx)?, ctx, false).map_err(|e| err_msg(e))?;
                    return Ok(());
                }
                exit(1)
            }
            Command::List { terms } => {
                crate::actions::list::list_test(&terms, buf, ctx, false)?;
            }
            Command::Listall { terms } => {
                get_done(ctx).map_err(|e| err_msg(e))?;
                list(&terms, buf, ctx, true)?;
            }
            Command::Listpri { priorities } => info!("Listing priorities {:?}", priorities),
            Command::Addto => info!("Adding to..."),
            Command::Append { item, text } => info!("Appending: {:?} to task {}", text, item),
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
