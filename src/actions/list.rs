use crate::{
    cli::*,
    style::format_buffer,
    task::{SortBy, SortByField},
};
use std::io::Write;

/// List tasks from todo.txt file
pub fn list(
    terms: &[String],
    buf: &mut termcolor::Buffer,
    ctx: &mut Context,
    list_all: bool,
) -> Result {
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
