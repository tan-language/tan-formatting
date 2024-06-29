use std::collections::HashMap;

use tan::{
    expr::Expr,
    util::{fmt::format_float, put_back_iterator::PutBackIterator},
};

use crate::{types::Dialect, util::escape_string};

// #todo use source-code annotations to control formatting

// #todo add some explanation about the design, e.g. what does Layout do.

// #todo somehow extract the force_vertical computation to include all parameters.

// #todo conds get corrupted
// #todo remove empty lines from beginning of blocks!
// #todo implement `html` and `css` dialects

// #todo refine this enum, potentially split into 2 enums?
// #todo could name this layout 'Cell' or Fragment
/// A Layout is an abstract representation (model) of formatted source.
#[derive(Clone, Debug)]
pub enum Layout {
    /// Indentation block, supports both indentation and alignment.
    Indent(Vec<Layout>, Option<usize>), // #todo no need for Indent, add option to stack
    /// Vertical arrangement
    Stack(Vec<Layout>),
    /// Horizontal arrangement
    Row(Vec<Layout>, String),
    // #todo wtf is this?
    Apply(Box<Layout>),
    Item(String),
    Ann(HashMap<String, Expr>, Box<Layout>),
    Separator,
}

impl Layout {
    pub fn indent(list: Vec<Layout>) -> Self {
        Self::Indent(list, None)
    }

    pub fn align(list: Vec<Layout>, indent_size: usize) -> Self {
        Self::Indent(list, Some(indent_size))
    }

    pub fn row(list: impl Into<Vec<Layout>>) -> Self {
        Self::Row(list.into(), " ".to_string())
    }

    pub fn join(list: impl Into<Vec<Layout>>) -> Self {
        Self::Row(list.into(), "".to_string())
    }

    pub fn apply(l: Layout) -> Self {
        Self::Apply(Box::new(l))
    }

    pub fn item(s: impl Into<String>) -> Self {
        Self::Item(s.into())
    }

    pub fn space() -> Self {
        Self::Item(" ".into())
    }
}

// #todo should allow for multiple concurrent modes?
/// An arranger mode to allow for formatting specializations.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum ArrangerMode {
    Default,
    // #todo document what this does.
    // #todo find a better name, more encompassing.
    Let,
    // Forces inline alignment, for example for function arguments.
    Inline,
}

// #todo should allow for multiple modes, make a Set.
// #todo maybe Dialects should just get expanded to modes?

// #todo find a better name.
/// The Arranger organizes the input expressions into an abstract Layout. The
/// Formatter renders the Layout model into the formatted output string.
pub struct Arranger<'a> {
    // #todo consider different names, e.g. `flavor`?
    // #todo use a builder pattern.
    pub dialect: Dialect,
    exprs: PutBackIterator<'a, Expr>,
    mode: ArrangerMode,
}

impl<'a> Arranger<'a> {
    pub fn new(exprs: &'a [Expr], dialect: Dialect) -> Self {
        Self {
            dialect,
            exprs: PutBackIterator::new(exprs),
            mode: ArrangerMode::Default,
        }
    }

    fn arrange_next(&mut self) -> Option<Layout> {
        let expr0 = self.exprs.next()?;

        let layout = self.layout_from_expr(expr0);

        // #insight
        // Fetch the next expression and try to detect an inline comment.
        // If an inline comment is found, force vertical layout.
        if let Some(expr1) = self.exprs.next() {
            match expr1.unpack() {
                Expr::Comment(..) => {
                    if expr1.range().unwrap().start.line == expr0.range().unwrap().start.line {
                        let comment = self.layout_from_expr(expr1);
                        return Some(Layout::row(vec![layout, comment]));
                    } else {
                        self.exprs.put_back(expr1);
                    }
                }
                _ => {
                    self.exprs.put_back(expr1);
                }
            }
        };

        Some(layout)
    }

    fn arrange_all(&mut self) -> (Vec<Layout>, bool) {
        let mut layouts = Vec::new();

        let mut force_vertical = false;

        while let Some(layout) = self.arrange_next() {
            // force vertical if there is an inline comment.
            if let Layout::Row(v, ..) = &layout {
                if let Some(Layout::Item(t)) = &v.last() {
                    force_vertical = force_vertical || t.starts_with(';'); // is comment?
                }
            };

            // #todo make a constant, find a good threshold value.
            // #todo compute from max_line_len?
            // let item_length_vertical_arrange_threshold = 8;

            // force vertical if there is a full-line comment.
            // force vertical if an item length exceeds a threshold.
            if let Layout::Item(item) = &layout {
                force_vertical = force_vertical || item.starts_with(';') // is comment?
                                                                         // force_vertical = force_vertical
                                                                         // || item.starts_with(';') // is comment?
                                                                         // || item.len() > item_length_vertical_arrange_threshold; // is long item?
            }

            layouts.push(layout);
        }

        (layouts, force_vertical)
    }

