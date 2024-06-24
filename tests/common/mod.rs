use tan::api::lex_string;
use tan::error::Error;
use tan::expr::Expr;
use tan::lexer::token::Token;
use tan::lexer::Lexer;
use tan::parser::Parser;

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
    parse_string_all(input)
}

// #todo Extract to util library (tan-analysis)?
// Custom method that uses the analysis parser!
pub fn parse_string_all(input: impl AsRef<str>) -> Result<Vec<Expr>, Vec<Error>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::for_analysis(&tokens);
    let exprs = parser.parse()?;

    Ok(exprs)
}
