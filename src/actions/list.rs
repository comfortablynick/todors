use crate::{
    cli::*,
    style::{fmt_test, format_buffer},
    task::{SortBy, SortByField},
};

pub fn command_list(cmds: &mut Vec<App>) {
    const ABOUT: &str =
        "Displays all tasks that contain TERM(s) sorted by priority with line numbers.";
    let cmd = App::command("list")
        .alias("ls")
        .about(ABOUT)
        .arg(arg_terms());
    cmds.push(cmd);

    // TODO: make sure list filter actually works according to help
    // local args
    fn arg_terms() -> Arg {
        const SHORT: &str = "Term to filter task list by.";
        const LONG: &str = long!("\
Term to filter task list by.

Each task must match all TERM(s) (logical AND); to display tasks that contain any TERM (logical OR), use
\"TERM1\\|TERM2\\|...\" (with quotes), or TERM1|TERM2 (unquoted).

Hides all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM).");
        Arg::positional("terms", "TERM")
            .about(SHORT)
            .long_about(LONG)
            .multiple(true)
    }
}

pub fn command_listall(cmds: &mut Vec<App>) {
    const ABOUT: &str = "Displays all the lines in todo.txt AND done.txt that contain TERM(s) sorted by priority with line numbers.";
    let cmd = App::command("listall")
        .alias("lsa")
        .about(ABOUT)
        .arg(arg_terms());
    cmds.push(cmd);

    fn arg_terms() -> Arg {
        const SHORT: &str = "Term to filter task list by.";
        const LONG: &str = long!("\
Term to filter task list by.

Displays all the lines in todo.txt AND done.txt that contain TERM(s) sorted by priority with line numbers.

Hides all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM).  If no TERM specified, 
lists entire todo.txt AND done.txt concatenated and sorted.");
        Arg::positional("terms", "TERM")
            .about(SHORT)
            .long_about(LONG)
            .multiple(true)
    }
}

/// List tasks from todo.txt file
pub fn list<T>(
    terms: &[String],
    // buf: &mut termcolor::Buffer,
    buf: &mut T,
    ctx: &mut Context,
    list_all: bool,
) -> Result
where
    T: std::io::Write + termcolor::WriteColor,
{
    // TODO: extract filter and sort logic so I don't have to repeat it
    ctx.tasks.retain(|t| !t.is_blank());
    ctx.done.retain(|t| !t.is_blank());
    let prefilter_task_ct = ctx.tasks.len();
    let prefilter_done_ct = ctx.done.len();
    ctx.tasks.sort(&[SortBy {
        field:   SortByField::Id,
        reverse: false,
    }]);
    if list_all {
        ctx.done.sort(&[SortBy {
            field:   SortByField::Id,
            reverse: false,
        }]);
    }
    // filter based on terms
    if !terms.is_empty() {
        info!("Listing with terms: {:?}", terms);
        ctx.tasks.filter_terms(terms);
        ctx.done.filter_terms(terms);
    } else {
        info!("Listing without filter");
    }
    if list_all {
        ctx.tasks += ctx.done.clone();
    }
    // fill buffer with formatted (colored) output
    format_buffer(buf, &ctx)?;
    // write footer
    write!(
        buf,
        "--\nTODO: {} of {} tasks shown\n",
        ctx.tasks.len(),
        prefilter_task_ct,
    )?;
    if list_all {
        writeln!(
            buf,
            "DONE: {} of {} tasks shown",
            ctx.done.len(),
            prefilter_done_ct,
        )?;
    }
    Ok(())
}

pub fn list_test<T>(terms: &[String], buf: &mut T, ctx: &mut Context, list_all: bool) -> Result
where
    T: std::io::Write,
{
    // TODO: extract filter and sort logic so I don't have to repeat it
    ctx.tasks.retain(|t| !t.is_blank());
    ctx.done.retain(|t| !t.is_blank());
    let prefilter_task_ct = ctx.tasks.len();
    let prefilter_done_ct = ctx.done.len();
    ctx.tasks.sort(&[SortBy {
        field:   SortByField::Id,
        reverse: false,
    }]);
    if list_all {
        ctx.done.sort(&[SortBy {
            field:   SortByField::Id,
            reverse: false,
        }]);
    }
    // filter based on terms
    if !terms.is_empty() {
        info!("Listing with terms: {:?}", terms);
        ctx.tasks.filter_terms(terms);
        ctx.done.filter_terms(terms);
    } else {
        info!("Listing without filter");
    }
    if list_all {
        ctx.tasks += ctx.done.clone();
    }
    // fill buffer with formatted (colored) output
    fmt_test(buf, &ctx)?;
    // write footer
    writeln!(buf, "\n--")?;
    writeln!(
        buf,
        "TODO: {} of {} tasks shown",
        ctx.tasks.len(),
        prefilter_task_ct,
    )?;
    if list_all {
        writeln!(
            buf,
            "DONE: {} of {} tasks shown",
            ctx.done.len(),
            prefilter_done_ct,
        )?;
    }
    Ok(())
}
