use common::parse_file;
use tan_formatting::pretty::Formatter;

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

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_nested_call() {
    let exprs = parse_file("nested-call.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("nested-call.pretty.tan");

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_makes_quoting_uniform() {
    let exprs = parse_file("quote.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("quote.pretty.tan");

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_cond() {
    let exprs = parse_file("cond.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("cond.pretty.tan");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_retains_interpolated_strings() {
    let exprs = parse_file("interpolated-str.tan").unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file("interpolated-str.pretty.tan");

    assert_eq!(output, expected_output);
}
