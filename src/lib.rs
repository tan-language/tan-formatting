pub mod pretty;
mod util;

use tan::{
    error::{Error, ErrorNote},
    range::Position,
};

// #TODO reuse the Position from tan?
// #TODO split into `format_expr`, `format_error`.
// #TODO add special support for formatting multiple errors?

pub fn format_error_note_pretty(note: &ErrorNote, input: &str) -> String {
    let Some(range) = &note.range else {
        return format!("{}", note.text);
    };

    // #TODO do this once, outside of this function!
    let chars = input.chars();

    let mut index: usize = 0;
    let mut line = 0;
    let mut line_start: usize = 0;
    let mut line_str = String::new();

    for c in chars {
        index += 1;

        if c == '\n' {
            if index > range.start {
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

    let len = range.len();

    // let indicator = if len == 1 {
    //     "^--- near here".to_owned()
    // } else {
    //     "^".repeat(len)
    // };

    // #TODO use `^` or `-` depending on note importance, like Rust.

    let indicator = "^".repeat(len);

    let col = range.start - line_start;
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

// #TODO also format error without input.
// #TODO implement this in ...Tan :)
// #TODO format the error as symbolic expression.
// #TODO format the error as JSON.
// #TODO make more beautiful than Rust.
// #TODO add as method to Ranged<E: Error>? e.g. `format_pretty`
pub fn format_error_pretty(error: &Error, input: &str) -> String {
    let Some(note) = error.notes.first() else {
        return format!("{}\n at {}", error.kind(), error.file_path);
    };

    let prologue = if let Some(range) = &note.range {
        let position = Position::from_range(&range, input);
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

// #TODO also format error without input.
// #TODO implement this in ...Tan :)
// #TODO format the error as symbolic expression.
// #TODO format the error as JSON.
// #TODO make more beautiful than Rust.
// #TODO add as method to Ranged<E: Error>? e.g. `format_pretty`
// pub fn format_error_pretty_old<E: Error>(error: &E, input: &str, url: Option<&str>) -> String {
//     let chars = input.chars();

//     let mut index: usize = 0;
//     let mut line = 0;
//     let mut line_start: usize = 0;
//     let mut line_str = String::new();

//     for c in chars {
//         index += 1;

//         if c == '\n' {
//             if index > error.range.start {
//                 break;
//             }

//             line += 1;
//             line_start = index;

//             line_str.clear();

//             continue;
//         }

//         line_str.push(c);
//     }

//     let line_space = " ".repeat(format!("{}", line + 1).len());

//     let len = span.len();

//     let indicator = if len == 1 {
//         "^--- near here".to_owned()
//     } else {
//         "^".repeat(len)
//     };

//     let col = span.start - line_start;
//     let indicator_space = " ".repeat(col);

//     let url = url.unwrap_or("input");

//     format!(
//         "{error}\n{}at {url}:{}:{}\n{}|\n{}| {}\n{}|{} {}",
//         line_space,
//         line + 1,
//         col + 1,
//         line_space,
//         line + 1,
//         line_str,
//         line_space,
//         indicator_space,
//         indicator,
//     )
// }
