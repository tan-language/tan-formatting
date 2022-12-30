use std::error::Error;

use tan::{expr::Expr, range::Ranged};

// #TODO split into `format_expr`, `format_error`.

/// Formats an expression in compact form.
pub fn format_expr_compact(expr: &Expr) -> String {
    // #TODO even compact, strip whitespace aggressively!
    format!("{expr}")
}

/// Formats an expression in aestheticall pleasing form.
/// This is the standard textual representation of expressions.
pub fn format_expr_pretty(_expr: &Expr) -> String {
    todo!()
}

// #TODO implement this in ...Tan :)
// #TODO move to other crate? e.g. tan_fmt, or tan_analysis, or tan_cli?
// #TODO format the error as symbolic expression.
// #TODO format the error as JSON.
// #TODO make more beautiful than Rust.
// #TODO add as method to Ranged<E: Error>? e.g. `format_pretty`

pub fn format_error_pretty<E: Error>(error: &Ranged<E>, input: &str, url: Option<&str>) -> String {
    let chars = input.chars();
    let Ranged(error, span) = error;

    let mut index: usize = 0;
    let mut line = 0;
    let mut line_start: usize = 0;
    let mut line_str = String::new();

    for c in chars {
        index += 1;

        if c == '\n' {
            if index > span.start {
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

    let len = span.len();

    let indicator = if len == 1 {
        "^--- near here".to_owned()
    } else {
        "^".repeat(len)
    };

    let col = span.start - line_start;
    let indicator_space = " ".repeat(col);

    let url = url.unwrap_or("input");

    format!(
        "error: {error}\n{}at {url}:{}:{}\n{}|\n{}| {}\n{}|{} {}",
        line_space,
        line + 1,
        col + 1,
        line_space,
        line + 1,
        line_str,
        line_space,
        indicator_space,
        indicator,
    )
}
