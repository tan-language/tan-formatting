use std::collections::{BTreeMap, HashMap};

use tan::ann::Ann;
use tan::expr::Expr;

use crate::{
    layout::{Arranger, Layout},
    util::ensure_ends_with_empty_line,
};

// #insight The formatter cannot err.

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

    fn format_annotations(&self, ann: &HashMap<String, Expr>) -> String {
        if ann.len() < 2 {
            return "".to_string();
        }

        // #TODO temp solution (sorts annotations by key), ideally we want insertion order? or not?

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

    fn format_layout(&self, layout: &Layout, indent: usize) -> String {
        match layout {
            Layout::Span(s) => s.clone(),
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
            Layout::Ann(ann, l) => format!(
                "{}{}",
                self.format_annotations(ann),
                self.format_layout(l, indent)
            ),
            Layout::Separator => "".to_owned(),
        }
    }

    /// Formats expressions into an aestheticall pleasing form.
    /// This is the standard textual representation of expressions.
    pub fn format(mut self) -> String {
        let layout = self.arranger.arrange();
        eprintln!("{:?}", &layout);
        let output = self.format_layout(&layout, 0);
        let output = ensure_ends_with_empty_line(&output);
        output
    }
}
