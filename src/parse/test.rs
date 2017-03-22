#![cfg(test)]

use super::*;

pub fn test_ok(input: &str, expected: Vec<Sexpr>) {
    let tokens = tokenize(input.into(), &[]);

    let ParseResult { roots, diagnostics } = parse(&input.into(), tokens);
    if !diagnostics.is_empty() {
        println!("{:?}", diagnostics);
        assert!(diagnostics.is_empty());
    }
    assert_eq!(roots, expected);
}

#[test]
fn single_ident() {
    test_ok("foo".into(),
            vec![Sexpr::Terminal(TokenInfo {
                                     line_number: 1,
                                     column_number: 1,
                                     byte_offset: 0,
                                     typ: TokenType::Atom,
                                     string: "foo".into(),
                                 },
                                 Span {
                                     full_text: "foo".into(),
                                     text: "foo".into(),
                                     lines: "foo".into(),

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
    test_ok("foo bar".into(),
            vec![Sexpr::Terminal(TokenInfo {
                                     line_number: 1,
                                     column_number: 1,
                                     byte_offset: 0,
                                     typ: TokenType::Atom,
                                     string: "foo".into(),
                                 },
                                 Span {
                                     full_text: "foo bar".into(),
                                     text: "foo".into(),
                                     lines: "foo bar".into(),

                                     line_start: 1,
                                     column_start: 1,
                                     byte_start: 0,

                                     line_end: 1,
                                     column_end: 4,
                                     byte_end: 3,
                                 }),
                 Sexpr::Terminal(TokenInfo {
                                     line_number: 1,
                                     column_number: 5,
                                     byte_offset: 4,
                                     typ: TokenType::Atom,
                                     string: "bar".into(),
                                 },
                                 Span {
                                     full_text: "foo bar".into(),
                                     text: "bar".into(),
                                     lines: "foo bar".into(),

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
    test_ok("()".into(),
            vec![Sexpr::List {
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(0),
                         string: "(".into(),
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(0),
                         string: ")".into(),
                     },

                     children: vec![],
                     span: Span {
                         full_text: "()".into(),
                         text: "()".into(),
                         lines: "()".into(),

                         line_start: 1,
                         column_start: 1,
                         byte_start: 0,

                         line_end: 1,
                         column_end: 3,
                         byte_end: 2,
                     },
                 }]);
    test_ok("{}".into(),
            vec![Sexpr::List {
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(1),
                         string: "{".into(),
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(1),
                         string: "}".into(),
                     },

                     children: vec![],
                     span: Span {
                         full_text: "{}".into(),
                         text: "{}".into(),
                         lines: "{}".into(),

                         line_start: 1,
                         column_start: 1,
                         byte_start: 0,

                         line_end: 1,
                         column_end: 3,
                         byte_end: 2,
                     },
                 }]);
    test_ok("[]".into(),
            vec![Sexpr::List {
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(2),
                         string: "[".into(),
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(2),
                         string: "]".into(),
                     },

                     children: vec![],
                     span: Span {
                         full_text: "[]".into(),
                         text: "[]".into(),
                         lines: "[]".into(),

                         line_start: 1,
                         column_start: 1,
                         byte_start: 0,

                         line_end: 1,
                         column_end: 3,
                         byte_end: 2,
                     },
                 }]);
}

