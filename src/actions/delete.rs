use crate::{cli::*, task::Task, util};
use regex::Regex;

pub fn command_del(cmds: &mut Vec<App>) {
    const SHORT: &str = "Deletes the task on line of todo.txt";
    const LONG: &str = long!(
        "\
Deletes the task on line of todo.txt.
If TERM specified, deletes only TERM from the task"
    );
    cmds.push(
        App::command("del")
            .alias("rm")
            .about(SHORT)
            .long_about(LONG)
            .args(&[arg_item(), arg_term()]),
    );

    fn arg_item() -> Arg {
        const SHORT: &str = "Line number of task to delete";
        Arg::positional("item", "ITEM").help(SHORT).required(true)
    }

    fn arg_term() -> Arg {
        const SHORT: &str = "Optional term to remove from item";
        const LONG: &str = long!(
            "\
Optional term to remove from item.

If TERM is specified, only the TERM is removed from ITEM.

If no TERM is specified, the entire ITEM is deleted."
        );
        Arg::positional("term", "TERM").help(SHORT).long_help(LONG)
    }
}

#[allow(clippy::needless_range_loop)]
/// Delete task by line number, or delete word from task
pub fn delete(item: usize, term: &Option<String>, ctx: &mut Context) -> Result<bool> {
    if let Some(t) = term {
        let re = Regex::new(t).unwrap();

        for i in 0..ctx.tasks.len() {
            let task = &ctx.tasks.0[i];
            if task.id == item {
                info!("Removing {:?} from {}", t, task);
                println!("{} {}", task.id, task.raw);
                if !re.is_match(&task.raw) {
                    info!("'{}' not found in task.", t);
                    println!("TODO: '{}' not found; no removal done.", t);
                    return Ok(false);
                }
                let result = re.replace_all(&task.raw, "");
                let new = Task::new(task.id, &result.into_owned()).normalize_whitespace();
                info!("Task after editing: {}", new.raw);
                println!("TODO: Removed '{}' from task.", t);
                println!("{}", new);
                ctx.tasks.0[i] = new;
            }
        }
        return Ok(true);
    }
    for i in 0..ctx.tasks.len() {
        let t = &ctx.tasks.0[i];
        if t.id == item {
            info!("Removing '{}' at index {}", t, i);
            if util::ask_user_yes_no(&format!("Delete '{}'?  (y/n)\n", t.raw,))? {
                let msg = format!("{}\nTODO: {} deleted.", t, t.id);
                ctx.tasks.0[i] = t.clear();
                println!("{}", msg);
                return Ok(true);
            }
            println!("TODO: No tasks were deleted.");
            return Ok(true);
        }
    }
    println!("TODO: No task {}.", item);
    Ok(false)
}
