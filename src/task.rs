//! Module containing Task objects and the Tasks container

use crate::cli::*;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    ops::AddAssign,
};

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Tasks(pub Vec<Task>);

impl Display for Tasks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for t in &self.0 {
            writeln!(f, "{}", t)?;
        }
        Ok(())
    }
}

impl IntoIterator for Tasks {
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    type Item = Task;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl AddAssign for Tasks {
    fn add_assign(&mut self, other: Self) {
        for i in other {
            self.0.push(i);
        }
    }
}

#[allow(dead_code)]
impl Tasks {
    /// Create new Tasks object
    pub fn new() -> Self {
        let new = Vec::new();
        Self(new)
    }

    /// Add a new Task to Tasks collection
    pub fn add_task(mut self, new_task: Task) -> Self {
        self.0.push(new_task);
        self
    }

    /// Remove Task by id
    pub fn remove_by_id(&mut self, id: usize) -> &Self {
        for i in 0..self.0.len() {
            if self.0[i].id == id {
                self.0[i] = Task::new(id, "");
            }
        }
        self
    }

    /// Returns the number of elements in the slice
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if container is empty
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Retain based on closure
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Task) -> bool,
    {
        self.0.retain(|x| f(x));
    }

    /// Filter tasks list against terms
    pub fn filter_terms(&mut self, terms: &[String]) {
        self.0.retain(|t| {
            for term in terms.iter() {
                if !t.raw.contains(term) {
                    return false;
                }
            }
            true
        });
    }

    /// Sort task list by slice of TaskSort objects
    pub fn sort(&mut self, sorts: &[SortBy]) {
        self.0.sort_by(|a, b| {
            let mut cmp = Ordering::Equal;
            for sort in sorts {
                if cmp != Ordering::Equal {
                    break;
                }
                cmp = match sort.field {
                    SortByField::CompleteDate => a.parsed.finish_date.cmp(&b.parsed.finish_date),
                    SortByField::Completed => a.parsed.finished.cmp(&b.parsed.finished),
                    SortByField::Context => a.parsed.contexts.get(0).cmp(&b.parsed.contexts.get(0)),
                    SortByField::CreateDate => a.parsed.create_date.cmp(&b.parsed.create_date),
                    SortByField::DueDate => a.parsed.due_date.cmp(&b.parsed.due_date),
                    SortByField::Id => a.id.cmp(&b.id),
                    SortByField::Priority => a.parsed.priority.cmp(&b.parsed.priority),
                    SortByField::Project => a.parsed.projects.get(0).cmp(&b.parsed.projects.get(0)),
                    SortByField::Body => a.parsed.subject.cmp(&b.parsed.subject),
                    SortByField::Raw => a.raw.cmp(&b.raw),
                    SortByField::ThresholdDate => {
                        a.parsed.threshold_date.cmp(&b.parsed.threshold_date)
                    }
                };
                cmp = if sort.reverse { cmp.reverse() } else { cmp };
            }
            cmp
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// Contains parsed task data and original raw string
pub struct Task {
    /// Line number in todo.txt file
    pub id: usize,
    /// Task data parsed by todo_txt crate
    pub parsed: todo_txt::Task,
    /// Original unmodified text
    pub raw: String,
}

impl Task {
    /// Create new task from string and ID
    pub fn new<T>(id: usize, raw_text: T) -> Self
    where
        T: Into<String>,
        T: Copy,
    {
        Task {
            id,
            parsed: todo_txt::parser::task(&raw_text.into())
                .unwrap_or_else(|_| panic!("couldn't parse into todo: '{}'", raw_text.into())),
            raw: raw_text.into(),
        }
    }

    /// Turn into blank task with same id
    pub fn clear(&self) -> Self {
        Task::new(self.id, "")
    }

    /// Returns true if the task is a blank line
    pub fn is_blank(&self) -> bool {
        self.raw == ""
    }

    /// Normalize whitespace (condense >1 space to 1) and reparse
    pub fn normalize_whitespace(&self) -> Self {
        Task::new(
            self.id,
            &self.raw.split_whitespace().collect::<Vec<&str>>().join(" "),
        )
    }

    /// Turn into plain string with properly padded line number
    #[allow(dead_code)]
    pub fn stringify(&self, task_ct: usize) -> impl Display {
        format!(
            "{:0ct$} {}",
            self.id,
            self.raw,
            ct = task_ct.to_string().len(),
        )
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw
            .to_ascii_lowercase()
            .cmp(&other.raw.to_ascii_lowercase())
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.id, self.raw,)
    }
}

/// Convert a slice of tasks to a newline-delimited string
pub fn tasks_to_string(ctx: &mut Context) -> Result<String> {
    if ctx.opts.remove_blank_lines {
        ctx.tasks.retain(|t| !t.is_blank());
    }
    Ok(ctx.tasks.to_string())
}

/// Fields of `Task` we can sort by
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortByField {
    /// Parsed body of the task
    Body,
    /// Complete date of completed task
    CompleteDate,
    /// Whether task is completed or not
    Completed,
    /// The first context
    Context,
    /// Create date if present
    CreateDate,
    /// Due date tag if present
    DueDate,
    /// Line number
    Id,
    /// Priority code (A-Z)
    Priority,
    /// The first project
    Project,
    /// The unparsed line from todo.txt file
    Raw,
    /// Threshold date tag if present
    ThresholdDate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortBy {
    /// Sorting criterion
    pub field: SortByField,
    /// Whether to reverse the sort
    pub reverse: bool,
}