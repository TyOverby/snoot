use super::*;
use serde::*;
use super::super::Result as ParseResult;


fn run_test_good<T: ::std::fmt::Debug + Eq + for <'a> Deserialize<'a>>(input: &str, value: T) {
    let ParseResult { roots, diagnostics }= ::simple_parse(input, &[":"], Some("run_test"));
    assert!(roots.len() == 1);
    diagnostics.assert_no_errors();

    match deserialize::<T>(&roots[0]) {
        DeserializeResult::AllGood(t) => assert_eq!(t, value),
        DeserializeResult::CouldRecover(t, mut diagnostics) => {
            diagnostics.sort();
            diagnostics.assert_empty(); panic!();
        }
        DeserializeResult::CouldntRecover(mut diagnostics) => {
            diagnostics.sort();
            diagnostics.assert_empty(); panic!();
        }
    }

}

fn run_test_bad<T: ::std::fmt::Debug + Eq + for <'a> Deserialize<'a>>(input: &str, diagnostic_messages: &[&str]) {
    let ParseResult { roots, diagnostics }= ::simple_parse(input, &[":"], Some("run_test"));
    diagnostics.assert_no_errors();
    assert!(roots.len() == 1);

    match deserialize::<T>(&roots[0]) {
        DeserializeResult::AllGood(t) => {
            panic!("expected to fail")
        }
        DeserializeResult::CouldRecover(t, mut diagnostics) => {
            diagnostics.sort();
            for (diagnostic, message) in diagnostics.iter().zip(diagnostic_messages.iter()) {
                assert_eq!(&diagnostic.message, &**message);
            }
        }
        DeserializeResult::CouldntRecover(diagnostics) => {
            diagnostics.assert_empty(); panic!();
        }
    }
}

#[test]
fn test_simple_deserialization() {
    // bool
    run_test_good("true", true);
    run_test_good("false", false);

    // u8
    run_test_good("5", 5 as u8);
    run_test_bad::<u8>("600", &["could not parse `600` as a unsigned integer (u8)"]);
    run_test_bad::<u8>("-50", &["could not parse `-50` as a unsigned integer (u8)"]);

    // u16
    run_test_good("5", 5 as u16);
    run_test_bad::<u16>("600000", &["could not parse `600000` as a unsigned integer (u16)"]);
    run_test_bad::<u16>("-50", &["could not parse `-50` as a unsigned integer (u16)"]);
}

#[test]
fn test_seq_deserialization() {
    run_test_good("(true false true)", vec![true, false, true]);
    run_test_good::<Vec<bool>>("()", vec![]);

    run_test_good("(1 2 3 4)", vec![1, 2, 3, 4]);
}

#[test]
fn test_map_deserialization() {
    use std::collections::HashMap;
    run_test_good::<HashMap<_,_>>("{1:true 2:false 3:true}", vec![(1, true), (2, false), (3, true)].into_iter().collect())
}

#[test]
fn test_struct_deserialization() {
    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename="foo", rename_all="kebab-case")]
    struct Foo {
        my_integer: i32,
        is_good: bool,
    }

    let expected = Foo { my_integer: 5, is_good: true };
    run_test_good(r#"(foo my-integer:5 is-good:true)"#, expected);
}

#[test]
fn test_tuple_deserialization() {
    run_test_good("(true 5)", (true, 5));
    run_test_good("()", ());
}

#[test]
fn test_tuple_struct_deserialization() {
    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename="foo")]
    pub struct Foo(bool, i32, bool);
    run_test_good("(foo true 5 false)", Foo(true, 5, false));
}

#[test]
fn test_deserialize_newtype_structu() {
    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename="foo")]
    pub struct Foo(bool);
    run_test_good("(foo true)", Foo(true));
    run_test_good("(foo false)", Foo(false));
}

#[test]
fn test_deserialize_unit_structure() {
    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename="foo")]
    pub struct Foo();
    run_test_good("(foo)", Foo());
    run_test_good("(foo)", Foo());

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename="bar")]
    pub struct Bar{};
    run_test_good("(bar)", Bar{});
    run_test_good("(bar)", Bar{});
}

#[test]
fn test_deserialize_option() {
    run_test_good("nil", None as Option<i32>);
    run_test_good("32", Some(32));
}

#[test]
fn test_default_field() {
    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename="foo", rename_all="kebab-case")]
    struct Foo {
        my_integer: i32,
        is_good: Option<bool>,
    }

    run_test_good("(foo my-integer: 5)", Foo{my_integer: 5, is_good: None});
    run_test_good("(foo my-integer: 5 is-good: true)", Foo{my_integer: 5, is_good: Some(true)});
}

#[test]
fn test_enum() {
    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename="foo", rename_all="kebab-case")]
    enum Foo {
        UnitEnum,
        NewtypeEnum(i32),
        TupleEnum(i32, bool),
        StructEnum {
            x: i32,
            b: bool
        }
    }

    run_test_good("(unit-enum)", Foo::UnitEnum);
    run_test_good("(newtype-enum 5)", Foo::NewtypeEnum(5));
    run_test_good("(tuple-enum 5 true)", Foo::TupleEnum(5, true));
    run_test_good("(struct-enum x:5 b:true)", Foo::StructEnum{x: 5, b: true});
}
