use crate::prelude::*;
use std::io::{stdin, stdout, Write};

/// Get user response to question as 'y' or 'n'
pub fn ask_user_yes_no(prompt_ln: &str) -> Result<bool> {
    let mut input = String::new();
    let stdout = stdout();
    let mut lock = stdout.lock();
    lock.write_all(prompt_ln.as_bytes())?;
    stdin().read_line(&mut input)?;
    if let Some(c) = input.to_lowercase().chars().next() {
        debug!("User input: '{}'", c);
        if c == 'y' {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Get string priority name in the form of `pri_x`
pub fn get_pri_name(pri: u8) -> Option<String> {
    match pri {
        0..=25 => {
            let mut s = String::from("pri_");
            s.push((pri + 97).into());
            Some(s)
        }
        _ => None,
    }
}
