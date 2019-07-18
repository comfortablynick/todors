use crate::errors::Result;
use log::debug;
use std::io::{stdin, stdout, Write};

/// Get user response to question as 'y' or 'n'
pub fn ask_user_yes_no(prompt_ln: &str) -> Result<bool> {
    let mut cin = String::new();
    stdout().write_all(prompt_ln.as_bytes())?;
    stdout().flush()?;
    stdin().read_line(&mut cin)?;
    if let Some(c) = cin.to_lowercase().chars().next() {
        debug!("User input: '{}'", c);
        if c == 'y' {
            return Ok(true);
        }
    }
    Ok(false)
}
