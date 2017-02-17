#![cfg(test)]

use super::*;

pub fn test_ok(input: &str, expected: Vec<Sexpr<&'static str>>) {
    let tokens = tokenize(input);

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
            vec![Sexpr::Terminal(TokenInfo {
                                  line_number: 1,
                                  column_number: 1,
                                  byte_offset: 0,
                                  typ: TokenType::Atom,
                                  string: "foo",
                              },
                              Span {
                                  text: "foo",
                                  lines: "foo",

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
            vec![Sexpr::Terminal(TokenInfo {
                                  line_number: 1,
                                  column_number: 1,
                                  byte_offset: 0,
                                  typ: TokenType::Atom,
                                  string: "foo",
                              },
                              Span {
                                  text: "foo",
                                  lines: "foo bar",

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
                                  string: "bar",
                              },
                              Span {
                                  text: "bar",
                                  lines: "foo bar",

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
                         typ: TokenType::ListOpening(0),
                         string: "(",
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(0),
                         string: ")",
                     },

                     children: vec![],
                     span: Span {
                         text: "()",
                         lines: "()",

                         line_start: 1,
                         column_start: 1,
                         byte_start: 0,

                         line_end: 1,
                         column_end: 3,
                         byte_end: 2,
                     },
                 }]);
    test_ok("{}",
            vec![Sexpr::List {
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(1),
                         string: "{",
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(1),
                         string: "}",
                     },

                     children: vec![],
                     span: Span {
                         text: "{}",
                         lines: "{}",

                         line_start: 1,
                         column_start: 1,
                         byte_start: 0,

                         line_end: 1,
                         column_end: 3,
                         byte_end: 2,
                     },
                 }]);
    test_ok("[]",
            vec![Sexpr::List {
                     opening_token: TokenInfo {
                         line_number: 1,
                         column_number: 1,
                         byte_offset: 0,
                         typ: TokenType::ListOpening(2),
                         string: "[",
                     },
                     closing_token: TokenInfo {
                         line_number: 1,
                         column_number: 2,
                         byte_offset: 1,
                         typ: TokenType::ListClosing(2),
                         string: "]",
                     },

                     children: vec![],
                     span: Span {
                         text: "[]",
                         lines: "[]",

                         line_start: 1,
                         column_start: 1,
                         byte_start: 0,

                         line_end: 1,
                         column_end: 3,
                         byte_end: 2,
                     },
                 }]);
}
