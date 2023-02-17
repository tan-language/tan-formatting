use std::collections::HashMap;

use tan::ann::Ann;
use tan::expr::Expr;

// #TODO add pragmas to define sections with different formatting options or even disabled formatting.
// #TODO try to use annotations to define the above-mentioned sections.
// #TODO this is the ugliest code ever written, wip.
// #TODO rename to `formatter.rs`
// #TODO optimize formatter to minimize diffs.
// #TODO try to maintain some empty separator lines.
// #TODO consider using tabs to indent?
// #TODO consider allowing absolutely no parameters for the formatter.

/// The default indentation size (char count)
const DEFAULT_INDENT_SIZE: usize = 4;

/// The default (target) line size (char count)
const DEFAULT_LINE_SIZE: usize = 80;

pub struct Formatter<'a> {
    // #TODO no need to keep this!
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
    pub fn format_horizontal(&mut self, exprs: &[Expr]) -> String {
        let mut output: Vec<String> = Vec::new();

        for expr in exprs {
            output.push(self.format_expr(expr));
        }

        output.join(" ")
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
                " ".repeat(self.nesting * self.indent_size)
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
            Expr::List(terms) => {
                if terms.is_empty() {
                    return "()".to_owned();
                }

                let (head, tail) = terms.split_at(1);

                let head = &head[0].0;
                let tail: Vec<Expr> = tail.iter().map(|expr| expr.0.clone()).collect(); // #TODO argh, remove the clone!

                let head = self.format_expr(&head);

                if head == "do" {
                    // The tail terms are rendered vertically.
                    let mut s = "(do\n".to_string();
                    self.nesting += 1;
                    s.push_str(&self.format_vertical(&tail));
                    self.nesting -= 1;
                    s.push_str(&format!(
                        "\n{})",
                        " ".repeat(self.nesting * self.indent_size)
                    ));
                    s
                } else if head == "Func" || head == "if" {
                    // The first tail term is rendered in same line, the
                    // rest are rendered vertically.
                    let mut s = format!("({head} ");

                    let (tail_first, tail_rest) = tail.split_at(1);

                    let tail_first = &tail_first[0];
                    s.push_str(&format!("{}\n", self.format_expr(tail_first)));

                    self.nesting += 1;
                    s.push_str(&self.format_vertical(tail_rest));
                    self.nesting -= 1;

                    s.push_str(&format!(
                        "\n{})",
                        " ".repeat(self.nesting * self.indent_size)
                    ));

                    s
                } else {
                    let terms: Vec<Expr> = terms.iter().map(|expr| expr.0.clone()).collect(); // #TODO argh, remove the clone! -> use Ann<Expr> everywhere!
                    let mut s = "(".to_string();
                    s.push_str(&self.format_horizontal(&terms));
                    s.push(')');
                    s
                }
            }
            Expr::Array(items) => {
                let mut s = "[\n".to_string();
                self.nesting += 1;
                s.push_str(&self.format_vertical(items));
                self.nesting -= 1;
                s.push_str(&format!(
                    "\n{}]",
                    " ".repeat(self.nesting * self.indent_size)
                ));
                s
            }
            Expr::Dict(dict) => {
                // #TODO argh! insertion order is not kept! must change parser!
                let mut s = "{\n".to_string();
                self.nesting += 1;
                s.push_str(&self.format_dict(dict));
                self.nesting -= 1;
                s.push_str(&format!(
                    "\n{}}}",
                    " ".repeat(self.nesting * self.indent_size)
                ));
                s
            }
            _ => expr.to_string(),
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

        let output = output.join("\n");

        // Add an empty line at the end, as a standard practice.
        format!("{output}\n")
    }
}
