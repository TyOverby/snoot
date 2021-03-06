# Snoot

<img align="right" width="300px" src="./snoot.png" />

"Parsing s-expressions is easy" you say.  Well yes - but - does *your* hypothetical s-expression parser

* perform no copies on the source text while parsing?
* come with an error-reporting and formatting mechanism?
* keep parsing after encountering the first parse error?
* have name as cute as "Snoot"?

At the very least the answer to the last of those is a definitive "no".

## Zero Copy Parsing
Because the parsing functions take `Tendril`s instead of `String`s, you don't need to worry about
tracking the lifetimes of your parsed values.

Is this magic!?  Maybe!

## Error Reporting
You know how the rust compiler spits out beautiful and informative error messages?
With Snoot, you can easily make error messages just like those!
With one of the Span objects that you get from parsing, you can build error messages with
embedded source without even needing to get your hands dirty.

## Parsing Resiliency
Do your homemade parsers run home crying after they encounter the first parse error?  Sad.
In the face of hardships, Snoot stoicly marches on so that you can report as many errors
to your users as possible.

# Examples

## Error Formatting

```rust
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
```

#### Output
```
error: this is the message
 --> filename.lisp:2:1
2 | (define map (lambda (xs f)
3 |             (if (nil xs) xs
4 |                 (cons (f (car xs))
5 |                 (map (cdr xs) f)))))
```

## Parsing

```rust
extern crate snoot;

const PROGRAM: &'static str = "
(hello world
    (片仮名
        (العَرَبِيَّة‎‎)))
";

fn main() {
    let snoot::ParseResult{roots, diagnostics} = snoot::simple_parse(PROGRAM);
    assert!(diagnostics.is_empty());
    println!("{:#?}", roots);
}
```

#### Output

```rust
[
    List {
        opening_token: TokenInfo { line_number: 2, column_number: 1, byte_offset: 1, typ: ListOpening(0), string: "(" },
        closing_token: TokenInfo { line_number: 4, column_number: 26, byte_offset: 70, typ: ListClosing(0), string: ")" },
        span: Span {
            text: "(hello world\n    (片仮名\n        (العَرَبِيَّة\u{200e}\u{200e})))",
            lines: "\n(hello world\n    (片仮名\n        (العَرَبِيَّة\u{200e}\u{200e})))",
            line_start: 2, column_start: 1, byte_start: 1, line_end: 4, column_end: 27, byte_end: 71
        },
        children: [
            Terminal(
                TokenInfo { line_number: 2, column_number: 2, byte_offset: 2, typ: Atom, string: "hello" },
                Span {
                    text: "hello",
                    lines: "\n(hello world",
                    line_start: 2, column_start: 2, byte_start: 2, line_end: 2, column_end: 7, byte_end: 7
                }
            ),
            Terminal(
                TokenInfo { line_number: 2, column_number: 8, byte_offset: 8, typ: Atom, string: "world" },
                Span {
                    text: "world",
                    lines: "\n(hello world",
                    line_start: 2, column_start: 8, byte_start: 8, line_end: 2, column_end: 13, byte_end: 13
                }
            ),
            List {
                opening_token: TokenInfo { line_number: 3, column_number: 5, byte_offset: 18, typ: ListOpening(0), string: "(" },
                closing_token: TokenInfo { line_number: 4, column_number: 25, byte_offset: 69, typ: ListClosing(0), string: ")" },
                span: Span {
                    text: "(片仮名\n        (العَرَبِيَّة\u{200e}\u{200e}))",
                    lines: "    (片仮名\n        (العَرَبِيَّة\u{200e}\u{200e})))",
                    line_start: 3, column_start: 5, byte_start: 18, line_end: 4, column_end: 26, byte_end: 70
                },
                children: [
                    Terminal(
                        TokenInfo { line_number: 3, column_number: 6, byte_offset: 19, typ: Atom, string: "片仮名" },
                        Span {
                            text: "片仮名",
                            lines: "    (片仮名",
                            line_start: 3, column_start: 6, byte_start: 19, line_end: 3, column_end: 9, byte_end: 28
                        }
                    ),
                    List {
                        opening_token: TokenInfo { line_number: 4, column_number: 9, byte_offset: 37, typ: ListOpening(0), string: "(" },
                        closing_token: TokenInfo { line_number: 4, column_number: 24, byte_offset: 68, typ: ListClosing(0), string: ")" },
                        span: Span {
                            text: "(العَرَبِيَّة\u{200e}\u{200e})",
                            lines: "        (العَرَبِيَّة\u{200e}\u{200e})))",
                            line_start: 4, column_start: 9, byte_start: 37, line_end: 4, column_end: 25, byte_end: 69
                        },
                        children: [
                            Terminal(
                                TokenInfo { line_number: 4, column_number: 10, byte_offset: 38, typ: Atom, string: "العَرَبِيَّة\u{200e}\u{200e}"
                                },
                                Span {
                                    text: "العَرَبِيَّة\u{200e}\u{200e}",
                                    lines: "        (العَرَبِيَّة\u{200e}\u{200e})))",
                                    line_start: 4, column_start: 10, byte_start: 38, line_end: 4, column_end: 24, byte_end: 68
                                }
                            )
                        ]
                    }
                ]
            }
        ]
    }
]
```

Did you know that there are more commits to this readme than there are to the entire rest of the project!
