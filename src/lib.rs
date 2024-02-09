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

    // #insight
    // Currently because of wrong input the line may be missing, so we cannot
    // unwrap.
    let line_str = input.lines().nth(range.start.line).unwrap_or_else(|| "???");

    let line_space = " ".repeat(format!("{}", range.start.line + 1).len());

    let len = range.end.index - range.start.index;

    // #todo use `^` or `-` depending on note importance, like Rust.

    let indicator = "^".repeat(len);

    let indicator_space = " ".repeat(range.start.col);

    // #todo trim the leading spaces from the line?

    format!(
        "{}|\n{}| {}\n{}|{} {} {}",
        line_space,
        range.start.line + 1,
        line_str,
        line_space,
        indicator_space,
        indicator,
        note.text,
    )
}

pub fn format_error(error: &Error) -> String {
    format!("{} at {}", error.kind(), error.file_path)
}

// #todo reuse this in format_error_pretty.
pub fn format_error_short(error: &Error) -> String {
    if let Some(note) = error.notes.first() {
        if let Some(range) = &note.range {
            let position = &range.start;
            return format!(
                "{} at {}:{}:{}",
                error.kind(),
                error.file_path,
                position.line + 1,
                position.col + 1,
            );
        }
    }
    format!("{} at {}", error.kind(), error.file_path)
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
