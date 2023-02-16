use std::collections::HashMap;

use tan::ann::Ann;
use tan::expr::Expr;

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

pub struct Formatter<'a> {
    exprs: &'a [Ann<Expr>],
    nesting: usize,
    indent_size: usize,
    #[allow(dead_code)]
    line_size: usize,
    #[allow(dead_code)]
    col: usize,
}

// #TODO introduce default constructor.
// #TODO introduce 'builder' api?

impl<'a> Formatter<'a> {
    pub fn new(exprs: &'a [Ann<Expr>]) -> Self {
        Self {
            exprs,
            nesting: 0,
            indent_size: DEFAULT_INDENT_SIZE,
            line_size: DEFAULT_LINE_SIZE,
            col: 0,
        }
    }

    // #TODO find better name
    pub fn format_vertical(&mut self, exprs: &[Expr]) -> String {
        let mut output: Vec<String> = Vec::new();

        for expr in exprs {
            let s = self.format_expr(expr);
            output.push(format!(
                "{}{s}",
                " ".repeat(self.nesting * self.indent_size)
            ));
        }

        output.join("\n")
    }

    // #TODO rename to format_pairs.
    pub fn format_dict(&mut self, dict: &HashMap<String, Expr>) -> String {
        let mut output: Vec<String> = Vec::new();

        for (key, value) in dict {
            let key = format!(":{key}"); // #TODO temp solution!
            let value = self.format_expr(value);

            output.push(format!(
                "{}{key} {value}",
                " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
            ));
        }

        output.join("\n")
    }

    // #TODO automatically put `_` separators to numbers.

    pub fn format_expr(&mut self, expr: &Expr) -> String {
        let output = match expr {
            Expr::Comment(s) => s.clone(),
            // Expr::Annotation(s) => format!("#{s}"),
            Expr::String(s) => format!("\"{s}\""),
            Expr::Symbol(s) => s.clone(),
            Expr::Int(n) => n.to_string(),
            Expr::One => "()".to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Float(n) => n.to_string(),
            Expr::KeySymbol(s) => format!(":{s}"),
            Expr::Char(c) => format!(r#"(Char "{c}")"#),
            Expr::List(_) => todo!(),
            Expr::Array(items) => {
                let mut s = "[\n".to_string();
                self.nesting += 1;
                s.push_str(&self.format_vertical(items));
                self.nesting -= 1;
                s.push_str(&format!(
                    "\n{}]",
                    " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
                ));
                s
            }
            Expr::Dict(dict) => {
                let mut s = "{\n".to_string();
                self.nesting += 1;
                s.push_str(&self.format_dict(dict));
                self.nesting -= 1;
                s.push_str(&format!(
                    "\n{}}}",
                    " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
                ));
                s
            }
            // #TODO no need to format the remaining.
            Expr::Func(_, _) => todo!(),
            Expr::Macro(_, _) => todo!(),
            Expr::ForeignFunc(_) => todo!(),
            Expr::Do => todo!(),
            Expr::Let => todo!(),
            Expr::If(_, _, _) => todo!(),
            // Expr::Annotation(s) => {
            //     format!("#{s}")
            // }
            // Token::Quote => "'".to_owned(),
            // Token::LeftParen => {
            //     // #TODO detect kind of expression and format accordingly!
            //     // #TODO we need lookahead.

            //     let Some(token) = self.next_token() else {
            //         // #TODO how to handle this?
            //         self.push_error(Error::UnterminatedList);
            //         return Err(Break {});
            //     };

            //     if let Ranged(Token::Symbol(lexeme), _) = &token {
            //         if lexeme == "do" {
            //             // The tail terms are rendered vertically.
            //             let mut s = "(do\n".to_string();
            //             self.nesting += 1;
            //             s.push_str(&self.format_list_vertical(Token::RightParen)?);
            //             self.nesting -= 1;
            //             s.push_str(&format!(
            //                 "\n{})",
            //                 " ".repeat(self.nesting * self.indent_size)
            //             ));
            //             s
            //         } else if lexeme == "Func" || lexeme == "if" {
            //             // The first tail term is rendered in same line, the
            //             // rest are rendered vertically.
            //             let mut s = format!("({lexeme} ");
            //             let Some(token) = self.next_token() else {
            //                 // #TODO how to handle this?
            //                 self.push_error(Error::UnterminatedList);
            //                 return Err(Break {});
            //             };
            //             s.push_str(&format!("{}\n", self.format_expr(token)?));
            //             self.nesting += 1;
            //             s.push_str(&self.format_list_vertical(Token::RightParen)?);
            //             self.nesting -= 1;
            //             s.push_str(&format!(
            //                 "\n{})",
            //                 " ".repeat(self.nesting * self.indent_size)
            //             ));
            //             s
            //         // #TODO custom
            //         // } else if lexeme == "let" {
            //         } else {
            //             self.put_back_token(token);

            //             let mut s = "(".to_string();
            //             s.push_str(&self.format_list_horizontal(Token::RightParen)?);
            //             s.push(')');
            //             s
            //         }
            //     } else {
            //         self.put_back_token(token);

            //         let mut s = "(".to_string();
            //         s.push_str(&self.format_list_horizontal(Token::RightParen)?);
            //         s.push(')');
            //         s
            //     }
            // }
            // Token::LeftBracket => {
            //     // Syntactic sugar for a List/Array.

            //     let mut s = "[\n".to_string();
            //     self.nesting += 1;
            //     s.push_str(&self.format_list_vertical(Token::RightBracket)?);
            //     self.nesting -= 1;
            //     s.push_str(&format!(
            //         "\n{}]",
            //         " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
            //     ));
            //     s
            // }
            // Token::LeftBrace => {
            //     // Syntactic sugar for a Dict.

            //     let mut s = "{\n".to_string();
            //     self.nesting += 1;
            //     s.push_str(&self.format_dict(Token::RightBrace)?);
            //     self.nesting -= 1;
            //     s.push_str(&format!(
            //         "{}}}",
            //         " ".repeat(self.nesting * DEFAULT_INDENT_SIZE)
            //     ));
            //     s
            // }
            // Token::RightParen | Token::RightBracket | Token::RightBrace => {
            //     // #TODO custom error for this?
            //     self.push_error(Error::UnexpectedToken(t));
            //     // Parsing can continue.
            //     return Ok("".to_owned());
            // }
        };

        output
    }

    // #Insight
    // The formatter cannot err.

    /// Formats expressions into an aestheticall pleasing form.
    /// This is the standard textual representation of expressions.
    pub fn format(&mut self) -> String {
        let mut output: Vec<String> = Vec::new();

        for expr in self.exprs {
            output.push(self.format_expr(&expr.0));
        }

        output.join("\n")
    }
}
