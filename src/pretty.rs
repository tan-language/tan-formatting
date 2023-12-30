use std::collections::{BTreeMap, HashMap};

use tan::expr::Expr;

use crate::{
    layout::{Arranger, Layout},
    util::{ensure_ends_with_empty_line, trim_separators},
};

// #insight The formatter cannot err.

// #todo align inline/side comments
// #todo align vertical pairs (e.g. let)

// #todo preprocess to handle inline comments?

// #todo add pragmas to define sections with different formatting options or even disabled formatting.
// #todo try to use annotations to define the above-mentioned sections.
// #todo rename to `formatter.rs`
// #todo optimize formatter to minimize diffs.
// #todo try to maintain some empty separator lines.
// #todo consider using tabs to indent?
// #todo consider allowing absolutely no parameters for the formatter.
// #todo idea: pre-process the input, add artificial separator-line annotations to maintain some of the user's separators?

/// The default indentation size (char count)
const DEFAULT_INDENT_SIZE: usize = 4;

/// The default (target) line size (char count)
const DEFAULT_LINE_SIZE: usize = 80;

pub struct Formatter<'a> {
    arranger: Arranger<'a>,
    indent: usize,
    indent_size: usize,
    #[allow(dead_code)]
    line_size: usize,
    #[allow(dead_code)]
    col: usize,
}

// #todo introduce default constructor.
// #todo introduce 'builder' api?

impl<'a> Formatter<'a> {
    pub fn new(exprs: &'a [Expr]) -> Self {
        Self {
            arranger: Arranger::new(exprs),
            indent: 0,
            indent_size: DEFAULT_INDENT_SIZE,
            line_size: DEFAULT_LINE_SIZE,
            col: 0,
        }
    }

    fn apply_indent(&self, s: String, indent: usize) -> String {
        format!("{ }{s}", " ".repeat(indent))
    }

    fn format_annotations(&self, ann: &HashMap<String, Expr>) -> String {
        if ann.is_empty() {
            return "".to_string();
        }

        // #todo temp solution (sorts annotations by key), ideally we want insertion order? or not?

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

    // #todo automatically put `_` separators to numbers.

    fn format_layout(&mut self, layout: &Layout) -> String {
        match layout {
            Layout::Item(s) => s.clone(),
            Layout::Row(v, separator) => v
                .iter()
                .map(|l| self.format_layout(l))
                .collect::<Vec<String>>()
                .join(separator),
            Layout::Stack(v) => v
                .iter()
                .map(|l| self.format_layout(l))
                .collect::<Vec<String>>()
                .join("\n"),
            Layout::Indent(v, indent_size) => {
                let indent_size = indent_size.unwrap_or(self.indent_size);
                self.indent += indent_size;
                let string = v
                    .iter()
                    .map(|l| {
                        let string = self.format_layout(l);
                        self.apply_indent(string, self.indent)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                self.indent -= indent_size;
                string
            }
            Layout::Apply(l) => {
                let string = self.format_layout(l);
                self.apply_indent(string, self.indent)
            }
            // Layout::Indent(l, indent_size) => {
            //     let indent_size = indent_size.unwrap_or(self.indent_size);
            //     self.indent += indent_size;
            //     let string = self.format_layout(l, self.indent);
            //     self.indent -= indent_size;
            //     string
            // }
            Layout::Ann(ann, l) => {
                let ann = self.format_annotations(ann);
                let string = self.format_layout(l);
                format!("{ann}{string}")
            }
            Layout::Separator => "".to_owned(),
        }
    }

    /// Formats expressions into an aestheticall pleasing form.
    /// This is the standard textual representation of expressions.
    pub fn format(mut self) -> String {
        let layout = self.arranger.arrange();
        // eprintln!("{:?}", &layout);
        // dbg!(&layout);
        let output = self.format_layout(&layout);
        let output = trim_separators(&output);
        let output = ensure_ends_with_empty_line(&output);
        output
    }
}
