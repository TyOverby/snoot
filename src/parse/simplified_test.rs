#![cfg(test)]

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
    String(String),
    Ident(String),
}

impl <'a> From<Sexpr<&'a str>> for SimpleSexpr {
    fn from(sexpr: Sexpr<&'a str>) -> SimpleSexpr {
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

            Sexpr::String(tok, _) => SimpleSexpr::String(tok.string.into()),
            Sexpr::Terminal(tok, _) => SimpleSexpr::Ident(tok.string.into()),
        }
    }
}

fn parse_simple_ok(string: &str, expected: Vec<SimpleSexpr>) {
    let tokens = tokenize(string);

    let ParseResult { roots, diagnostics } = parse(&string, tokens);
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

#[test]
fn multiple_top_level_lists() {
    parse_simple_ok("() ()",
                    vec![SimpleSexpr::List {
                             opening: "(".into(),
                             closing: ")".into(),
                             entire: "()".into(),
                             children: vec![],
                         },
                         SimpleSexpr::List {
                             opening: "(".into(),
                             closing: ")".into(),
                             entire: "()".into(),
                             children: vec![],
                         }]);
    parse_simple_ok("()()",
                    vec![SimpleSexpr::List {
                             opening: "(".into(),
                             closing: ")".into(),
                             entire: "()".into(),
                             children: vec![],
                         },
                         SimpleSexpr::List {
                             opening: "(".into(),
                             closing: ")".into(),
                             entire: "()".into(),
                             children: vec![],
                         }]);
}
