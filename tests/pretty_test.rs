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

    println!("{output}");

    assert_eq!(output, expected_output);
}
