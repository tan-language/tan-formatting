use tan::error::Error;
use tan::{lexer::token::Token, parser::NonRecoverableError, range::Ranged};

// #TODO add pragmas to define sections with different formatting options or even disabled formatting.
// #TODO try to use annotations to define the above-mentioned sections.
// #TODO this is the ugliest code ever written, wip.
// #TODO rename to `formatter.rs`
// #TODO how to handle parse errors?

const IDENT_SIZE: usize = 4;

pub struct Formatter<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    tokens: I::IntoIter,
    nesting: usize,
    errors: Vec<Error>,
}

impl<I> Formatter<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    pub fn new(tokens: I) -> Self {
        let tokens = tokens.into_iter();

        Self {
            tokens,
            nesting: 0,
            errors: Vec::new(),
        }
    }

    fn push_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn format_list(&mut self, delimiter: Token) -> Result<String, NonRecoverableError> {
        let mut output = String::new();

        self.nesting += 1;

        loop {
            let Some(token) = self.tokens.next() else {
                // #TODO how to handle this?
                self.push_error(Error::UnterminatedList);
                return Err(NonRecoverableError {});
            };

            if token.0 == delimiter {
                self.nesting -= 1;
                return Ok(output);
            } else {
                let s = self.format_expr(token)?;
                output.push_str(&format!("{}{s}\n", " ".repeat(self.nesting * IDENT_SIZE)));
            }
        }
    }

    pub fn format_dict(&mut self, delimiter: Token) -> Result<String, NonRecoverableError> {
        let mut output = String::new();

        self.nesting += 1;

        loop {
            loop {
                let Some(token) = self.tokens.next() else {
                    // #TODO how to handle this?
                    self.push_error(Error::UnterminatedList);
                    return Err(NonRecoverableError {});
                };

                let cont = matches!(token.0, Token::Comment(..) | Token::Annotation(..));

                if token.0 == delimiter {
                    self.nesting -= 1;
                    return Ok(output);
                } else {
                    let s = self.format_expr(token)?;
                    output.push_str(&format!("{}{s}", " ".repeat(self.nesting * IDENT_SIZE)));
                }

                if !cont {
                    break;
                }
            }

            loop {
                let Some(token) = self.tokens.next() else {
                    // #TODO how to handle this?
                    self.push_error(Error::UnterminatedList);
                    return Err(NonRecoverableError {});
                };

                if matches!(token.0, Token::Comment(..) | Token::Annotation(..)) {
                    let s = self.format_expr(token)?;
                    output.push_str(&format!(" {s}"));
                } else {
                    if token.0 == delimiter {
                        self.nesting -= 1;
                        return Ok(output);
                    } else {
                        let s = self.format_expr(token)?;
                        output.push_str(&format!(" {s}\n"));
                    }

                    break;
                }
            }
        }
    }

    pub fn format_expr(&mut self, token: Ranged<Token>) -> Result<String, NonRecoverableError> {
        let Ranged(t, _) = token;

        let output = match t {
            Token::Comment(s) => format!("{s}\n"),
            Token::String(s) => format!("\"{s}\""),
            Token::Symbol(s) => s,
            Token::Number(n) => n.to_string(),
            Token::Annotation(s) => {
                format!("#{s}")
            }
            Token::Quote => "'".to_owned(),
            Token::LeftParen => {
                let mut s = "(\n".to_string();
                s.push_str(&self.format_list(Token::RightParen)?);
                s.push_str(&format!("{})", " ".repeat(self.nesting * IDENT_SIZE)));
                s
            }
            Token::LeftBracket => {
                // Syntactic sugar for a List/Array.

                let mut s = "[\n".to_string();
                s.push_str(&self.format_list(Token::RightBracket)?);
                s.push_str(&format!("{}]", " ".repeat(self.nesting * IDENT_SIZE)));
                s
            }
            Token::LeftBrace => {
                // Syntactic sugar for a Dict.

                let mut s = "{\n".to_string();
                s.push_str(&self.format_dict(Token::RightBrace)?);
                s.push_str(&format!("{}}}", " ".repeat(self.nesting * IDENT_SIZE)));
                s
            }
            Token::RightParen | Token::RightBracket | Token::RightBrace => {
                // #TODO custom error for this?
                self.push_error(Error::UnexpectedToken(t));
                // Parsing can continue.
                return Ok("".to_owned());
            }
        };

        Ok(output)
    }

    /// Formats an expression in aestheticall pleasing form.
    /// This is the standard textual representation of expressions.
    pub fn format(&mut self) -> Result<String, Vec<Error>> {
        let mut output = String::new();

        loop {
            let Some(token) = self.tokens.next() else {
                break;
            };

            let Ok(s) = self.format_expr(token) else {
                // A non-recoverable parse error was detected, stop parsing.
                let errors = std::mem::take(&mut self.errors);
                return Err(errors);
                // break;
            };

            output.push_str(&s);
        }

        Ok(output)
    }
}
