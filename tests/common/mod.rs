use tan::api::lex_string;
use tan::error::Error;
use tan::expr::Expr;
use tan::lexer::token::Token;
use tan_analysis::parsing::parse_string_for_analysis;

pub fn read_file(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
}

#[allow(dead_code)]
pub fn lex_file(filename: &str) -> Result<Vec<Token>, Vec<Error>> {
    let input = &read_file(filename);
    lex_string(input)
}

#[allow(dead_code)]
pub fn parse_file(filename: &str) -> Result<Vec<Expr>, Vec<Error>> {
    let input = &read_file(filename);
    parse_string_for_analysis(input)
}
