use std::collections::{BTreeMap, HashMap};

use tan::ann::Ann;
use tan::expr::Expr;

use crate::util::{ensure_ends_with_empty_line, format_float};

// #TODO add pragmas to define sections with different formatting options or even disabled formatting.
// #TODO try to use annotations to define the above-mentioned sections.
// #TODO rename to `formatter.rs`
// #TODO optimize formatter to minimize diffs.
// #TODO try to maintain some empty separator lines.
// #TODO consider using tabs to indent?
// #TODO consider allowing absolutely no parameters for the formatter.
// #TODO idea: pre-process the input, add artificial separator-line annotations to maintain some of the user's separators?

/// The default indentation size (char count)
const DEFAULT_INDENT_SIZE: usize = 4;

/// The default (target) line size (char count)
const DEFAULT_LINE_SIZE: usize = 80;

pub struct Formatter<'a> {
    // #TODO no need to keep this!
    exprs: &'a [Ann<Expr>],
    indent: usize,
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
            indent: 0,
            indent_size: DEFAULT_INDENT_SIZE,
            line_size: DEFAULT_LINE_SIZE,
            col: 0,
        }
    }

    pub fn format_horizontal(&mut self, exprs: &[Ann<Expr>]) -> String {
        let mut output: Vec<String> = Vec::new();

        for expr in exprs {
            output.push(self.format_expr(expr));
        }

        output.join(" ")
    }

    pub fn format_vertical(&mut self, exprs: &[Ann<Expr>]) -> String {
        let mut output: Vec<String> = Vec::new();

        for expr in exprs {
            match expr {
                Ann(Expr::TextSeparator, ..) => output.push("".to_owned()),
                _ => {
                    let s = self.format_expr(expr);
                    output.push(format!("{}{s}", " ".repeat(self.indent)));
                }
            }
        }

        output.join("\n")
    }

    pub fn format_vertical_pairs(&mut self, exprs: &[Ann<Expr>]) -> String {
        let mut output: Vec<String> = Vec::new();

        let mut i = 0;

        while i < exprs.len() {
            let expr = &exprs[i];
            i += 1;
            match expr {
                Ann(Expr::TextSeparator, ..) => output.push("".to_owned()),
                _ => {
                    let key = expr;
                    let value = &exprs[i];
                    i += 1;
                    let value = self.format_expr(value);
                    output.push(format!("{}{key} {value}", " ".repeat(self.indent)));
                }
            }
        }

        output.join("\n")
    }

    fn format_annotations(&self, ann: &Option<HashMap<String, Expr>>) -> String {
        let Some(ann) = ann else {
            return "".to_string()
        };

        if ann.len() < 2 {
            return "".to_string();
        }

        // #TODO temp solution (sorts annotations by key), ideally we want insertion order? or not.

        // Sort the annotations map, for stable formatting.
        let ann = BTreeMap::from_iter(ann);

        let mut output = String::new();

        for (key, value) in ann {
            if key == "range" {
                continue;
            } else if let Expr::Bool(true) = value {
                // Abbreviation for true booleans.
                output.push_str(&format!("#{key} "));
            } else {
                // This case handles both (type X) and (key value) annotations.
                // The value is the whole expression.
                output.push_str(&format!("#{value} "));
            }
        }

        output
    }

    // #TODO automatically put `_` separators to numbers.

    pub fn format_expr(&mut self, expr: &Ann<Expr>) -> String {
        let Ann(expr, ann) = expr;

        let output = match expr {
            Expr::Comment(s) => s.clone(),
            Expr::TextSeparator => "".to_owned(),
            // #TODO maybe it's better to format annotations from Expr?
            // Expr::Annotation(s) => format!("#{s}"),
            Expr::String(s) => format!("\"{s}\""),
            Expr::Symbol(s) => s.clone(),
            Expr::Int(n) => n.to_string(),
            Expr::One => "()".to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Float(n) => format_float(*n),
            Expr::KeySymbol(s) => format!(":{s}"),
            Expr::Char(c) => format!(r#"(Char "{c}")"#),
            Expr::List(terms) => {
                if terms.is_empty() {
                    return "()".to_owned();
                }

                let (head, tail) = terms.split_at(1);

                let head = &head[0];
                // let tail: Vec<Expr> = tail.iter().map(|expr| expr.0.clone()).collect(); // #TODO argh, remove the clone!

                let head = self.format_expr(head);

                if head == "do" {
                    // The tail terms are rendered vertically.
                    let mut s = "(do\n".to_string();
                    self.indent += self.indent_size;
                    s.push_str(&self.format_vertical(tail));
                    self.indent -= self.indent_size;
                    s.push_str(&format!("\n{})", " ".repeat(self.indent)));
                    s
                } else if head == "Func" || head == "if" {
                    // The first tail term is rendered in same line, the
                    // rest are rendered vertically.
                    let mut s = format!("({head} ");

                    let (tail_first, tail_rest) = tail.split_at(1);

                    let tail_first = &tail_first[0];
                    s.push_str(&format!("{}\n", self.format_expr(tail_first)));

                    self.indent += self.indent_size;
                    s.push_str(&self.format_vertical(tail_rest));
                    self.indent -= self.indent_size;

                    s.push_str(&format!("\n{})", " ".repeat(self.indent)));

                    s
                } else if head == "Array" {
                    let mut s = "[\n".to_string();
                    self.indent += self.indent_size;
                    s.push_str(&self.format_vertical(tail));
                    self.indent -= self.indent_size;
                    s.push_str(&format!("\n{}]", " ".repeat(self.indent)));
                    s
                } else if head == "Dict" {
                    let mut s = "{\n".to_string();
                    self.indent += self.indent_size;
                    s.push_str(&self.format_vertical_pairs(tail));
                    self.indent -= self.indent_size;
                    s.push_str(&format!("\n{}}}", " ".repeat(self.indent)));
                    s
                } else if head == "let" {
                    let mut s = "(let ".to_string();

                    if tail.len() > 4 {
                        self.indent += 5; // indent = "(let ".len()
                        s.push_str(self.format_vertical_pairs(tail).trim_start());
                        self.indent -= 5;
                        s.push_str(&format!("\n{})", " ".repeat(self.indent)));
                    } else {
                        s.push_str(&self.format_horizontal(tail));
                        s.push(')');
                    }
                    s
                } else {
                    // let terms: Vec<Expr> = terms.iter().map(|expr| expr.0.clone()).collect(); // #TODO argh, remove the clone! -> use Ann<Expr> everywhere!
                    let mut s = "(".to_string();
                    s.push_str(&self.format_horizontal(terms));
                    s.push(')');
                    s
                }
            }
            _ => expr.to_string(),
        };

        format!("{}{output}", self.format_annotations(ann))
    }

    // #Insight
    // The formatter cannot err.

    /// Formats expressions into an aestheticall pleasing form.
    /// This is the standard textual representation of expressions.
    pub fn format(&mut self) -> String {
        let mut output: Vec<String> = Vec::new();

        // #TODO support look-ahead?
        for expr in self.exprs {
            output.push(self.format_expr(expr));
        }

        let output = output.join("\n");

        let output = ensure_ends_with_empty_line(&output);

        output
    }
}
