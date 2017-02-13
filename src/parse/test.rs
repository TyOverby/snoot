#![cfg(test)]

use super::*;

pub fn test_ok(input: &str, expected: Vec<Sexpr>) {
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
                                  lines: "foo bar".into(),

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
                         lines: "()".into(),

                         line_start: 1,
                         column_start: 1,
                         byte_start: 0,

                         line_end: 1,
                         column_end: 3,
                         byte_end: 2,
                     },
                 }])
}
