use common::parse_file;
use tan_formatting::{pretty::Formatter, types::Dialect};

use crate::common::read_file;

mod common;

// #todo a lot of duplication here, refactor.

// #todo find a better name.
// #todo support dialect.
fn test_fixture(name: &str) {
    let exprs = parse_file(&format!("{name}.tan")).unwrap();
    let formatter = Formatter::new(&exprs);

    let output = formatter.format();
    let expected_output = read_file(&format!("{name}.pretty.tan"));

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_data_input() {
    let exprs = parse_file("data.tan").unwrap();
    let formatter = Formatter::for_dialect(&exprs, Dialect::Data);

    let output = formatter.format();
    let expected_output = read_file("data.pretty.tan");

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_data_using_dialect() {
    let exprs = parse_file("data-2.tan").unwrap();
    let formatter = Formatter::for_dialect(&exprs, Dialect::Data);

    let output = formatter.format();
    let expected_output = read_file("data-2.pretty.tan");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_code_input() {
    test_fixture("code");
}

#[test]
pub fn format_pretty_handles_function_definitions() {
    test_fixture("func-def");
}

#[test]
pub fn format_pretty_handles_more_code_input() {
    test_fixture("fibalike");
}

#[test]
pub fn format_pretty_handles_inline_comments() {
    test_fixture("inline-comments");
}

#[test]
pub fn format_pretty_handles_nested_function() {
    test_fixture("nested-function");
}

#[test]
pub fn format_pretty_handles_nested_call() {
    test_fixture("nested-call");
}

#[test]
pub fn format_pretty_makes_quoting_uniform() {
    test_fixture("quote");
}

#[test]
pub fn format_pretty_handles_cond() {
    test_fixture("cond");
}

#[test]
pub fn format_pretty_handles_for() {
    test_fixture("for");
}

#[test]
pub fn format_pretty_retains_interpolated_strings() {
    test_fixture("interpolated-str");
}

#[test]
pub fn format_pretty_makes_unquoting_uniform() {
    test_fixture("unquote");
}

// #todo make this test pass.
// #[test]
// pub fn pairs_and_comments_pathological_case() {
//     test_fixture("pairs-and-comments");
// }