    // #insight this is problematic for function parameter arrays!
    // #todo temp specialize arrange_all for arrays.
    fn arrange_all_array(&mut self) -> (Vec<Layout>, bool) {
        let mut layouts = Vec::new();

        let mut force_vertical = false;

        let mut items_cumulative_length = 0;

        while let Some(layout) = self.arrange_next() {
            // force vertical if there is an inline comment.
            if let Layout::Row(v, ..) = &layout {
                if let Some(Layout::Item(t)) = &v.last() {
                    force_vertical = force_vertical || t.starts_with(';'); // is comment?
                }
            };

            // #todo make a constant, find a good threshold value.
            // #todo compute from max_line_len?
            let item_length_vertical_arrange_threshold = 8;

            // force vertical if there is a full-line comment.
            // force vertical if an item length exceeds a threshold.
            if let Layout::Item(item) = &layout {
                items_cumulative_length += item.len();

                force_vertical = force_vertical
                    || item.starts_with(';') // is comment?
                    || (self.mode != ArrangerMode::Inline && item.len() > item_length_vertical_arrange_threshold);
                // is long item?
            }

            layouts.push(layout);
        }

        if self.mode != ArrangerMode::Inline {
            // #todo find a good threshold!
            // #todo also support wrapping to next line.
            force_vertical = force_vertical || items_cumulative_length > 32;
        }

        (layouts, force_vertical)
    }

    // #todo Find good name.
    fn maybe_annotated_layout_from_expr(&mut self, expr: &Expr) -> Option<Layout> {
        // Try to bundle all annotations into one Row (span).

        // let mut expr = self.exprs.next()?;
        let mut expr = expr;

        let mut annotated = Vec::new();

        while let Expr::Annotation(..) = expr.unpack() {
            annotated.push(self.layout_from_expr(expr));
            expr = self.exprs.next()?;
        }

        // #todo Pretty-print the value/payload of the annotation.

        if annotated.is_empty() {
            Some(self.layout_from_expr(expr))
        } else {
            annotated.push(self.layout_from_expr(expr));
            Some(Layout::row(annotated))
        }
    }

    // #todo add doc-comment.
    fn arrange_next_pair(&mut self) -> Option<Layout> {
        // #todo Add unit-test just for this method.

        let expr = self.exprs.next()?;

        // #insight Handles (skips) full line comments.
        // #todo Needs more elegant solution.
        if let Expr::Comment(..) = expr.unpack() {
            return Some(self.layout_from_expr(expr));
        }

        let mut tuple = Vec::new();

        tuple.push(self.maybe_annotated_layout_from_expr(expr)?);

        let expr = self.exprs.next()?;
        tuple.push(self.maybe_annotated_layout_from_expr(expr)?);

        // Try to skip trailing comments.
        if let Some(expr) = self.exprs.next() {
            match expr.unpack() {
                Expr::Comment(..) => {
                    if expr.range().unwrap().start.line == expr.range().unwrap().start.line {
                        tuple.push(self.layout_from_expr(expr));
                    } else {
                        self.exprs.put_back(expr);
                    }
                }
                _ => {
                    self.exprs.put_back(expr);
                }
            }
        };

        Some(Layout::row(tuple))
    }

    fn arrange_all_pairs(&mut self) -> (Vec<Layout>, bool) {
        let mut layouts = Vec::new();

        let mut should_force_vertical = false;

        while let Some(layout) = self.arrange_next_pair() {
            if let Layout::Row(items, ..) = &layout {
                if items.len() > 2 {
                    // If a pair has an inline comments, force vertical layout
                    should_force_vertical = true;
                }
            };

            layouts.push(layout);
        }

        (layouts, should_force_vertical)
    }

