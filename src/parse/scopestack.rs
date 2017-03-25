use super::*;
use super::super::token::TokenInfo;
use tendril::StrTendril;

enum ParseStackItem {
    Global { children: Vec<Sexpr> },
    ListOpening {
        opening: TokenInfo,
        typ: ListType,
        children: Vec<Sexpr>,
    },
}

pub struct ScopeStack {
    stack: Vec<ParseStackItem>,
    string: StrTendril,
}

impl ScopeStack {
    pub fn new(string: StrTendril) -> ScopeStack {
        ScopeStack {
            stack: vec![ParseStackItem::Global { children: vec![] }],
            string: string,
        }
    }

    pub fn open_list(&mut self, typ: ListType, token: TokenInfo) {
        self.stack.push(ParseStackItem::ListOpening {
                            opening: token,
                            typ,
                            children: vec![],
                        });
    }

    pub fn end(mut self, diagnostics: &mut Vec<Diagnostic>) -> Vec<Sexpr> {
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

    pub fn put(&mut self, expr: Sexpr) {
        let recurse = match self.stack.last_mut().unwrap() {
            &mut ParseStackItem::Global { ref mut children } => {
                children.push(expr);
                None
            }
            &mut ParseStackItem::ListOpening { ref mut children, .. } => {
                children.push(expr);
                None
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

    pub fn close(&mut self, closed_by: Option<(ListType, TokenInfo)>, diagnostics: &mut Vec<Diagnostic>) {
        match (self.stack.pop().unwrap(), closed_by) {
            (g @ ParseStackItem::Global { .. }, Some((_closed_by_lst_typ, closed_by_tok))) => {
                self.stack.push(g);
                diagnostics.push(Diagnostic::ExtraClosing(Span::from_token(&closed_by_tok,
                                                                           &self.string)));
            }
            // TODO: Check to see if opening matches close
            (ParseStackItem::ListOpening { children, typ, opening }, Some((_closed_by_lst_typ, closed_by_tok))) => {
                let span = Span::from_spans(&Span::from_token(&opening, &self.string),
                                            &Span::from_token(&closed_by_tok, &self.string));
                let list_sexpr = Sexpr::List {
                    opening_token: opening,
                    list_type: typ,
                    closing_token: closed_by_tok,
                    children: children,
                    span: span,
                };

                self.put(list_sexpr);
            }
            (ParseStackItem::Global { .. }, None) => {
                unreachable!();
            }
            (ParseStackItem::ListOpening { children, typ, opening }, None) => {
                let closed_token = if let Some(chld) = children.last() {
                    chld.last_token().clone()
                } else {
                    opening.clone()
                };

                let span = Span::from_spans(&Span::from_token(&opening, &self.string),
                                            &Span::from_token(&closed_token, &self.string));

                let list_sexpr = Sexpr::List {
                    opening_token: opening,
                    list_type: typ,
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

