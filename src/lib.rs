pub mod layout;
pub mod pretty;
pub mod types;
mod util;

use tan::error::{Error, ErrorNote};

// #todo reuse the Position from tan?
// #todo split into `format_expr`, `format_error`.
// #todo add special support for formatting multiple errors?

pub fn format_error_note_pretty(note: &ErrorNote, input: &str) -> String {
    let Some(range) = &note.range else {
        return note.text.to_string();
    };

    // #todo do this once, outside of this function!
    // #todo can we reuse the position line/col?

    let chars = input.chars();

    let mut index: usize = 0;
    let mut line = 0;
    let mut line_start: usize = 0;
    let mut line_str = String::new();

    for c in chars {
        index += 1;

        if c == '\n' {
            if index > range.start.index {
                break;
            }

            line += 1;
            line_start = index;

            line_str.clear();

            continue;
        }

        line_str.push(c);
    }

    let line_space = " ".repeat(format!("{}", line + 1).len());

    let len = range.end.index - range.start.index;

    // let indicator = if len == 1 {
    //     "^--- near here".to_owned()
    // } else {
    //     "^".repeat(len)
    // };

    // #todo use `^` or `-` depending on note importance, like Rust.

    let indicator = "^".repeat(len);

    let col = range.start.index - line_start; // #todo range.start.col
    let indicator_space = " ".repeat(col);

    format!(
        "{}|\n{}| {}\n{}|{} {} {}",
        line_space,
        line + 1,
        line_str,
        line_space,
        indicator_space,
        indicator,
        note.text,
    )
}

pub fn format_error(error: &Error) -> String {
    format!("{}\n", error.kind())
}

// #todo also format error without input.
// #todo implement this in ...Tan :)
// #todo format the error as symbolic expression.
// #todo format the error as JSON.
// #todo make more beautiful than Rust.
// #todo add as method to Ranged<E: Error>? e.g. `format_pretty`
pub fn format_error_pretty(error: &Error, input: &str) -> String {
    let Some(note) = error.notes.first() else {
        return format!("{}\n at {}", error.kind(), error.file_path);
    };

    let prologue = if let Some(range) = &note.range {
        let position = &range.start;
        format!(
            "{}\n at {}:{}:{}",
            error.kind(),
            error.file_path,
            position.line + 1,
            position.col + 1,
        )
    } else {
        format!("{}\n at {}", error.kind(), error.file_path)
    };

    let notes: Vec<String> = error
        .notes
        .iter()
        .map(|note| format_error_note_pretty(note, input))
        .collect();

    format!("{prologue}\n{}", notes.join("\n"))
}
