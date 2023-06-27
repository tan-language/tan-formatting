use common::parse_file;
use tan_fmt::pretty::Formatter;

use crate::common::read_file;

mod common;

// #TODO a lot of duplication here, refactor.

#[test]
pub fn format_pretty_handles_data_input() {
    let exprs = parse_file("data.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("data.pretty.tan");

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_code_input() {
    let exprs = parse_file("code.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("code.pretty.tan");

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_more_code_input() {
    let exprs = parse_file("fibalike.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("fibalike.pretty.tan");

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_inline_comments() {
    let exprs = parse_file("inline-comments.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("inline-comments.pretty.tan");

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_nested_function() {
    let exprs = parse_file("nested-function.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("nested-function.pretty.tan");

    eprintln!("{output}");

    assert_eq!(output, expected_output);
}
