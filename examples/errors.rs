#[macro_use]
extern crate snoot;

use snoot::simple_parse;

const PROGRAM: &'static str = "
(define map (lambda (xs f)
            (if (nil xs) xs
                (cons (f (car xs))
                (map (cdr xs) f)))))
";

fn main() {
    let snoot::Result { roots, diagnostics } = simple_parse(PROGRAM, &[], Some("filename.lisp"));
    assert!(diagnostics.is_empty());

    // Report an error over the entire program
    let span = roots[0].span();

    let error = diagnostic!(span, "this is the message");

    println!("{}", error);
}
