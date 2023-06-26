use std::collections::HashMap;

use tan::{ann::Ann, expr::Expr, util::put_back_iterator::PutBackIterator};

use crate::util::format_float;

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
    Indent(Box<Layout>, Option<usize>),
    /// List, to be joined, no separator.
    Join(Vec<Layout>),
    /// Vertical list, separated by EOL
    VList(Vec<Layout>), // #TODO Could name this Column
    /// Horizontal list, separated by SPACE
    HList(Vec<Layout>), // #TODO Could name this Row
    Span(String),
    Ann(HashMap<String, Expr>, Box<Layout>),
    Separator,
}

impl Layout {
    pub fn indent(layout: Layout) -> Self {
        Self::Indent(Box::new(layout), None)
    }

    pub fn align(layout: Layout, indent_size: usize) -> Self {
        Self::Indent(Box::new(layout), Some(indent_size))
    }

    pub fn span(s: impl Into<String>) -> Self {
        Self::Span(s.into())
    }

    pub fn hlist(list: impl Into<Vec<Layout>>) -> Self {
        Self::HList(list.into())
    }

    pub fn separator() -> Self {
        Self::Separator
        // Self::Span("".to_owned())
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
                        return Some(Layout::HList(vec![layout, comment]));
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

    fn arrange_all(&mut self) -> Vec<Layout> {
        let mut layouts = Vec::new();

        while let Some(layout) = self.arrange_next() {
            layouts.push(layout);
        }

        layouts
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

        Some(Layout::HList(tuple))
    }

    fn arrange_all_pairs(&mut self) -> Layout {
        let mut layouts = Vec::new();

        let mut has_inline_comment = false;

        while let Some(layout) = self.arrange_next_pair() {
            if let Layout::HList(spans) = &layout {
                if spans.len() > 2 {
                    has_inline_comment = true;
                }
            };

            layouts.push(layout);
        }

        // #TODO consider extracting the following outside, for extra flexibility.

        let should_arrange_vertical = has_inline_comment || layouts.len() > 2;

        if should_arrange_vertical {
            Layout::VList(layouts)
        } else {
            Layout::HList(layouts)
        }
    }

    fn arrange_list(&mut self) -> Layout {
        // #indsight not need to check here.
        let expr = self.exprs.next().unwrap();

        let mut layouts = Vec::new();

        let head = &expr.0;

        // #TODO should decide between (h)list/vlist.
        // #TODO special formatting for `if`.

        match head {
            Expr::Symbol(name) if name == "do" => {
                layouts.push(Layout::span("(do"));
                // Always arrange a `do` block vertically.
                let block = Layout::VList(self.arrange_all());
                layouts.push(Layout::indent(block));
                layouts.push(Layout::span(")"));
                Layout::VList(layouts)
            }
            Expr::Symbol(name) if name == "Func" || name == "if" => {
                // The first expr is rendered inline, the rest are rendered vertically.
                layouts.push(Layout::HList(vec![
                    Layout::span(format!("({name}")),
                    self.arrange_next().unwrap(),
                ]));
                let block = self.arrange_all();
                if block.len() > 1 {
                    layouts.push(Layout::indent(Layout::VList(block)));
                    layouts.push(Layout::span(")"));
                    Layout::VList(layouts)
                } else {
                    layouts.push(Layout::span(" "));
                    layouts.push(block[0].clone());
                    layouts.push(Layout::span(")"));
                    Layout::Join(layouts)
                }
            }
            Expr::Symbol(name) if name == "Array" => {
                // #TODO more sophisticated Array formatting needed.
                // Try to format the array horizontally.
                layouts.push(Layout::span("["));
                let block = self.arrange_all();
                if block.len() > 0 {
                    match &block[0] {
                        // Heuristic: if the array includes blocks, arrange
                        // vertically.
                        Layout::VList(_) | Layout::Indent(..) => {
                            layouts.push(Layout::indent(Layout::VList(block)));
                            layouts.push(Layout::span("]"));
                            Layout::VList(layouts)
                        }
                        _ => {
                            layouts.push(Layout::HList(block));
                            layouts.push(Layout::span("]"));
                            Layout::Join(layouts)
                        }
                    }
                } else {
                    layouts.push(Layout::span("]"));
                    Layout::Join(layouts)
                }
            }
            Expr::Symbol(name) if name == "Dict" => {
                layouts.push(Layout::span("{"));
                let block = self.arrange_all_pairs();
                match block {
                    Layout::HList(_) => {
                        layouts.push(block);
                        layouts.push(Layout::span('}'));
                        Layout::Join(layouts)
                    },
                    _ /* Layout::VList */ => {
                        // #TODO Indent should auto convert to VList
                        layouts.push(Layout::indent(block));
                        layouts.push(Layout::span('}'));
                        Layout::VList(layouts)
                    }
                }
            }
            Expr::Symbol(name) if name == "let" => {
                // #TODO put the first binding on the same line.
                // #TODO precise alignment.
                layouts.push(Layout::span("(let"));
                let block = self.arrange_all_pairs();
                match block {
                    Layout::HList(_) => {
                        layouts.push(Layout::span(" "));
                        layouts.push(block);
                        layouts.push(Layout::span(')'));
                        Layout::Join(layouts)
                    },
                    _ /* Layout::VList */ => {
                        // #TODO Indent should auto convert to VList
                        layouts.push(Layout::align(block, 5 /* "(let " */));
                        layouts.push(Layout::span(')'));
                        Layout::VList(layouts)
                    }
                }
            }
            _ => {
                // Function call.
                layouts.push(Layout::span(format!("({head}")));
                let args = self.arrange_all();
                if !args.is_empty() {
                    layouts.push(Layout::span(" "));
                    layouts.push(Layout::HList(args));
                }
                layouts.push(Layout::span(")"));
                Layout::Join(layouts)
            }
        }
    }

    fn layout_from_expr(&mut self, expr: &Ann<Expr>) -> Layout {
        let Ann(expr, ann) = expr;

        let layout = match expr {
            Expr::Comment(s, _) => Layout::Span(s.clone()),
            Expr::TextSeparator => Layout::separator(), // #TODO different impl!
            Expr::String(s) => Layout::Span(format!("\"{s}\"")),
            Expr::Symbol(s) => Layout::Span(s.clone()),
            Expr::Int(n) => Layout::Span(n.to_string()),
            Expr::One => Layout::Span("()".to_string()),
            Expr::Bool(b) => Layout::Span(b.to_string()),
            Expr::Float(n) => Layout::Span(format_float(*n)),
            Expr::KeySymbol(s) => Layout::Span(format!(":{s}")),
            Expr::Char(c) => Layout::Span(format!(r#"(Char "{c}")"#)),
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Layout::Span("()".to_owned());
                }

                // #insight Recursive data structure, we recurse.

                let mut list_arranger = Arranger::new(exprs);
                list_arranger.arrange_list()
            }
            _ => Layout::Span(expr.to_string()),
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
        Layout::VList(self.arrange_all())
    }
}
