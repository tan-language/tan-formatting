use std::sync::OnceLock;

use regex::Regex;

static EEWEL_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn ensure_ends_with_empty_line(input: &str) -> String {
    let regex = EEWEL_REGEX.get_or_init(|| Regex::new(r"\n+$").unwrap());

    let mut input = regex.replace(&input, "\n").to_string();

    if !input.ends_with('\n') {
        input.push('\n');
    }

    input
}
