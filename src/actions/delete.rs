use crate::{config::AppContext, prelude::*, task::Task, util};
use log::info;
use regex::Regex;

/// Delete task by line number, or delete term from task
pub fn delete(item: usize, term: &Option<String>, ctx: &mut AppContext) -> Result<bool> {
    if let Some(t) = term {
        let re = Regex::new(t)?;

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
                let new = Task::new(task.id, result.as_ref()).normalize_whitespace();
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
