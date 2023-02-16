use tan::error::Error;
use tan::util::Break;
use tan::{lexer::token::Token, range::Ranged};

// #TODO add pragmas to define sections with different formatting options or even disabled formatting.
// #TODO try to use annotations to define the above-mentioned sections.
// #TODO this is the ugliest code ever written, wip.
// #TODO rename to `formatter.rs`
// #TODO how to handle parse errors?
// #TODO optimize formatter to minimize diffs.
// #TODO try to maintain some empty separator lines.
// #TODO consider using tabs to indent?
// #TODO consider allowing absolutely no parameters for the formatter.

/// The default indentation size (char count)
const DEFAULT_INDENT_SIZE: usize = 4;

/// The default (target) line size (char count)
const DEFAULT_LINE_SIZE: usize = 80;

pub struct Formatter<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    tokens: I::IntoIter,
    nesting: usize,
    lookahead: Vec<Ranged<Token>>,
    errors: Vec<Error>,
    indent_size: usize,
    #[allow(dead_code)]
    line_size: usize,
    #[allow(dead_code)]
    col: usize,
}

// #TODO introduce default constructor.
// #TODO introduce 'builder' api?

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
            indent_size: DEFAULT_INDENT_SIZE,
            line_size: DEFAULT_LINE_SIZE,
            lookahead: Vec::new(),
            col: 0,
        }
    }

    // #TODO unit test
    // #TODO refactor
    fn next_token(&mut self) -> Option<Ranged<Token>> {
        if let Some(t) = self.lookahead.pop() {
            return Some(t);
        }

        self.tokens.next()
    }

    fn put_back_token(&mut self, t: Ranged<Token>) {
        self.lookahead.push(t);
    }

    fn push_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    // #TODO find better name
    pub fn format_list_horizontal(&mut self, delimiter: Token) -> Result<String, Break> {
        let mut output: Vec<String> = Vec::new();

        loop {
            let Some(token) = self.next_token() else {
                // #TODO how to handle this?
                self.push_error(Error::UnterminatedList);
                return Err(Break {});
            };

            if token.0 == delimiter {
                return Ok(output.join(" "));
            } else {
                let s = self.format_expr(token)?;
                output.push(s);
            }
        }
    }

    // #TODO find better name
    pub fn format_list_vertical(&mut self, delimiter: Token) -> Result<String, Break> {
        let mut output: Vec<String> = Vec::new();

        loop {
            let Some(token) = self.next_token() else {
                // #TODO how to handle this?
                self.push_error(Error::UnterminatedList);
                return Err(Break {});
            };

            if token.0 == delimiter {
                return Ok(output.join("\n"));
            } else {
                let s = self.format_expr(token)?;
                output.push(format!(
                    "{}{s}",
                    " ".repeat(self.nesting * self.indent_size)
                ));
            }
        }
    }

    pub fn format_dict(&mut self, delimiter: Token) -> Result<String, Break> {
        let mut output = String::new();

        loop {
            loop {
                let Some(token) = self.next_token() else {
                    // #TODO how to handle this?
                    self.push_error(Error::UnterminatedList);
                    return Err(Break {});
                };

                let cont = matches!(token.0, Token::Comment(..) | Token::Annotation(..));

                if token.0 == delimiter {
                    return Ok(output);
                } else {
                    let s = self.format_expr(token)?;
                    output.push_str(&format!(
                        "{}{s}",
                        " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
                    ));
                }

                if !cont {
                    break;
                }
            }

            loop {
                let Some(token) = self.next_token() else {
                    // #TODO how to handle this?
                    self.push_error(Error::UnterminatedList);
                    return Err(Break {});
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

    pub fn format_expr(&mut self, token: Ranged<Token>) -> Result<String, Break> {
        let Ranged(t, _) = token;

        let output = match t {
            Token::Comment(s) => s,
            Token::String(s) => format!("\"{s}\""),
            Token::Symbol(s) => s,
            Token::Number(s) => s,
            Token::Annotation(s) => {
                format!("#{s}")
            }
            Token::Quote => "'".to_owned(),
            Token::LeftParen => {
                // #TODO detect kind of expression and format accordingly!
                // #TODO we need lookahead.

                let Some(token) = self.next_token() else {
                    // #TODO how to handle this?
                    self.push_error(Error::UnterminatedList);
                    return Err(Break {});
                };

                if let Ranged(Token::Symbol(lexeme), _) = &token {
                    if lexeme == "do" {
                        // The tail terms are rendered vertically.
                        let mut s = "(do\n".to_string();
                        self.nesting += 1;
                        s.push_str(&self.format_list_vertical(Token::RightParen)?);
                        self.nesting -= 1;
                        s.push_str(&format!(
                            "\n{})",
                            " ".repeat(self.nesting * self.indent_size)
                        ));
                        s
                    } else if lexeme == "Func" || lexeme == "if" {
                        // The first tail term is rendered in same line, the
                        // rest are rendered vertically.
                        let mut s = format!("({lexeme} ");
                        let Some(token) = self.next_token() else {
                            // #TODO how to handle this?
                            self.push_error(Error::UnterminatedList);
                            return Err(Break {});
                        };
                        s.push_str(&format!("{}\n", self.format_expr(token)?));
                        self.nesting += 1;
                        s.push_str(&self.format_list_vertical(Token::RightParen)?);
                        self.nesting -= 1;
                        s.push_str(&format!(
                            "\n{})",
                            " ".repeat(self.nesting * self.indent_size)
                        ));
                        s
                    // #TODO custom 
                    // } else if lexeme == "let" {
                    } else {
                        self.put_back_token(token);

                        let mut s = "(".to_string();
                        s.push_str(&self.format_list_horizontal(Token::RightParen)?);
                        s.push(')');
                        s
                    }
                } else {
                    self.put_back_token(token);

                    let mut s = "(".to_string();
                    s.push_str(&self.format_list_horizontal(Token::RightParen)?);
                    s.push(')');
                    s
                }
            }
            Token::LeftBracket => {
                // Syntactic sugar for a List/Array.

                let mut s = "[\n".to_string();
                self.nesting += 1;
                s.push_str(&self.format_list_vertical(Token::RightBracket)?);
                self.nesting -= 1;
                s.push_str(&format!(
                    "\n{}]",
                    " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
                ));
                s
            }
            Token::LeftBrace => {
                // Syntactic sugar for a Dict.

                let mut s = "{\n".to_string();
                self.nesting += 1;
                s.push_str(&self.format_dict(Token::RightBrace)?);
                self.nesting -= 1;
                s.push_str(&format!(
                    "{}}}",
                    " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
                ));
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
            let Some(token) = self.next_token() else {
                break;
            };

            let Ok(s) = self.format_expr(token) else {
                // A non-recoverable parse error was detected, stop parsing.
                let errors = std::mem::take(&mut self.errors);
                return Err(errors);
            };

            output.push_str(&format!("{s}\n"));
        }

        Ok(output)
    }
}
