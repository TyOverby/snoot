use super::*;
use super::super::token::TokenInfo;
use super::super::Parseable;

enum ParseStackItem<S: Parseable> {
    Global { children: Vec<Sexpr<S>> },
    ListOpening {
        opening: TokenInfo<S>,
        children: Vec<Sexpr<S>>,
    },
    UnaryOperator { op: Option<TokenInfo<S>> },
}

pub struct ScopeStack<S: Parseable> {
    stack: Vec<ParseStackItem<S>>,
    string: S,
}

impl <S: Parseable> ScopeStack<S> {
    pub fn new(string: S) -> ScopeStack<S> {
        ScopeStack {
            stack: vec![ParseStackItem::Global { children: vec![] }],
            string: string,
        }
    }

    pub fn open_unary(&mut self, token: TokenInfo<S>) {
        self.stack.push(ParseStackItem::UnaryOperator { op: Some(token) });
    }

    pub fn open_list(&mut self, token: TokenInfo<S>) {
        self.stack.push(ParseStackItem::ListOpening {
            opening: token,
            children: vec![],
        });
    }

    pub fn end(mut self, diagnostics: &mut Vec<Diagnostic<S>>) -> Vec<Sexpr<S>> {
        while self.stack.len() != 1 {
            self.close(None, diagnostics);
        }

        let global = self.stack.pop().unwrap();

        if let ParseStackItem::Global { children } = global {
            children
        } else {
            panic!("not global");
        }
    }

    pub fn put(&mut self, expr: Sexpr<S>) {
        let recurse = match self.stack.last_mut().unwrap() {
            &mut ParseStackItem::Global { ref mut children } => {
                children.push(expr);
                None
            }
            &mut ParseStackItem::ListOpening { ref mut children, .. } => {
                children.push(expr);
                None
            }
            &mut ParseStackItem::UnaryOperator { ref mut op } => {
                let op = op.take().unwrap();
                let total_span =
                    Span::from_spans(&Span::from_token(&op, &self.string), expr.span(), &self.string);
                let finished = Sexpr::UnaryOperator {
                    op: op,
                    child: Box::new(expr),
                    span: total_span,
                };
                Some(finished)
            }
        };

        match recurse {
            None => {}
            Some(expr) => {
                self.stack.pop();
                self.put(expr);
            }
        }
    }

    pub fn close(&mut self, closed_by: Option<TokenInfo<S>>, diagnostics: &mut Vec<Diagnostic<S>>) {
        match (self.stack.pop().unwrap(), closed_by) {
            (g @ ParseStackItem::Global { .. }, Some(closed_by)) => {
                self.stack.push(g);
                diagnostics.push(Diagnostic::ExtraClosing(Span::from_token(&closed_by, &self.string)));
            }
            (ParseStackItem::UnaryOperator { op }, closed_by) => {
                let op = op.unwrap();
                diagnostics.push(Diagnostic::UnaryOpWithNoArgument(Span::from_token(&op, &self.string)));
                self.close(closed_by, diagnostics);
            }
            // TODO: Check to see if opening matches close
            (ParseStackItem::ListOpening { children, opening }, Some(closed_by)) => {
                let span = Span::from_spans(&Span::from_token(&opening, &self.string),
                                            &Span::from_token(&closed_by, &self.string),
                                            &self.string);
                let list_sexpr = Sexpr::List {
                    opening_token: opening,
                    closing_token: closed_by,
                    children: children,
                    span: span,
                };

                self.put(list_sexpr);
            }
            (ParseStackItem::Global { .. }, None) => {
                unreachable!();
            }
            (ParseStackItem::ListOpening { children, opening }, None) => {
                let closed_token = if let Some(chld) = children.last() {
                    chld.last_token().clone()
                } else {
                    opening.clone()
                };

                let span = Span::from_spans(&Span::from_token(&opening, &self.string),
                                            &Span::from_token(&closed_token, &self.string),
                                            &self.string);

                let list_sexpr = Sexpr::List {
                    opening_token: opening,
                    closing_token: closed_token,
                    children: children,
                    span: span.clone(),
                };
                self.put(list_sexpr);

                diagnostics.push(Diagnostic::UnclosedList(span));
            }
        }
    }
}
