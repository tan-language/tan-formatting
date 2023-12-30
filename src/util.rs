use std::sync::OnceLock;

use regex::Regex;

static TRAILING_EOL_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn ensure_ends_with_empty_line(input: &str) -> String {
    let regex = TRAILING_EOL_REGEX.get_or_init(|| Regex::new(r"\n+$").unwrap());

    let mut output = regex.replace(input, "\n").to_string();

    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

static SEPARATOR_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn trim_separators(input: &str) -> String {
    let regex = SEPARATOR_REGEX.get_or_init(|| Regex::new(r"[ \t]+\n").unwrap());
    regex.replace_all(input, "\n").to_string()
}

pub fn escape_string(input: &str) -> String {
    input
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use crate::util::escape_string;

    #[test]
    fn escape_string_works() {
        let input = "this is \"cool\"";
        let escaped = escape_string(input);

        assert_eq!(escaped, "this is \\\"cool\\\"");

        let input = "first\nsecond";
        let escaped = escape_string(input);

        assert_eq!(escaped, "first\\nsecond");
    }
}
