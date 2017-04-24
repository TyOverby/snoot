#![cfg(test)]

use tendril::StrTendril;

use super::*;

#[derive(PartialEq, Eq, Debug)]
enum SimpleSexpr {
    List {
        opening: String,
        closing: String,
        entire: String,
        children: Vec<SimpleSexpr>,
    },
    String(String),
    Ident(String),
}

impl<'a> From<Sexpr> for SimpleSexpr {
    fn from(sexpr: Sexpr) -> SimpleSexpr {
        match sexpr {
            Sexpr::List {
                list_type,
                children,
                span,
                ..
            } => {
                SimpleSexpr::List {
                    opening: list_type.to_string(true),
                    closing: list_type.to_string(false),
                    entire: span.text().into(),
                    children: children.into_iter().map(From::from).collect(),
                }
            }
            Sexpr::UnaryOperator { .. } => unimplemented!(),

            s @ Sexpr::String(_, _) => SimpleSexpr::String(s.span().text().into()),
            s @ Sexpr::Terminal(_, _) => SimpleSexpr::Ident(s.span().text().into()),
        }
    }
}

fn parse_simple_err(string: &str, expected: Vec<SimpleSexpr>, _error: &str) {
    let string: StrTendril = string.into();
    let (roots, _diagnostics) = {
        let tokens = tokenize(string.clone(), &[]);
        let Result { roots, diagnostics } = parse(&string, tokens, None);
        (roots, diagnostics)
    };

    // let actual = format!("{:?}", diagnostics);
    // assert_eq!(actual, error);

    for (actual, expected) in roots.into_iter().map(SimpleSexpr::from).zip(expected) {
        assert_eq!(actual, expected);
    }
}

fn parse_simple_ok(string: &str, expected: Vec<SimpleSexpr>) {
    parse_simple_ok_split(string, expected, &[]);
}

fn parse_simple_ok_split(string: &str, expected: Vec<SimpleSexpr>, splits: &[&str]) {
    let string: StrTendril = string.into();
    let (roots, diagnostics) = {
        let tokens = tokenize(string.clone(), splits);
        let Result { roots, diagnostics } = parse(&string, tokens, None);
        (roots, diagnostics)
    };

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

#[test]
fn prop_regression() {
    parse_simple_ok("{a: 5 b : 10}",
                    vec![SimpleSexpr::List {
                             opening: "{".into(),
                             closing: "}".into(),
                             entire: "{a: 5 b : 10}".into(),
                             children: vec![SimpleSexpr::Ident("a:".into()),
                                            SimpleSexpr::Ident("5".into()),
                                            SimpleSexpr::Ident("b".into()),
                                            SimpleSexpr::Ident(":".into()),
                                            SimpleSexpr::Ident("10".into())],
                         }]);

    parse_simple_ok_split("{a: 5 b : 10}",
                          vec![SimpleSexpr::List {
                                   opening: "{".into(),
                                   closing: "}".into(),
                                   entire: "{a: 5 b : 10}".into(),
                                   children: vec![SimpleSexpr::Ident("a".into()),
                                                  SimpleSexpr::Ident(":".into()),
                                                  SimpleSexpr::Ident("5".into()),
                                                  SimpleSexpr::Ident("b".into()),
                                                  SimpleSexpr::Ident(":".into()),
                                                  SimpleSexpr::Ident("10".into())],
                               }],
                          &[":"]);
}

#[test]
fn mismatched_list_recovery() {
    parse_simple_err("(a b { c d)",
                     vec![SimpleSexpr::List {
                              opening: "(".into(),
                              closing: ")".into(),
                              entire: "(a b { c d)".into(),
                              children: vec![SimpleSexpr::Ident("a".into()),
                                             SimpleSexpr::Ident("b".into()),
                                             SimpleSexpr::List {
                                                 opening: "{".into(),
                                                 closing: "}".into(),
                                                 entire: "{ c d)".into(),
                                                 children: vec![
                                                    SimpleSexpr::Ident("c".into()),
                                                    SimpleSexpr::Ident("d".into()),
                                                 ],
                                             }],
                          }],
                     "");
}
