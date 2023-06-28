use std::sync::OnceLock;

use regex::Regex;

static TRAILING_EOL_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn ensure_ends_with_empty_line(input: &str) -> String {
    let regex = TRAILING_EOL_REGEX.get_or_init(|| Regex::new(r"\n+$").unwrap());

    let mut output = regex.replace(&input, "\n").to_string();

    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

static SEPARATOR_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn trim_separators(input: &str) -> String {
    let regex = SEPARATOR_REGEX.get_or_init(|| Regex::new(r"[ \t]+\n").unwrap());
    regex.replace_all(&input, "\n").to_string()
}

// #TODO what about other float types?
/// Formats a float number, ensures it always has a decimal separator.
pub fn format_float(n: f64) -> String {
    let s = n.to_string();

    if !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}
