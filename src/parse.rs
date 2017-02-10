use tendril::StrTendril;
use super::token::*;
use itertools::structs::PutBack;
use itertools::put_back;

#[derive(Eq, PartialEq, Debug)]
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
        span: Span
    },
    UnaryOperator {
        op: TokenInfo,
        child: Box<Sexpr>,
        span: Span,
    },
    BinaryOperator {
        op: TokenInfo,
        left: Box<Sexpr>,
        right: Box<Sexpr>,
    },
    Number(TokenInfo, Span),
    String(TokenInfo, Span),
    Ident(TokenInfo, Span),
}

#[derive(Debug)]
pub enum Diagnostic {
    TokenizationError(TokError),
    IllegalOperatorGroup(Span),
    UnclosedList(Span),
    UnmatchedListClosing(Span, Span),
    ExtraClosing(Span),
}

pub struct ParseResult {
    pub roots: Vec<Sexpr>,
    pub diagnostics: Vec<Diagnostic>
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
        let (start, end) = if start.byte_start < end.byte_end {
            (start, end)
        } else {
            (end, start)
        };

        Span {
            text: string.subtendril(start.byte_start, end.byte_end),

            line_start: start.line_start,
            column_start: start.column_start,
            byte_start: start.byte_start,

            line_end: end.line_end,
            column_end: end.column_end,
            byte_end: end.byte_end,
        }

    }
}

