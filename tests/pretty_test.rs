use common::lex_file;
use tan_fmt::pretty::Formatter;

use crate::common::read_file;

mod common;

#[test]
pub fn format_pretty_handles_data_input() {
    let input = lex_file("data.tan").unwrap();
    let mut formatter = Formatter::new(input);
    let output = formatter.format().unwrap();
    let expected_output = read_file("data.pretty.tan");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_code_input() {
    let input = lex_file("code.tan").unwrap();
    let mut formatter = Formatter::new(input);
    let output = formatter.format().unwrap();
    let expected_output = read_file("code.pretty.tan");

    assert_eq!(output, expected_output);
}

#[test]
pub fn format_pretty_handles_more_code_input() {
    let input = lex_file("fibonacci.tan").unwrap();
    let mut formatter = Formatter::new(input);
    let output = formatter.format().unwrap();
    let expected_output = read_file("fibonacci.pretty.tan");

    println!("{output}");

    // assert_eq!(output, expected_output);
}
