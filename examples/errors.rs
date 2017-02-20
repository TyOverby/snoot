extern crate snoot;

use snoot::simple_parse;
use snoot::error::{ErrorBuilder, ErrorLevel};

const PROGRAM: &'static str = "
(define map (lambda (xs f)
            (if (nil xs) xs
                (cons (f (car xs))
                (map (cdr xs) f)))))
";

fn main() {
    let snoot::ParseResult{roots, diagnostics} = simple_parse(PROGRAM);
    assert!(diagnostics.is_empty());

    // Report an error over the entire program
    let span = roots[0].span();

    let error = ErrorBuilder::new("this is the message", span.clone())
        .with_file_name("filename.lisp")
        .with_error_level(ErrorLevel::Error)
        .build();

    println!("{}", error);
}
