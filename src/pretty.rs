use std::collections::{BTreeMap, HashMap};

use tan::ann::Ann;
use tan::expr::Expr;
use tan::util::put_back_iterator::PutBackIterator;

use crate::{
    layout::{Arranger, Layout},
    util::{ensure_ends_with_empty_line, format_float},
};

// #TODO create intermediate representation before joining!
// #TODO align inline/side comments
// #TODO align vertical pairs (e.g. let)

// #TODO preprocess to handle inline comments?

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

fn apply_indent(s: &str, indent: usize) -> String {
    format!("{}{s}", " ".repeat(indent))
}

pub struct Formatter<'a> {
    arranger: Arranger<'a>,
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
            arranger: Arranger::new(exprs),
            indent_size: DEFAULT_INDENT_SIZE,
            line_size: DEFAULT_LINE_SIZE,
            col: 0,
        }
    }

    // pub fn format_horizontal(&mut self, exprs: &[Ann<Expr>]) -> String {
    //     let mut output: Vec<String> = Vec::new();

    //     for expr in exprs {
    //         output.push(self.format_expr(expr));
    //     }

    //     output.join(" ")
    // }

    // pub fn format_vertical(&mut self, exprs: &[Ann<Expr>]) -> String {
    //     let mut output: Vec<String> = Vec::new();

    //     for expr in exprs {
    //         match expr {
    //             Ann(Expr::TextSeparator, ..) => output.push("".to_owned()),
    //             // Ann(Expr::Comment(_, comment_kind), ..) => {
    //             //     println!("==== {:?}", expr);
    //             //     let s = match comment_kind {
    //             //         CommentKind::Inline => "*******".to_owned(),
    //             //         _ => self.format_expr(expr),
    //             //     };
    //             //     output.push(format!("{}{s}", " ".repeat(self.indent)));
    //             // }
    //             _ => {
    //                 let s = self.format_expr(expr);
    //                 output.push(format!("{}{s}", " ".repeat(self.indent)));
    //             }
    //         }
    //     }

    //     output.join("\n")
    // }

    // pub fn format_vertical_pairs(&mut self, exprs: &[Ann<Expr>]) -> String {
    //     let mut output: Vec<String> = Vec::new();

    //     let mut i = 0;

    //     while i < exprs.len() {
    //         let expr = &exprs[i];
    //         i += 1;
    //         match expr {
    //             Ann(Expr::TextSeparator, ..) => output.push("".to_owned()),
    //             _ => {
    //                 let key = expr;
    //                 let value = &exprs[i];
    //                 i += 1;
    //                 let value = self.format_expr(value);
    //                 output.push(format!("{}{key} {value}", " ".repeat(self.indent)));
    //             }
    //         }
    //     }

    //     output.join("\n")
    // }

    // fn format_annotations(&self, ann: &Option<HashMap<String, Expr>>) -> String {
    //     let Some(ann) = ann else {
    //         return "".to_string()
    //     };

    //     if ann.len() < 2 {
    //         return "".to_string();
    //     }

    //     // #TODO temp solution (sorts annotations by key), ideally we want insertion order? or not.

    //     // Sort the annotations map, for stable formatting.
    //     let ann = BTreeMap::from_iter(ann);

    //     let mut output = String::new();

    //     for (key, value) in ann {
    //         if key == "range" {
    //             continue;
    //         } else if let Expr::Bool(true) = value {
    //             // Abbreviation for true booleans.
    //             output.push_str(&format!("#{key} "));
    //         } else {
    //             // This case handles both (type X) and (key value) annotations.
    //             // The value is the whole expression.
    //             output.push_str(&format!("#{value} "));
    //         }
    //     }

    //     output
    // }

    // #TODO automatically put `_` separators to numbers.

    // pub fn format_expr(&mut self) -> Layout {
    //     let Ann(expr, ann) = expr;

    //     let layout = match expr {
    //         Expr::Comment(s, _) => Layout::Span(s.clone()),
    //         Expr::TextSeparator => Layout::Separator, // #TODO different impl!
    //         // #TODO maybe it's better to format annotations from Expr?
    //         // Expr::Annotation(s) => format!("#{s}"),
    //         Expr::String(s) => Layout::Span(format!("\"{s}\"")),
    //         Expr::Symbol(s) => Layout::Span(s.clone()),
    //         Expr::Int(n) => Layout::Span(n.to_string()),
    //         Expr::One => Layout::Span("()".to_string()),
    //         Expr::Bool(b) => Layout::Span(b.to_string()),
    //         Expr::Float(n) => Layout::Span(format_float(*n)),
    //         Expr::KeySymbol(s) => Layout::Span(format!(":{s}")),
    //         Expr::Char(c) => Layout::Span(format!(r#"(Char "{c}")"#)),
    //         Expr::List(exprs) => {
    //             if exprs.is_empty() {
    //                 return Layout::Span("()".to_owned());
    //             }

    //             // #insight Recursive data structure, we recurse.

    //             let list_formatter = Formatter::new(exprs);
    //             list_formatter.format_list()
    //         }
    //         _ => Layout::Span(expr.to_string()),
    //     };

    //     layout
    // }

    fn format_layout(&self, layout: &Layout, indent: usize) -> String {
        let mut output = String::new();

        match layout {
            Layout::Span(s) => s.clone(),
            Layout::List(v) => v
                .iter()
                .map(|l| self.format_layout(l, indent))
                .collect::<Vec<String>>()
                .join(""),
            Layout::HList(v) => v
                .iter()
                .map(|l| self.format_layout(l, indent))
                .collect::<Vec<String>>()
                .join(" "),
            Layout::VList(v) => v
                .iter()
                .map(|l| apply_indent(&self.format_layout(l, indent), indent))
                .collect::<Vec<String>>()
                .join("\n"),
            Layout::Indent(l) => self.format_layout(l, indent + self.indent_size),
            Layout::Separator => "\n".to_owned(),
            _ => "TODO".to_owned(),
        }
    }

    // #Insight
    // The formatter cannot err.

    /// Formats expressions into an aestheticall pleasing form.
    /// This is the standard textual representation of expressions.
    pub fn format(mut self) -> String {
        let layout = self.arranger.arrange();
        let output = self.format_layout(&layout, 0);
        let output = ensure_ends_with_empty_line(&output);

        output
    }
}
