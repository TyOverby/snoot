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
    Number(String),
    String(String),
    Ident(String),
}

impl From<Sexpr> for SimpleSexpr {
    fn from(
        sexpr: Sexpr
    ) -> SimpleSexpr {
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

fn parse_simple_ok(
    string: &str,
    expected: Vec<SimpleSexpr>
) {
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
