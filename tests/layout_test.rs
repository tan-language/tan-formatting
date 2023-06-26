use common::parse_file;
use tan_fmt::layout::Arranger;

mod common;

// #TODO convert these to tests?

#[test]
pub fn arrange_handles_data_input() {
    let exprs = parse_file("data.tan").unwrap();
    let mut arranger = Arranger::new(&exprs);
    let layout = arranger.arrange();

    dbg!(&layout);

    // let expected_output = read_file("data.pretty.tan");

    // println!("{output}");

    // assert_eq!(output, expected_output);
}

#[test]
pub fn arrange_handles_code_input() {
    let exprs = parse_file("code.tan").unwrap();
    let mut arranger = Arranger::new(&exprs);
    let layout = arranger.arrange();

    dbg!(&layout);
}

#[test]
pub fn arrange_handles_more_code_input() {
    let exprs = parse_file("fibalike.tan").unwrap();
    let mut arranger = Arranger::new(&exprs);
    let layout = arranger.arrange();

    dbg!(&layout);
}

// #TODO consider renaming `inline` to `side`?

#[test]
pub fn arrange_handles_inline_comments() {
    let exprs = parse_file("inline-comments.tan").unwrap();
    let mut arranger = Arranger::new(&exprs);
    let layout = arranger.arrange();

    eprintln!("{:?}", layout);
    dbg!(&layout);
}
