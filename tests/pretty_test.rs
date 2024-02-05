use common::parse_file;
use tan_formatting::{pretty::Formatter, types::Dialect};

use crate::common::read_file;

mod common;

// #todo find a better name.
fn test_fixture(name: &str, dialect: Dialect) {
    let exprs = parse_file(&format!("{name}.tan")).unwrap();
    let formatter = Formatter::for_dialect(&exprs, dialect);

    let output = formatter.format();
    let expected_output = read_file(&format!("{name}.pretty.tan"));

    // eprintln!("{output}");

    assert_eq!(output, expected_output);
}

fn test_code_fixture(name: &str) {
    test_fixture(name, Dialect::Code)
}

fn test_data_fixture(name: &str) {
    test_fixture(name, Dialect::Data)
}

#[test]
pub fn format_pretty_handles_data_input() {
    test_data_fixture("data");
}

#[test]
pub fn format_pretty_handles_data_using_dialect() {
    test_data_fixture("data-2");
}

#[test]
pub fn format_pretty_handles_code_input() {
    test_code_fixture("code");
}

#[test]
pub fn format_pretty_handles_function_definitions() {
    test_code_fixture("func-def");
}

#[test]
pub fn format_pretty_handles_more_code_input() {
    test_code_fixture("fibalike");
}

#[test]
pub fn format_pretty_handles_inline_comments() {
    test_code_fixture("inline-comments");
}

#[test]
pub fn format_pretty_handles_nested_function() {
    test_code_fixture("nested-function");
}

#[test]
pub fn format_pretty_handles_nested_call() {
    test_code_fixture("nested-call");
}

#[test]
pub fn format_pretty_makes_quoting_uniform() {
    test_code_fixture("quote");
}

#[test]
pub fn format_pretty_handles_cond() {
    test_code_fixture("cond");
}

#[test]
pub fn format_pretty_handles_for() {
    test_code_fixture("for");
}

#[test]
pub fn format_pretty_retains_interpolated_strings() {
    test_code_fixture("interpolated-str");
}

#[test]
pub fn format_pretty_makes_unquoting_uniform() {
    test_code_fixture("unquote");
}

// #todo make this test pass.
// pathological snippet with inline comment.
// #think the formatter cannot format vertically, but it should at least handle the inline comment
#[test]
pub fn pairs_and_comments_pathological_case() {
    test_code_fixture("pairs-and-comments");
}
