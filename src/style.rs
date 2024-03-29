use crate::{config::AppContext, prelude::*, util::get_pri_name};
use serde::Deserialize;

use termcolor::{Color, ColorSpec};

#[derive(Debug, Deserialize)]
/// Color settings for terminal output
pub struct Style {
    pub name:      String,
    pub color_fg:  Option<u8>,
    pub color_bg:  Option<u8>,
    pub bold:      Option<bool>,
    pub intense:   Option<bool>,
    pub underline: Option<bool>,
}

impl Style {
    pub fn default(name: &str) -> Style {
        let mut default = Style {
            name:      name.into(),
            color_fg:  None,
            color_bg:  None,
            bold:      None,
            intense:   None,
            underline: None,
        };
        if name.starts_with("pri") {
            match name {
                "pri_a" => default.color_fg = Some(Ansi::HOTPINK),
                "pri_b" => default.color_fg = Some(Ansi::GREEN),
                "pri_c" => default.color_fg = Some(Ansi::BLUE),
                "pri_d" => default.color_fg = Some(Ansi::TURQUOISE),
                _ => default.color_fg = Some(Ansi::TAN),
            }
            default
        } else {
            match name {
                "project" => default.color_fg = Some(Ansi::LIME),
                "context" => default.color_fg = Some(Ansi::LIGHTORANGE),
                _ => default.color_fg = None,
            }
            default
        }
    }
}

#[derive(Debug)]
/// Store constants of ANSI 256-color code
pub struct Ansi;

#[allow(dead_code)]
impl Ansi {
    pub const BLUE: u8 = 4;
    pub const GREEN: u8 = 2;
    pub const GREY: u8 = 246;
    pub const HOTPINK: u8 = 198;
    pub const LIGHTORANGE: u8 = 215;
    pub const LIME: u8 = 154;
    pub const OLIVE: u8 = 113;
    pub const SKYBLUE: u8 = 111;
    pub const TAN: u8 = 179;
    pub const TURQUOISE: u8 = 37;
}

/// Get item style from preferences (or default)
pub fn get_colors_from_style(name: &str, ctx: &AppContext) -> Result<ColorSpec> {
    // TODO: build ColorSpecs for each style in the configuration and iterate once
    let default_style = Style::default(&name);
    let style = ctx
        .styles
        .iter()
        .find(|i| i.name.to_ascii_lowercase() == name)
        .unwrap_or(&default_style);
    let mut color = ColorSpec::new();
    color.set_reset(false);
    if let Some(fg) = style.color_fg {
        color.set_fg(Some(Color::Ansi256(fg)));
    }
    if let Some(bg) = style.color_bg {
        color.set_bg(Some(Color::Ansi256(bg)));
    }
    color.set_bold(style.bold.unwrap_or(false));
    color.set_intense(style.intense.unwrap_or(false));
    color.set_underline(style.underline.unwrap_or(false));
    Ok(color)
}

// pub fn get_stylespec(name: &str, ctx: &AppContext) -> Result<color::StyleContext> {
//     let default_style = Style::default(&name);
//     let style = ctx
//         .styles
//         .iter()
//         .find(|i| i.name.to_ascii_lowercase() == name)
//         .unwrap_or(&default_style);
//     let mut color_style = StyleContext::default();
//     if let Some(fg) = style.color_fg {
//         color_style.add(color::StyleSpec::Fg(color::Color::Ansi256(fg)));
//     }
//     if let Some(bg) = style.color_bg {
//         color_style.add(color::StyleSpec::Bg(color::Color::Ansi256(bg)));
//     }
//     if style.bold == Some(true) {
//         color_style.add(color::StyleSpec::Bold);
//     }
//     if style.underline == Some(true) {
//         color_style.add(color::StyleSpec::Underline);
//     }
//     Ok(color_style)
// }

/// Format output and add color to priorities, projects and contexts
pub fn format_buffer<W>(buf: &mut W, ctx: &AppContext) -> Result
where
    W: std::io::Write + termcolor::WriteColor,
{
    // let leading_zeros = max(1, ctx.task_ct.to_string().len());
    let leading_zeros = ctx.task_ct.to_string().len();
    for task in &*ctx.tasks {
        let line = &task.raw;
        let pri = get_pri_name(u8::from(task.parsed.priority.clone())).unwrap_or_default();
        let color = if task.parsed.finished {
            get_colors_from_style("done", ctx)?
        } else {
            get_colors_from_style(&pri, ctx)?
        };
        buf.set_color(&color)?;
        // write line number
        // TODO: why is this leaving out leading zero suddenly?
        write!(buf, "{:0width$} ", task.id, width = leading_zeros)?;
        let mut words = line.split_whitespace().peekable();
        while let Some(word) = words.next() {
            let first_char = word.chars().next();
            let prev_color = color.clone();
            match first_char {
                Some('+') => {
                    if ctx.opts.hide_project % 2 == 0 {
                        buf.set_color(&get_colors_from_style("project", ctx)?)?;
                        write!(buf, "{}", word)?;
                        buf.reset()?;
                        buf.set_color(&prev_color)?;
                    }
                }
                Some('@') => {
                    if ctx.opts.hide_context % 2 == 0 {
                        buf.set_color(&get_colors_from_style("context", ctx)?)?;
                        write!(buf, "{}", word)?;
                        buf.reset()?;
                        buf.set_color(&prev_color)?;
                    }
                }
                _ => {
                    write!(buf, "{}", word)?;
                }
            }
            if words.peek().is_some() {
                write!(buf, " ")?;
            }
        }
        if !task.parsed.priority.is_lowest() || task.parsed.finished {
            buf.reset()?;
        }
        writeln!(buf)?;
    }
    Ok(())
}
