// use common::parse_file;
// use tan_fmt::pretty::Formatter;

// use crate::common::read_file;

// mod common;

// #[test]
// pub fn format_pretty_handles_data_input() {
//     let exprs = parse_file("data.tan").unwrap();
//     let mut formatter = Formatter::new(&exprs);
//     let output = formatter.format();

//     let expected_output = read_file("data.pretty.tan");

//     // println!("{output}");

//     assert_eq!(output, expected_output);
// }

// #[test]
// pub fn format_pretty_handles_code_input() {
//     let exprs = parse_file("code.tan").unwrap();
//     let mut formatter = Formatter::new(&exprs);
//     let output = formatter.format();
//     let expected_output = read_file("code.pretty.tan");

//     assert_eq!(output, expected_output);
// }

// #[test]
// pub fn format_pretty_handles_more_code_input() {
//     let exprs = parse_file("fibalike.tan").unwrap();

//     dbg!(&exprs);

//     let mut formatter = Formatter::new(&exprs);
//     let output = formatter.format();
//     let expected_output = read_file("fibalike.pretty.tan");

//     assert_eq!(output, expected_output);
// }

// // #TODO consider renaming `inline` to `side`?

// // #[test]
// pub fn format_pretty_handles_inline_comments() {
//     let exprs = parse_file("inline-comments.tan").unwrap();

//     // dbg!(&exprs);

//     let mut formatter = Formatter::new(&exprs);
//     let output = formatter.format();
//     let expected_output = read_file("inline-comments.pretty.tan");

//     eprintln!("{output}");

//     assert_eq!(output, expected_output);
// }
