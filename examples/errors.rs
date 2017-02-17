extern crate snoot;

const PROGRAM: &'static str = "
(define map (lambda (xs f)
            (if (nil xs) xs
                (cons (f (car xs))
                (map (cdr xs) f)))))
";

fn main() {
    let snoot::ParseResult{roots, diagnostics} = snoot::simple_parse(PROGRAM);
    assert!(diagnostics.is_empty());

    // Report an error over the entire program
    let span = roots[0].span();

    let custom_error = snoot::error::format_error(
        "this is the message", &snoot::error::ErrorLevel::Info, span, "filename.lisp");
    println!("{}", custom_error);
}