    fn arrange_list(&mut self) -> Layout {
        // #insight not need to check here.
        let expr = self.exprs.next().unwrap();

        let mut layouts = Vec::new();

        let head = expr.unpack();

        // #todo should decide between (h)list/vlist.
        // #todo special formatting for `if`.

        // #todo #warning (Func [...] ...) generate an Expr::Type("Func") !!

        match head {
            Expr::Symbol(name) if name == "quot" => {
                // #todo this is a temp solution, ideally it should recourse into arrange_list again.
                // Always arrange a `quot` block horizontally.
                let (exprs, _) = self.arrange_all();
                layouts.push(Layout::item("'"));
                layouts.push(Layout::row(exprs));
                Layout::join(layouts)
            }
            Expr::Symbol(name) if name == "unquot" => {
                // Always arrange a `unquot` block horizontally.
                let (exprs, _) = self.arrange_all();
                layouts.push(Layout::item("$"));
                layouts.push(Layout::row(exprs));
                Layout::join(layouts)
            }
            Expr::Symbol(name) if name == "do" => {
                // Always arrange a `do` block vertically.
                let (exprs, _) = self.arrange_all();
                layouts.push(Layout::item("(do"));
                layouts.push(Layout::indent(exprs));
                layouts.push(Layout::apply(Layout::item(")")));
                Layout::Stack(layouts)
            }
            // #todo #hack super nasty way to handle both Symbol and Type.
            // #todo #warning (Func [...] ...) generate an Expr::Type("Func") !!
            Expr::Symbol(name) | Expr::Type(name)
                if name == "if" || name == "for" || name == "Func" =>
            {
                // The first expr is rendered inline, the rest are rendered vertically.
                layouts.push(Layout::row(vec![
                    Layout::item(format!("({name}")),
                    // #todo special handling for `for` also needed, separate from Func.
                    // #todo could set mode here!
                    // #todo #hack nasty, refactor!
                    if name == "Func" || name == "for" {
                        let old_mode = self.mode;
                        self.mode = ArrangerMode::Inline;
                        let layout = self.arrange_next().unwrap();
                        self.mode = old_mode;
                        layout
                    } else {
                        self.arrange_next().unwrap()
                    },
                ]));
                let (block, should_force_vertical) = self.arrange_all();

                // #todo consider making `if` always multiline? no.

                let should_force_vertical = should_force_vertical || block.len() > 1;

                // #todo reconsider forced-multiline for `for`.
                // #insight
                // force `for`, to always be multiline, as it doesn't return a
                // useful value.
                let should_force_vertical = should_force_vertical || name == "for";

                let should_force_vertical = should_force_vertical || self.mode == ArrangerMode::Let;

                if should_force_vertical {
                    layouts.push(Layout::indent(block));
                    layouts.push(Layout::apply(Layout::item(")")));
                    Layout::Stack(layouts)
                } else {
                    layouts.push(Layout::item(" "));
                    layouts.push(block[0].clone());
                    layouts.push(Layout::item(")"));
                    Layout::join(layouts)
                }
            }
            Expr::Symbol(name) if name == "Range" => {
                // #todo support open-ended ranges.
                // safe to unwrap, it's already parsed.
                let start = self.exprs.next().unwrap();
                let end = self.exprs.next().unwrap();
                let mut range = format!("{start}..{end}");
                if let Some(step) = self.exprs.next() {
                    range = format!("{range}|{step}");
                }
                Layout::Item(range)
            }
            Expr::Symbol(name) if name == "Array" => {
                // #todo more sophisticated Array formatting needed.
                // Try to format the array horizontally.
                layouts.push(Layout::item("["));
                // let (items, should_force_vertical) = self.arrange_all();
                let (items, should_force_vertical) = self.arrange_all_array();

                // #todo consider allowing horizontal for only one element.
                // For `data` dialect always force vertical.
                let should_force_vertical = should_force_vertical || self.dialect == Dialect::Data;

                if !items.is_empty() {
                    if should_force_vertical {
                        layouts.push(Layout::indent(items));
                        layouts.push(Layout::apply(Layout::item("]")));
                        Layout::Stack(layouts)
                    } else {
                        match &items[0] {
                            // Heuristic: if the array includes stacks, arrange
                            // vertically.
                            Layout::Stack(..) | Layout::Indent(..) => {
                                layouts.push(Layout::indent(items));
                                layouts.push(Layout::apply(Layout::item("]")));
                                Layout::Stack(layouts)
                            }
                            _ => {
                                layouts.push(Layout::row(items));
                                layouts.push(Layout::item("]"));
                                Layout::join(layouts)
                            }
                        }
                    }
                } else {
                    layouts.push(Layout::item("]"));
                    Layout::join(layouts)
                }
            }
            Expr::Symbol(name) if name == "Map" => {
                // #todo in data mode consider formatting empty Map like this: {}
                let (bindings, should_force_vertical) = self.arrange_all_pairs();

                // If more than 2 bindings force vertical.
                let should_force_vertical = should_force_vertical || bindings.len() > 2;

                // For `data` dialect always force vertical.
                let should_force_vertical = should_force_vertical || self.dialect == Dialect::Data;

                if should_force_vertical {
                    layouts.push(Layout::item("{"));
                    layouts.push(Layout::indent(bindings));
                    layouts.push(Layout::apply(Layout::item("}")));
                    Layout::Stack(layouts)
                } else {
                    layouts.push(Layout::item("{"));
                    layouts.push(Layout::row(bindings));
                    layouts.push(Layout::item('}'));
                    Layout::join(layouts)
                }
            }
            Expr::Symbol(name) if name == "let" => {
                // #todo add a more intuitive mechanism for mode, maybe a stack?
                let old_mode = self.mode;
                self.mode = ArrangerMode::Let;
                let (mut bindings, should_force_vertical) = self.arrange_all_pairs();

                self.mode = old_mode;

                if should_force_vertical {
                    // Special case: one binding with inline comment, arrange vertically.
                    layouts.push(Layout::item("(let"));
                    layouts.push(Layout::indent(bindings));
                    layouts.push(Layout::apply(Layout::item(')')));
                    Layout::Stack(layouts)
                } else if bindings.len() > 1 {
                    // More than one binding, arrange vertically.
                    layouts.push(Layout::row(vec![Layout::item("(let"), bindings.remove(0)]));
                    if !bindings.is_empty() {
                        layouts.push(Layout::align(bindings, 5 /* "(let " */));
                    }
                    layouts.push(Layout::apply(Layout::item(')')));
                    Layout::Stack(layouts)
                } else {
                    // One binding, arrange horizontally.
                    layouts.push(Layout::item("(let "));
                    layouts.push(Layout::row(bindings));
                    layouts.push(Layout::item(')'));
                    Layout::join(layouts)
                }
            }
            // #todo currently this is exactly the same code as for `let`, extract.
            // #todo hmm not exactly the same, always forces multiline!
            Expr::Symbol(name) if name == "cond" => {
                let (clauses, should_force_vertical) = self.arrange_all_pairs();

                if should_force_vertical {
                    // #todo not relevant for `cond`, remove!
                    // Special case: one clause with inline comment, arrange vertically.
                    layouts.push(Layout::item("(cond"));
                    layouts.push(Layout::indent(clauses));
                    layouts.push(Layout::apply(Layout::item(')')));
                    Layout::Stack(layouts)
                } else if clauses.len() > 1 {
                    // More than one clause, arrange vertically.
                    layouts.push(Layout::item("(cond"));
                    // layouts.push(Layout::row(vec![Layout::item("(cond"), bindings.remove(0)]));
                    if !clauses.is_empty() {
                        layouts.push(Layout::align(clauses, 4 /* "(cond " */));
                    }
                    layouts.push(Layout::apply(Layout::item(')')));
                    Layout::Stack(layouts)
                } else {
                    // #todo there should never be one clause, remove!
                    // One clause, arrange horizontally.
                    layouts.push(Layout::item("(cond "));
                    layouts.push(Layout::row(clauses));
                    layouts.push(Layout::item(')'));
                    Layout::join(layouts)
                }
            }
            _ => {
                // Function call.
                layouts.push(Layout::item(format!("({head}")));
                let (args, should_force_vertical) = self.arrange_all();
                if !args.is_empty() {
                    if should_force_vertical {
                        layouts.push(Layout::indent(args));
                        layouts.push(Layout::apply(Layout::item(")")));
                        Layout::Stack(layouts)
                    } else {
                        layouts.push(Layout::item(" "));
                        layouts.push(Layout::row(args));
                        layouts.push(Layout::item(")"));
                        Layout::join(layouts)
                    }
                } else {
                    layouts.push(Layout::item(")"));
                    Layout::join(layouts)
                }
            }
        }
    }

