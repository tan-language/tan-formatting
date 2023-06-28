use std::collections::HashMap;

use tan::{ann::Ann, expr::Expr, util::put_back_iterator::PutBackIterator};

use crate::util::format_float;

// #TODO resolve ident + correct Layout structure.

// #TODO precisely delineate 'line', 'span'
// #TODO HList, Join, etc can not have layouts!
// #TODO extract Line enum, Fragment { Span, Ann }
// #TODO Layout { Indent(Box<Layout>), Block(Vec<Span>), Ann(HashMap<String, Expr>, Box<Layout>) }, Span { Text(String), Separator, Ann(Hashmap, Box<Span>) }
// span helper constructors (joined, separated, new)
// replace separator with Span?

// #TODO could name this layout 'Cell' or Fragment
#[derive(Clone, Debug)]
pub enum Layout {
    /// Indentation block, supports both indentation and alignment.
    Indent(Vec<Layout>, Option<usize>), // #TODO no need for Indent, add option to stack
    /// Vertical arrangement
    Stack(Vec<Layout>),
    /// Horizontal arrangement
    Row(Vec<Layout>, String),
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

// #TODO find a better name.
pub struct Arranger<'a> {
    exprs: PutBackIterator<'a, Ann<Expr>>,
}

impl<'a> Arranger<'a> {
    pub fn new(exprs: &'a [Ann<Expr>]) -> Self {
        Self {
            exprs: PutBackIterator::new(exprs),
        }
    }

    fn arrange_next(&mut self) -> Option<Layout> {
        let Some(expr0) = self.exprs.next() else {
            return None;
        };

        let layout = self.layout_from_expr(expr0);

        if let Some(expr1) = self.exprs.next() {
            match expr1 {
                Ann(Expr::Comment(..), _) => {
                    if expr1.get_range().unwrap().start.line
                        == expr0.get_range().unwrap().start.line
                    {
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

    // #TODO return force_vertical
    fn arrange_all(&mut self) -> (Vec<Layout>, bool) {
        let mut layouts = Vec::new();

        let mut force_vertical = false;

        while let Some(layout) = self.arrange_next() {
            if let Layout::Row(v, ..) = &layout {
                if let Some(Layout::Item(t)) = &v.last() {
                    force_vertical = t.starts_with(";"); // is comment?
                }
            };

            layouts.push(layout);
        }

        (layouts, force_vertical)
    }

    fn arrange_next_pair(&mut self) -> Option<Layout> {
        let mut tuple = Vec::new();

        let Some(expr0) = self.exprs.next() else {
            return None;
        };

        tuple.push(self.layout_from_expr(expr0));

        let Some(expr1) = self.exprs.next() else {
            return None;
        };

        tuple.push(self.layout_from_expr(expr1));

        if let Some(expr2) = self.exprs.next() {
            match expr2 {
                Ann(Expr::Comment(..), _) => {
                    if expr2.get_range().unwrap().start.line
                        == expr0.get_range().unwrap().start.line
                    {
                        tuple.push(self.layout_from_expr(expr2));
                    } else {
                        self.exprs.put_back(expr2);
                    }
                }
                _ => {
                    self.exprs.put_back(expr2);
                }
            }
        };

        Some(Layout::row(tuple))
    }

    fn arrange_all_pairs(&mut self) -> (Vec<Layout>, bool) {
        let mut layouts = Vec::new();

        let mut force_vertical = false;

        while let Some(layout) = self.arrange_next_pair() {
            if let Layout::Row(items, ..) = &layout {
                if items.len() > 2 {
                    // If a pair has an inline comments, force vertical layout
                    force_vertical = true;
                }
            };

            layouts.push(layout);
        }

        (layouts, force_vertical)
    }

    fn arrange_list(&mut self) -> Layout {
        // #insight not need to check here.
        let expr = self.exprs.next().unwrap();

        let mut layouts = Vec::new();

        let head = &expr.0;

        // #TODO should decide between (h)list/vlist.
        // #TODO special formatting for `if`.

        match head {
            Expr::Symbol(name) if name == "do" => {
                // Always arrange a `do` block vertically.
                let (exprs, _) = self.arrange_all();
                layouts.push(Layout::item("(do"));
                layouts.push(Layout::indent(exprs));
                layouts.push(Layout::apply(Layout::item(")")));
                Layout::Stack(layouts)
            }
            Expr::Symbol(name) if name == "Func" || name == "if" => {
                // The first expr is rendered inline, the rest are rendered vertically.
                layouts.push(Layout::row(vec![
                    Layout::item(format!("({name}")),
                    self.arrange_next().unwrap(),
                ]));
                let (block, should_force_vertical) = self.arrange_all();
                if should_force_vertical || block.len() > 1 {
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
            Expr::Symbol(name) if name == "Array" => {
                // #TODO more sophisticated Array formatting needed.
                // Try to format the array horizontally.
                layouts.push(Layout::item("["));
                let (items, should_force_vertical) = self.arrange_all();
                if items.len() > 0 {
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
            Expr::Symbol(name) if name == "Dict" => {
                let (bindings, should_force_vertical) = self.arrange_all_pairs();

                if should_force_vertical || bindings.len() > 2 {
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
                let (mut bindings, should_force_vertical) = self.arrange_all_pairs();

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

    fn layout_from_expr(&mut self, expr: &Ann<Expr>) -> Layout {
        let Ann(expr, ann) = expr;

        let layout = match expr {
            Expr::Comment(s, _) => Layout::Item(s.clone()),
            Expr::TextSeparator => Layout::Separator, // #TODO different impl!
            Expr::String(s) => Layout::Item(format!("\"{s}\"")),
            Expr::Symbol(s) => Layout::Item(s.clone()),
            Expr::Int(n) => Layout::Item(n.to_string()),
            Expr::One => Layout::Item("()".to_string()),
            Expr::Bool(b) => Layout::Item(b.to_string()),
            Expr::Float(n) => Layout::Item(format_float(*n)),
            Expr::KeySymbol(s) => Layout::Item(format!(":{s}")),
            Expr::Char(c) => Layout::Item(format!(r#"(Char "{c}")"#)),
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Layout::Item("()".to_owned());
                }

                // #insight Recursive data structure, we recurse.

                let mut list_arranger = Arranger::new(exprs);
                list_arranger.arrange_list()
            }
            _ => Layout::Item(expr.to_string()),
        };

        if let Some(ann) = ann {
            if ann.len() > 1 {
                // #TODO give special key to implicit range annotation.
                // Remove the range annotation.
                let mut ann = ann.clone();
                ann.remove("range");
                return Layout::Ann(ann, Box::new(layout));
            }
        }

        layout
    }

    pub fn arrange(&mut self) -> Layout {
        let (rows, _) = self.arrange_all();
        Layout::Stack(rows)
    }
}
