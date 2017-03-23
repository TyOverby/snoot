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
                                     length: 3,
                                     string: "foo".into(),
                                 },
                                 Span {
                                     full_text: "foo".into(),
                                     text_bytes: StartEnd { start: 0, end: 3 },
                                     lines_bytes: StartEnd { start: 0, end: 3 },
                                     lines_covered: StartEnd { start: 1, end: 1 },
                                     columns: StartEnd { start: 1, end: 4 },
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
                                     length: 3,
                                     string: "foo".into(),
                                 },
                                 Span {
                                     full_text: "foo bar".into(),
                                     text_bytes: StartEnd { start: 0, end: 3 },
                                     lines_bytes: StartEnd { start: 0, end: 7 },
                                     lines_covered: StartEnd { start: 1, end: 1 },
                                     columns: StartEnd { start: 1, end: 4 },
                                 }),
                 Sexpr::Terminal(TokenInfo {
                                     line_number: 1,
                                     column_number: 5,
                                     byte_offset: 4,
                                     typ: TokenType::Atom,
                                     length: 3,
                                     string: "bar".into(),
                                 },
                                 Span {
                                     full_text: "foo bar".into(),
                                     text_bytes: StartEnd { start: 4, end: 7 },
                                     lines_bytes: StartEnd { start: 0, end: 7 },
                                     lines_covered: StartEnd { start: 1, end: 1 },
                                     columns: StartEnd { start: 5, end: 8 },
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
                         length: 1,
                         string: "(".into(),
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(0),
                         length: 1,
                         string: ")".into(),
                     },

                     children: vec![],
                     span: Span {
                         full_text: "()".into(),
                         text_bytes: StartEnd { start: 0, end: 2 },
                         lines_bytes: StartEnd { start: 0, end: 2 },
                         lines_covered: StartEnd { start: 1, end: 1 },
                         columns: StartEnd { start: 1, end: 3 },
                     },
                 }]);
    test_ok("{}".into(),
            vec![Sexpr::List {
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(1),
                         length: 1,
                         string: "{".into(),
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(1),
                         length: 1,
                         string: "}".into(),
                     },

                     children: vec![],
                     span: Span {
                         full_text: "{}".into(),
                         text_bytes: StartEnd { start: 0, end: 2 },
                         lines_bytes: StartEnd { start: 0, end: 2 },
                         lines_covered: StartEnd { start: 1, end: 1 },
                         columns: StartEnd { start: 1, end: 3 },
                     },
                 }]);
    test_ok("[]".into(),
            vec![Sexpr::List {
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(2),
                         length: 1,
                         string: "[".into(),
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(2),
                         length: 1,
                         string: "]".into(),
                     },

                     children: vec![],
                     span: Span {
                         full_text: "[]".into(),
                         text_bytes: StartEnd { start: 0, end: 2 },
                         lines_bytes: StartEnd { start: 0, end: 2 },
                         lines_covered: StartEnd { start: 1, end: 1 },
                         columns: StartEnd { start: 1, end: 3 },
                     },
                 }]);
}

