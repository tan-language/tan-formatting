use common::parse_file;
use tan_fmt::format_expr_compact;

use crate::common::read_file;

mod common;

#[test]
pub fn format_compact_works() {
    let expr = parse_file("simple_example.tan").unwrap();
    let output = format_expr_compact(expr);
    let expected_output = read_file("simple_example.compact.tan");

    assert_eq!(output, expected_output);
}
