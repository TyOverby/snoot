use super::parse::Span;
use std::fmt::Write;

pub enum ErrorLevel {
    Info,
    Warning,
    Error,
    Custom(String),
}

impl ErrorLevel {
    fn to_str(&self) -> &str {
        match *self {
            ErrorLevel::Info => "info",
            ErrorLevel::Warning => "warning",
            ErrorLevel::Error => "error",
            ErrorLevel::Custom(ref m) => m,
        }
    }
}

pub fn format_error(
    message: &str,
    error_level: &ErrorLevel,
    span: &Span,
    file: &str
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{}: {}", error_level.to_str(), message).unwrap();
    writeln!(&mut out,
             " --> {}:{}:{}",
             file,
             span.line_start,
             span.column_start)
        .unwrap();
    let padding = base_10_length(span.line_end as usize + span.lines.lines().count());
    for (i, line) in span.lines.lines().enumerate() {
        writeln!(&mut out,
                 "{x:pd$} | {st}",
                 pd = padding,
                 x = i + span.line_start as usize,
                 st = line)
            .unwrap();
    }

    out
}

fn base_10_length(x: usize) -> usize {
    if x < 10 {
        return 1;
    }

    return 1 + base_10_length(x / 10);
}

#[cfg(test)]
mod test {
    use super::super::parse::{Sexpr, ParseResult, parse};
    use super::super::token::{tokenize, TokenizationOptions};
    use super::{base_10_length, format_error};
    use tendril::StrTendril;

    fn parse_ok(string: &str) -> Vec<Sexpr> {
        let input: StrTendril = string.into();
        let to = TokenizationOptions::default();
        let cto = to.compile().unwrap();

        let tokens = tokenize(input.clone(), &cto);

        let ParseResult { roots, diagnostics } = parse(&input, tokens);

        if !diagnostics.is_empty() {
            println!("{:?}", diagnostics);
            assert!(diagnostics.is_empty());
        }

        return roots;
    }

    #[test]
    fn test_base_10_length() {
        assert_eq!(base_10_length(0), 1);
        assert_eq!(base_10_length(5), 1);
        assert_eq!(base_10_length(10), 2);
        assert_eq!(base_10_length(100), 3);
    }

    #[test]
    fn test_basic_error() {
        let source = r#"(define map (lambda (xs f)
  (if (nil xs) xs
      (cons (f (car xs))
            (map (cdr xs) f)))))
"#;
        let trees = parse_ok(source);
        let error = format_error("this is the message", &super::ErrorLevel::Info, trees[0].span(), "<anon>");
        println!("{}", error);
        assert_eq!(error.trim(),
r#"info: this is the message
 --> <anon>:1:1
1 | (define map (lambda (xs f)
2 |   (if (nil xs) xs
3 |       (cons (f (car xs))
4 |             (map (cdr xs) f)))))"#);
    }
}
