use tan::{ann::Ann, expr::Expr, util::put_back_iterator::PutBackIterator};

use crate::util::format_float;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    // #TODO could name this layout 'Cell' or Fragment
    /// Indentation block
    Indent(Box<Layout>),
    /// List, no separator
    List(Vec<Layout>), // #TODO Could name this SpanMany, or Join
    /// Vertical list, separated by EOL
    VList(Vec<Layout>), // #TODO Could name this Column
    /// Horizontal list, separated by SPACE
    HList(Vec<Layout>), // #TODO Could name this Row
    /// Justified vertical list
    // Grid(Vec<Layout>),
    Span(String),
    Separator,
}

impl Layout {
    pub fn span(s: impl Into<String>) -> Self {
        Self::Span(s.into())
    }

    pub fn hlist(list: impl Into<Vec<Layout>>) -> Self {
        Self::HList(list.into())
    }
}

// #TODO what about annotations?

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

    fn arrange_next(&mut self) -> Layout {
        // #TODO should handle inline comment?

        // #insight not checking, input to formatter should be valid.
        let expr = self.exprs.next().unwrap();
        self.arrange_expr(expr)
    }

    fn arrange_rest(&mut self) -> Vec<Layout> {
        let mut layouts = Vec::new();

        loop {
            let mut row = Vec::new();

            let Some(expr0) = self.exprs.next() else {
                break;
            };

            row.push(self.arrange_expr(expr0));

            let mut has_inline_comment = false;

            if let Some(expr1) = self.exprs.next() {
                match expr1 {
                    Ann(Expr::Comment(..), _) => {
                        if expr1.get_range().unwrap().start.line
                            == expr0.get_range().unwrap().start.line
                        {
                            has_inline_comment = true;
                            row.push(self.arrange_expr(expr1));
                        } else {
                            self.exprs.put_back(expr1);
                        }
                    }
                    _ => {
                        self.exprs.put_back(expr1);
                    }
                }
            };

            if has_inline_comment {
                layouts.push(Layout::HList(row));
            } else {
                layouts.push(row[0].clone());
            }
        }

        layouts
    }

    fn arrange_rest_as_pairs(&mut self) -> Layout {
        let mut layouts = Vec::new();

        let mut has_inline_comment = false;

        loop {
            let mut row = Vec::new();

            let Some(expr0) = self.exprs.next() else {
                break;
            };

            row.push(self.arrange_expr(expr0));

            row.push(self.arrange_next());

            if let Some(expr2) = self.exprs.next() {
                match expr2 {
                    Ann(Expr::Comment(..), _) => {
                        if expr2.get_range().unwrap().start.line
                            == expr0.get_range().unwrap().start.line
                        {
                            has_inline_comment = true;
                            row.push(self.arrange_expr(expr2));
                        } else {
                            self.exprs.put_back(expr2);
                        }
                    }
                    _ => {}
                }
            };

            layouts.push(Layout::HList(row));
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
                let block = Layout::VList(self.arrange_rest());
                layouts.push(Layout::Indent(Box::new(block)));
                layouts.push(Layout::span(")"));
                Layout::VList(layouts)
            }
            Expr::Symbol(name) if name == "Func" || name == "if" => {
                // The first expr is rendered inline, the rest are rendered vertically.
                layouts.push(Layout::span(format!("({name} ")));
                layouts.push(self.arrange_next());
                let block = self.arrange_rest();
                if block.len() > 1 {
                    layouts.push(Layout::Indent(Box::new(Layout::VList(block))));
                    layouts.push(Layout::span(")"));
                    Layout::VList(layouts)
                } else {
                    layouts.push(block[0].clone());
                    layouts.push(Layout::span(")"));
                    Layout::HList(layouts)
                }
            }
            Expr::Symbol(name) if name == "Array" => {
                // #TODO more sophisticated Array formatting needed.
                // Try to format the array horizontally.
                layouts.push(Layout::span("["));
                layouts.push(Layout::HList(self.arrange_rest()));
                layouts.push(Layout::span("]"));
                Layout::List(layouts)
            }
            Expr::Symbol(name) if name == "Dict" => {
                layouts.push(Layout::span("{"));
                let block = self.arrange_rest();
                if block.len() > 2 {
                    layouts.push(Layout::Indent(Box::new(Layout::VList(block))));
                    layouts.push(Layout::span('}'));
                    Layout::VList(layouts)
                } else {
                    layouts.push(Layout::HList(block));
                    layouts.push(Layout::span("}"));
                    Layout::HList(layouts)
                }
            }
            Expr::Symbol(name) if name == "let" => {
                layouts.push(Layout::span("(let "));
                let block = self.arrange_rest_as_pairs();
                match block {
                    Layout::HList(_) => {
                        layouts.push(block);
                        layouts.push(Layout::span(')'));
                        Layout::HList(layouts)
                    },
                    _ /* Layout::VList */ => {
                        // #TODO Indent should auto convert to VList
                        layouts.push(Layout::Indent(Box::new(block)));
                        layouts.push(Layout::span(')'));
                        Layout::VList(layouts)
                    }
                }
            }
            _ => {
                layouts.push(Layout::span(format!("({head} ")));
                layouts.push(Layout::HList(self.arrange_rest()));
                layouts.push(Layout::span(")"));
                Layout::List(layouts)
            }
        }
    }

    fn arrange_expr(&mut self, expr: &Ann<Expr>) -> Layout {
        let Ann(expr, _) = expr;

        let layout = match expr {
            Expr::Comment(s, _) => Layout::Span(s.clone()),
            Expr::TextSeparator => Layout::Separator, // #TODO different impl!
            // #TODO maybe it's better to format annotations from Expr?
            // Expr::Annotation(s) => format!("#{s}"),
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

        layout
    }

    pub fn arrange(&mut self) -> Layout {
        Layout::VList(self.arrange_rest())
    }
}
