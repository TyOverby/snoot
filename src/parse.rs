use tendril::StrTendril;
use super::token::*;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Span {
    text: StrTendril,

    line_start: u32,
    column_start: u32,
    byte_start: u32,

    line_end: u32,
    column_end: u32,
    byte_end: u32,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Sexpr {
    List {
        opening_token: TokenInfo,
        closing_token: TokenInfo,

        children: Vec<Sexpr>,
        span: Span,
    },
    UnaryOperator {
        op: TokenInfo,
        child: Box<Sexpr>,
        span: Span,
    },
    Number(TokenInfo, Span),
    String(TokenInfo, Span),
    Ident(TokenInfo, Span),
}

#[derive(Debug)]
pub enum Diagnostic {
    TokenizationError(TokError),
    UnclosedList(Span),
    UnmatchedListClosing(Span, Span),
    UnaryOpWithNoArgument(Span),
    ExtraClosing(Span),
}

pub struct ParseResult {
    pub roots: Vec<Sexpr>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Sexpr {
    pub fn span(&self) -> &Span {
        match self {
            &Sexpr::List { ref span, .. } => span,
            &Sexpr::UnaryOperator { ref span, .. } => span,
            &Sexpr::Number(_, ref span) |
            &Sexpr::String(_, ref span) |
            &Sexpr::Ident(_, ref span) => span,
        }
    }

    pub fn last_token(&self) -> &TokenInfo {
        match self {
            &Sexpr::List { ref closing_token, .. } => closing_token,
            &Sexpr::UnaryOperator { ref child, .. } => child.last_token(),
            &Sexpr::Number(ref token, _) |
            &Sexpr::String(ref token, _) |
            &Sexpr::Ident(ref token, _) => token,
        }
    }
}

impl Span {
    fn from_token(token: &TokenInfo) -> Span {
        let chars = token.string.chars().count() as u32;
        let bytes = token.string.len() as u32;

        Span {
            text: token.string.clone(),
            line_start: token.line_number,
            column_start: token.column_number,
            byte_start: token.byte_offset,

            line_end: token.line_number,
            column_end: token.column_number + chars,
            byte_end: token.byte_offset + bytes,
        }
    }

    fn from_spans(start: &Span, end: &Span, string: &StrTendril) -> Span {
        let (start, end) = if start.byte_start < end.byte_start {
            (start, end)
        } else {
            (end, start)
        };

        let text = string.subtendril(start.byte_start, end.byte_end - start.byte_start);
        println!("combining {:#?} and {:#?} to get {}", start, end, text);

        Span {
            text: text,
            line_start: start.line_start,
            column_start: start.column_start,
            byte_start: start.byte_start,

            line_end: end.line_end,
            column_end: end.column_end,
            byte_end: end.byte_end,
        }

    }
}

enum ParseStackItem {
    Global { children: Vec<Sexpr> },
    ListOpening {
        opening: TokenInfo,
        children: Vec<Sexpr>,
    },
    UnaryOperator { op: Option<TokenInfo> },
}

fn put(stack: &mut Vec<ParseStackItem>, expr: Sexpr, string: &StrTendril) {
    let recurse = match stack.last_mut().unwrap() {
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
            let total_span = Span::from_spans(&Span::from_token(&op), expr.span(), string);
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
            stack.pop();
            put(stack, expr, string);
        }
    }
}

fn close(stack: &mut Vec<ParseStackItem>,
         closed_by: Option<TokenInfo>,
         diagnostics: &mut Vec<Diagnostic>,
         string: &StrTendril) {
    match (stack.pop().unwrap(), closed_by) {
        (g @ ParseStackItem::Global { .. }, Some(closed_by)) => {
            stack.push(g);
            diagnostics.push(Diagnostic::ExtraClosing(Span::from_token(&closed_by)));
        }
        (ParseStackItem::UnaryOperator { op }, closed_by) => {
            let op = op.unwrap();
            diagnostics.push(Diagnostic::UnaryOpWithNoArgument(Span::from_token(&op)));
            close(stack, closed_by, diagnostics, string);
        }
        // TODO: Check to see if opening matches close
        (ParseStackItem::ListOpening { children, opening }, Some(closed_by)) => {
            let span = Span::from_spans(&Span::from_token(&opening),
                                        &Span::from_token(&closed_by),
                                        &string);
            let list_sexpr = Sexpr::List {
                opening_token: opening,
                closing_token: closed_by,
                children: children,
                span: span,
            };

            put(stack, list_sexpr, string);
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

            let span = Span::from_spans(&Span::from_token(&opening),
                                        &Span::from_token(&closed_token),
                                        &string);

            let list_sexpr = Sexpr::List {
                opening_token: opening,
                closing_token: closed_token,
                children: children,
                span: span.clone(),
            };
            put(stack, list_sexpr, string);

            diagnostics.push(Diagnostic::UnclosedList(span));
        }
    }
}

pub fn parse<I>(string: &StrTendril, tokens: I) -> ParseResult
    where I: Iterator<Item = TokResult<TokenInfo>>
{

    fn actual_parse<I>(string: &StrTendril, mut tokens: I, diag: &mut Vec<Diagnostic>) -> Vec<Sexpr>
        where I: Iterator<Item = TokResult<TokenInfo>>
    {

        let mut parse_stack = vec![ParseStackItem::Global { children: vec![] }];

        loop {
            let token = match tokens.next() {
                Some(Ok(t)) => t,
                Some(Err(e)) => {
                    diag.push(Diagnostic::TokenizationError(e));
                    continue;
                }
                None => break,
            };

            match token.typ {
                TokenType::String => {
                    let span = Span::from_token(&token);
                    put(&mut parse_stack, Sexpr::String(token, span), string);
                }
                TokenType::Number => {
                    let span = Span::from_token(&token);
                    put(&mut parse_stack, Sexpr::Number(token, span), string);
                }
                TokenType::Identifier => {
                    let span = Span::from_token(&token);
                    put(&mut parse_stack, Sexpr::Ident(token, span), string);
                }
                TokenType::UnaryOperator => {
                    parse_stack.push(ParseStackItem::UnaryOperator { op: Some(token) });
                }
                TokenType::Whitespace => { /* do nothing for now */ }
                TokenType::ListOpening => {
                    parse_stack.push(ParseStackItem::ListOpening {
                        opening: token,
                        children: vec![],
                    });
                }
                TokenType::ListClosing => {
                    close(&mut parse_stack, Some(token), diag, string);
                }
            }
        }

        while parse_stack.len() != 1 {
            close(&mut parse_stack, None, diag, string);
        }

        let global = parse_stack.pop().unwrap();
        if let ParseStackItem::Global { children } = global {
            children
        } else {
            panic!("not global");
        }
    }

    let mut diagnostics = vec![];
    let out = actual_parse(string, tokens, &mut diagnostics);
    ParseResult {
        roots: out,
        diagnostics: diagnostics,
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn test_ok(input: &str, expected: Vec<Sexpr>) {
        let input: StrTendril = input.into();
        let to = TokenizationOptions::default();
        let cto = to.compile().unwrap();

        let tokens = tokenize(input.clone(), &cto);

        let ParseResult { roots, diagnostics } = parse(&input, tokens);
        if !diagnostics.is_empty() {
            println!("{:?}", diagnostics);
            assert!(diagnostics.is_empty());
        }
        assert_eq!(roots, expected);
    }

    #[test]
    fn single_ident() {
        test_ok("foo",
                vec![Sexpr::Ident(TokenInfo {
                                      line_number: 1,
                                      column_number: 1,
                                      byte_offset: 0,
                                      typ: TokenType::Identifier,
                                      string: "foo".into(),
                                  },
                                  Span {
                                      text: "foo".into(),

                                      line_start: 1,
                                      column_start: 1,
                                      byte_start: 0,

                                      line_end: 1,
                                      column_end: 4,
                                      byte_end: 3,
                                  })]);
    }

    #[test]
    fn two_idents() {
        test_ok("foo bar",
                vec![Sexpr::Ident(TokenInfo {
                                      line_number: 1,
                                      column_number: 1,
                                      byte_offset: 0,
                                      typ: TokenType::Identifier,
                                      string: "foo".into(),
                                  },
                                  Span {
                                      text: "foo".into(),

                                      line_start: 1,
                                      column_start: 1,
                                      byte_start: 0,

                                      line_end: 1,
                                      column_end: 4,
                                      byte_end: 3,
                                  }),
                     Sexpr::Ident(TokenInfo {
                                      line_number: 1,
                                      column_number: 5,
                                      byte_offset: 4,
                                      typ: TokenType::Identifier,
                                      string: "bar".into(),
                                  },
                                  Span {
                                      text: "bar".into(),

                                      line_start: 1,
                                      column_start: 5,
                                      byte_start: 4,

                                      line_end: 1,
                                      column_end: 8,
                                      byte_end: 7,
                                  })]);
    }

    #[test]
    fn parens() {
        test_ok("()",
                vec![Sexpr::List {
                         opening_token: TokenInfo {
                             line_number: 1,
                             column_number: 1,
                             byte_offset: 0,
                             typ: TokenType::ListOpening,
                             string: "(".into(),
                         },
                         closing_token: TokenInfo {
                             line_number: 1,
                             column_number: 2,
                             byte_offset: 1,
                             typ: TokenType::ListClosing,
                             string: ")".into(),
                         },

                         children: vec![],
                         span: Span {
                             text: "()".into(),

                             line_start: 1,
                             column_start: 1,
                             byte_start: 0,

                             line_end: 1,
                             column_end: 3,
                             byte_end: 2,
                         },
                     }])
    }
}


#[cfg(test)]
mod trx_test {
    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    enum SimpleSexpr {
        List {
            opening: String,
            closing: String,
            entire: String,
            children: Vec<SimpleSexpr>,
        },
        UnaryOperator {
            op: String,
            entire: String,
            child: Box<SimpleSexpr>,
        },
        Number(String),
        String(String),
        Ident(String),
    }

    impl From<Sexpr> for SimpleSexpr {
        fn from(sexpr: Sexpr) -> SimpleSexpr {
            match sexpr {
                Sexpr::List { opening_token, closing_token, children, span } => {
                    SimpleSexpr::List {
                        opening: opening_token.string.into(),
                        closing: closing_token.string.into(),
                        entire: span.text.into(),
                        children: children.into_iter().map(From::from).collect(),
                    }
                }
                Sexpr::UnaryOperator { op, child, span } => {
                    SimpleSexpr::UnaryOperator {
                        op: op.string.into(),
                        entire: span.text.into(),
                        child: Box::new(From::from(*child)),
                    }
                }
                Sexpr::Number(tok, _) => SimpleSexpr::Number(tok.string.into()),
                Sexpr::String(tok, _) => SimpleSexpr::String(tok.string.into()),
                Sexpr::Ident(tok, _) => SimpleSexpr::Ident(tok.string.into()),
            }
        }
    }

    fn parse_simple_ok(string: &str, expected: Vec<SimpleSexpr>) {
        let input: StrTendril = string.into();
        let to = TokenizationOptions::default();
        let cto = to.compile().unwrap();

        let tokens = tokenize(input.clone(), &cto);

        let ParseResult { roots, diagnostics } = parse(&input, tokens);
        if !diagnostics.is_empty() {
            println!("{:?}", diagnostics);
            assert!(diagnostics.is_empty());
        }
        for (actual, expected) in roots.into_iter().map(SimpleSexpr::from).zip(expected) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn ident() {
        parse_simple_ok("foo", vec![SimpleSexpr::Ident("foo".into())]);
        parse_simple_ok("foo bar",
                        vec![SimpleSexpr::Ident("foo".into()), SimpleSexpr::Ident("bar".into())]);
    }

    #[test]
    fn list() {
        parse_simple_ok("()",
                        vec![SimpleSexpr::List {
                                 opening: "(".into(),
                                 closing: ")".into(),
                                 entire: "()".into(),
                                 children: vec![],
                             }]);

        parse_simple_ok("(())",
                        vec![SimpleSexpr::List {
                                 opening: "(".into(),
                                 closing: ")".into(),
                                 entire: "(())".into(),
                                 children: vec![SimpleSexpr::List {
                                                    opening: "(".into(),
                                                    closing: ")".into(),
                                                    entire: "()".into(),
                                                    children: vec![],
                                                }],
                             }]);
    }
}