pub fn parse<I>(string: &StrTendril, tokens: I) -> ParseResult
    where I: Iterator<Item=TokResult<TokenInfo>> {

    fn parse_one<I>(string: &StrTendril, tokens: &mut PutBack<I>, diagnostics: &mut Vec<Diagnostic>) -> Result<Option<Sexpr>, ()>
        where I: Iterator<Item=TokResult<TokenInfo>> {
        let token = match tokens.next() {
            Some(Ok(t)) => t,
            Some(Err(e)) => {
                diagnostics.push(Diagnostic::TokenizationError(e));
                return Err(());
            }
            None => return Ok(None),
        };

        let sexpr = match token.typ {
            TokenType::String => {
                let span = Span::from_token(&token);
                Sexpr::String(token, span)
            }
            TokenType::Number => {
                let span = Span::from_token(&token);
                Sexpr::Number(token, span)
            }
            TokenType::Identifier => {
                let span = Span::from_token(&token);
                Sexpr::Ident(token, span)
            }
            TokenType::BinaryOperator => {
                diagnostics.push(Diagnostic::IllegalOperatorGroup(Span::from_token(&token)));
                return Err(());
            }
            TokenType::ListClosing => {
                tokens.put_back(Ok(token));
                return Ok(None);
            }
            TokenType::Whitespace => {
                unreachable!();
            }
            TokenType::UnaryOperator => {
                unimplemented!();
            }
            TokenType::ListOpening => {
                let mut children = vec![];
                let close;
                loop {
                    match parse_one(string, tokens, diagnostics) {
                        Ok(Some(child)) => children.push(child),
                        Ok(None) => match tokens.next() {
                            Some(Ok(t)) => {
                                // TODO: check to see if these match up

                                /*
                                diagnostics.push(Diagnostic::UnmatchedListClosing(
                                Span::from_token(token),
                                Span::from_token(t)));
                                */
                                close = t;
                                break;
                            }
                            None => {
                                diagnostics.push(Diagnostic::UnclosedList(Span::from_token(&token)));
                                return Err(());
                            }
                            Some(Err(e)) => {
                                diagnostics.push(Diagnostic::TokenizationError(e));
                                return Err(());
                            }
                        },
                        Err(()) => {
                            return Err(());
                        }
                    }
                }

                let span = Span::from_spans(&Span::from_token(&token), &Span::from_token(&close), string);

                Sexpr::List {
                    opening_token: token,
                    closing_token: close,
                    children: children,
                    span: span,
                }
            }
        };

        Ok(Some(sexpr))
    }

    fn parse_lookahead<I>(string: &StrTendril, tokens: &mut PutBack<I>, diagnostics: &mut Vec<Diagnostic>) -> Result<Option<Sexpr>, ()>
        where I: Iterator<Item=TokResult<TokenInfo>> {
        let first = match parse_one(string, tokens, diagnostics) {
            Ok(Some(s)) => s,
            Ok(None) => return Ok(None),
            Err(()) => return Err(()),
        };

        let binary_op = match tokens.next() {
            Some(Ok(ref t)) if t.typ == TokenType::BinaryOperator => t.clone(),
            Some(other) => {
                tokens.put_back(other);
                return Ok(Some(first));
            }
            None => return Ok(Some(first))
        };

        let second = match parse_lookahead(string, tokens, diagnostics) {
            Ok(Some(s)) => s,
            Ok(None) => return Ok(None),
            Err(()) => return Err(()),
        };

        Ok(Some(Sexpr::BinaryOperator {
            op: binary_op,
            left: Box::new(first),
            right: Box::new(second),
        }))
    }


    fn actually_parse<I>(string: &StrTendril, tokens: &mut PutBack<I>, diagnostics: &mut Vec<Diagnostic>) -> Vec<Sexpr>
        where I: Iterator<Item=TokResult<TokenInfo>> {
        let mut out = vec![];
        loop {
            match parse_lookahead(string, tokens, diagnostics) {
                Ok(Some(sexpr)) => out.push(sexpr),
                Ok(None) => break,
                Err(()) => continue,
            }
        }

        out
    }

    let mut diags = vec![];

    // filter out all the whitespace
    let mut tokens = put_back(tokens.filter(|tok| {
        if let Ok(TokenType::Whitespace) = tok.as_ref().map(|t| t.typ) {
            false
        } else {
            true
        }
    }));

    let roots = actually_parse(string, &mut tokens, &mut diags);

    ParseResult {
        roots: roots,
        diagnostics: diags,
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
        test_ok("foo", vec![Sexpr::Ident(
            TokenInfo {
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
            }
        )]);
    }

    #[test]
    fn two_idents() {
        test_ok("foo bar", vec![
            Sexpr::Ident(
                TokenInfo {
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
                }
            ),
            Sexpr::Ident(
                TokenInfo {
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
                }
            )
        ]);
    }

    #[test]
    fn parens() {
        test_ok("()", vec![Sexpr::List {
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
            }
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
        BinaryOperator {
            op: String,
            left: Box<SimpleSexpr>,
            right: Box<SimpleSexpr>,
        },
        Number(String),
        String(String),
        Ident(String),
    }

    impl From<Sexpr> for SimpleSexpr {
        fn from(sexpr: Sexpr) -> SimpleSexpr {
            match sexpr {
                Sexpr::List {opening_token, closing_token, children, span} => SimpleSexpr::List {
                    opening: opening_token.string.into(),
                    closing: closing_token.string.into(),
                    entire: span.text.into(),
                    children: children.into_iter().map(From::from).collect(),
                },
                Sexpr::UnaryOperator {op, child, span} => SimpleSexpr::UnaryOperator {
                    op: op.string.into(),
                    entire: span.text.into(),
                    child: Box::new(From::from(*child)),
                },
                Sexpr::BinaryOperator {op, left, right} => SimpleSexpr::BinaryOperator {
                    op: op.string.into(),
                    left: Box::new(From::from(*left)),
                    right: Box::new(From::from(*right)),
                },
                Sexpr::Number(tok, _) => SimpleSexpr::Number(tok.string.into()),
                Sexpr::String(tok, _) => SimpleSexpr::String(tok.string.into()),
                Sexpr::Ident(tok, _) => SimpleSexpr::Ident(tok.string.into()),
            }
        }
    }

    fn parse_simple_ok(string: &str, expected: Vec<SimpleSexpr>){
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
        parse_simple_ok("foo bar", vec![
            SimpleSexpr::Ident("foo".into()),
            SimpleSexpr::Ident("bar".into()),
        ]);
    }

    #[test]
    fn list() {
        parse_simple_ok("()", vec![SimpleSexpr::List{
            opening: "(".into(),
            closing: ")".into(),
            entire: "()".into(),
            children: vec![],
        }]);

        parse_simple_ok("(())", vec![SimpleSexpr::List{
            opening: "(".into(),
            closing: ")".into(),
            entire: "(())".into(),
            children: vec![
                SimpleSexpr::List{
                    opening: "(".into(),
                    closing: ")".into(),
                    entire: "()".into(),
                    children: vec![],
                }
            ],
        }]);
    }
}