    fn layout_from_expr(&self, expr: &Expr) -> Layout {
        let (expr, _ann) = expr.extract();

        let layout = match expr {
            Expr::Comment(s, _) => Layout::Item(s.clone()),
            Expr::TextSeparator => Layout::Separator, // #todo different impl!
            Expr::String(s) => Layout::Item(format!("\"{}\"", escape_string(s))),
            Expr::Symbol(s) => Layout::Item(s.clone()),
            Expr::Int(n) => Layout::Item(n.to_string()),
            // #insight `()` is the single instance of the Unit type `Nil`.
            Expr::None => Layout::Item("()".to_string()),
            Expr::Bool(b) => Layout::Item(b.to_string()),
            Expr::Float(n) => Layout::Item(format_float(*n)),
            Expr::KeySymbol(s) => Layout::Item(format!(":{s}")),
            Expr::Char(c) => Layout::Item(format!(r#"(Char "{c}")"#)),
            // #todo should handle Array?!
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Layout::Item("()".to_owned());
                }

                // #insight Recursive data structure, we recurse.

                let mut list_arranger = Arranger::new(exprs, self.dialect);
                list_arranger.mode = self.mode;
                list_arranger.arrange_list()
            }
            _ => Layout::Item(expr.to_string()),
        };

        // if let Some(ann) = ann {
        //     if ann.len() > 1 {
        //         // #todo give special key to implicit range annotation.
        //         // Remove the range annotation.
        //         let mut ann = ann.clone();
        //         ann.remove("range");
        //         return Layout::Ann(ann, Box::new(layout));
        //     }
        // }

        layout
    }

    pub fn arrange(&mut self) -> Layout {
        let (rows, _) = self.arrange_all();
        Layout::Stack(rows)
    }
}
