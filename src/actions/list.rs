use crate::{config::AppContext, prelude::*, style::format_buffer};
use log::{debug, info};

/// List tasks from todo.txt and done.txt files
pub fn list<T>(terms: &[String], buf: &mut T, ctx: &mut AppContext, list_all: bool) -> Result
where
    T: std::io::Write + termcolor::WriteColor,
{
    // TODO: extract filter and sort logic so I don't have to repeat it
    let prefilter_task_ct = ctx.tasks.len();
    debug!("Prefilter task ct: {}", prefilter_task_ct);
    let prefilter_done_ct = ctx.done.len();
    ctx.tasks.retain(|t| !t.is_blank());
    ctx.done.retain(|t| !t.is_blank());
    let blank_tasks = prefilter_task_ct - ctx.tasks.len();
    if list_all {
        debug!("Prefilter done ct: {}", prefilter_done_ct);
        // ctx.done.sort(&[SortBy {
        //     field:   SortByField::Id,
        //     reverse: false,
        // }]);
        ctx.done.iter_mut().for_each(|t| t.id = 0)
    }
    // filter based on terms
    if !terms.is_empty() {
        info!("Listing with terms: {:?}", terms);
        ctx.tasks.filter_terms_regex(terms)?;
        ctx.done.filter_terms_regex(terms)?;
    } else {
        info!("Listing without filter");
    }
    let postfilter_task_ct = ctx.tasks.len();
    let postfilter_done_ct = ctx.done.len();
    if list_all {
        ctx.tasks += ctx.done.clone();
    }
    // ctx.tasks.sort(&[
    //     SortBy {
    //         field:   SortByField::Priority,
    //         reverse: false,
    //     },
    // ]);
    ctx.tasks.sort(&ctx.opts.sort_by);
    // fill buffer with formatted (colored) output
    format_buffer(buf, &ctx)?;
    // write footer
    if list_all {
        writeln!(
            buf,
            "--\nTODO: {} of {} tasks shown",
            postfilter_task_ct, prefilter_task_ct,
        )?;
        writeln!(
            buf,
            "DONE: {} of {} tasks shown",
            postfilter_done_ct, prefilter_done_ct,
        )?;
        writeln!(
            buf,
            "total {} of {} tasks shown",
            postfilter_task_ct + postfilter_done_ct,
            prefilter_task_ct + prefilter_done_ct,
        )?;
    } else {
        writeln!(
            buf,
            "--\nTODO: {} of {} tasks shown",
            postfilter_task_ct,
            prefilter_task_ct - blank_tasks,
        )?;
    }
    Ok(())
}
