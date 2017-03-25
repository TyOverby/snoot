extern crate snoot;

const PROGRAM: &'static str = "
(hello world
    (片仮名
        (العَرَبِيَّة‎‎)))
";

fn main() {
    let snoot::Result{roots, diagnostics} = snoot::simple_parse(PROGRAM, &[]);
    assert!(diagnostics.is_empty());
    println!("{:#?}", roots);
}
