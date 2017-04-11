#![cfg(test)]

use super::*;

pub fn test_ok(input: &str, expected: Vec<Sexpr>) {
    let tokens = tokenize(input.into(), &[]);

    let Result { roots, diagnostics } = parse(&input.into(), tokens, None);
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
                                 },
                                 Span {
                                     full_text: "foo".into(),
                                     file: None,
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
                                 },
                                 Span {
                                     full_text: "foo bar".into(),
                                     file: None,
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
                                 },
                                 Span {
                                     full_text: "foo bar".into(),
                                     file: None,
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
                     list_type: ListType::Paren,
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(ListType::Paren),
                         length: 1,
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(ListType::Paren),
                         length: 1,
                     },

                     children: vec![],
                     span: Span {
                         file: None,
                         full_text: "()".into(),
                         text_bytes: StartEnd { start: 0, end: 2 },
                         lines_bytes: StartEnd { start: 0, end: 2 },
                         lines_covered: StartEnd { start: 1, end: 1 },
                         columns: StartEnd { start: 1, end: 3 },
                     },
                 }]);
    test_ok("{}".into(),
            vec![Sexpr::List {
                     list_type: ListType::Brace,
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(ListType::Brace),
                         length: 1,
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(ListType::Brace),
                         length: 1,
                     },

                     children: vec![],
                     span: Span {
                         file: None,
                         full_text: "{}".into(),
                         text_bytes: StartEnd { start: 0, end: 2 },
                         lines_bytes: StartEnd { start: 0, end: 2 },
                         lines_covered: StartEnd { start: 1, end: 1 },
                         columns: StartEnd { start: 1, end: 3 },
                     },
                 }]);
    test_ok("[]".into(),
            vec![Sexpr::List {
                     list_type: ListType::Bracket,
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(ListType::Bracket),
                         length: 1,
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(ListType::Bracket),
                         length: 1,
                     },

                     children: vec![],
                     span: Span {
                         file: None,
                         full_text: "[]".into(),
                         text_bytes: StartEnd { start: 0, end: 2 },
                         lines_bytes: StartEnd { start: 0, end: 2 },
                         lines_covered: StartEnd { start: 1, end: 1 },
                         columns: StartEnd { start: 1, end: 3 },
                     },
                 }]);
}

