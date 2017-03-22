use std::fmt::{self, Display, Formatter, Debug};

use parse::Span;

pub struct Error(ErrorBuilder);

pub enum ErrorLevel {
    Info,
    Warn,
    Error,
    Custom(String),
}

pub struct ErrorBuilder {
    pub message: String,
    pub annotations: Vec<ErrorAnnotation>,
    pub global_span: Span,
    pub padding: usize,

    // optional
    pub min_gap: Option<usize>,
    pub filename: Option<String>,
    pub error_level: Option<ErrorLevel>,
}

pub struct ErrorAnnotation {
    pub message: String,
    pub span: Span,
}

impl ErrorLevel {
    fn as_str(&self) -> &str {
        match self {
            &ErrorLevel::Info => "info",
            &ErrorLevel::Warn => "warn",
            &ErrorLevel::Error => "error",
            &ErrorLevel::Custom(ref s) => s,

        }
    }
}

impl ErrorBuilder {
    pub fn new<T: Into<String>>(message: T, span: &Span) -> ErrorBuilder {
        ErrorBuilder {
            message: message.into(),
            annotations: vec![],
            global_span: span.clone(),
            padding: 2,

            min_gap: None,
            filename: None,
            error_level: None,
        }
    }

    pub fn with_error_level(mut self, level: ErrorLevel) -> ErrorBuilder {
        self.error_level = Some(level);
        self
    }

    pub fn with_file_name<T: Into<String>>(mut self, name: T) -> ErrorBuilder {
        self.filename = Some(name.into());
        self
    }

    pub fn with_min_gap(mut self, gap: usize) -> ErrorBuilder {
        self.min_gap = Some(gap);
        self
    }

    pub fn with_garunteed_padding(mut self, padding: usize) -> ErrorBuilder {
        self.padding = padding;
        self
    }

    pub fn add_annotation(mut self, annotation: ErrorAnnotation) -> ErrorBuilder {
        self.annotations.push(annotation);
        self
    }

    pub fn build(self) -> Error {
        Error(self)
    }
}

impl ErrorAnnotation {
    pub fn new(message: String, span: Span) -> ErrorAnnotation {
        ErrorAnnotation {
            message: message,
            span: span,
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let &Error(ref builder) = self;

        // "error" message
        if let &Some(ref error_level) = &builder.error_level {
            writeln!(f, "{}: {}", error_level.as_str(), builder.message)?;
        } else {
            writeln!(f, "{}", builder.message)?;
        }

        // File, line number, column number information
        if let &Some(ref file) = &builder.filename {
            writeln!(f,
                     " --> {}:{}:{}",
                     file,
                     builder.global_span.line_start,
                     builder.global_span.column_start)?;
        } else {
            writeln!(f,
                     " --> {}:{}",
                     builder.global_span.line_start,
                     builder.global_span.column_start)?;
        }

        let padding = base_10_length(builder.global_span.line_end +
                                     builder.global_span.lines.as_ref().lines().count());

        let iter = builder.global_span
            .lines
            .as_ref()
            .lines()
            .enumerate()
            .map(|(i, line)| (i + builder.global_span.line_start, line));

        let mut skipped_streak = 0;
        for (i, line) in iter {
            let get_span = &get_span;
            let spans = builder.annotations.iter().map(get_span);
            if should_skip(i,
                           skipped_streak,
                           builder.padding,
                           builder.min_gap,
                           &builder.global_span,
                           spans) {
                skipped_streak += 1;
            } else {
                if skipped_streak > 0 {
                    write!(f, "{x:pd$} | ", pd = padding, x = "~")?;
                    writeln!(f,
                             "skipped <{}> through <{}>",
                             i - 1 - skipped_streak,
                             i - 1)?;
                }
                skipped_streak = 0;
                writeln!(f, "{x:pd$} | {st}", pd = padding, x = i, st = line)?;
            }
        }

        Ok(())
    }
}

fn get_span<'a>(ann: &'a ErrorAnnotation) -> &'a Span {
    &ann.span
}

fn should_skip<'a, I>(line: usize,
                                         already_skipped: usize,
                                         padding: usize,
                                         max_gap_size: Option<usize>,
                                         global_span: &'a Span,
                                         annot_span: I)
                                         -> bool
    where I: Iterator<Item = &'a Span> + Clone
{
    let max_gap = match max_gap_size {
        Some(t) => t,
        None => return false,
    };

    let dist = line_dist_all(line,
                             ::std::iter::once(global_span).chain(annot_span.clone()))
            .unwrap();

    if dist <= padding {
        return false;
    }

    let mut skip_count = already_skipped + 1;
    let mut i = 1;
    while should_skip(line + i,
                      skip_count,
                      padding,
                      max_gap_size,
                      global_span,
                      annot_span.clone()) {
        skip_count += 1;
        i += 1;
    }

    if skip_count < max_gap {
        return false;
    }

    return true;
}

pub fn base_10_length(mut x: usize) -> usize {
    let mut r = 1;
    while x >= 10 {
        r = r + 1;
        x /= 10;
    }
    r
}

fn line_dist_all<'a, I>(line: usize, i: I) -> Option<usize>
    where I: Iterator<Item = &'a Span>
{
    i.map(|s| line_distance(line, s)).min()
}

// Return the distance to
fn line_distance(line: usize, span: &Span) -> usize {

    let dist_start = (line as isize - span.line_start as isize).abs() as usize;
    let dist_end = (line as isize - span.line_end as isize).abs() as usize;

    let shortest_dist = ::std::cmp::min(dist_start, dist_end);

    shortest_dist
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

    let ::parse::ParseResult { roots, diagnostics } = ::simple_parse(source, &[]);
    assert!(diagnostics.is_empty());

    let error = ErrorBuilder::new("this is the message", roots[0].span())
        .with_file_name("<anon>")
        .with_error_level(ErrorLevel::Info)
        .build();

    println!("{}", error);
    assert_eq!(error.to_string().trim(),
               r#"info: this is the message
 --> <anon>:1:1
1 | (define map (lambda (xs f)
2 |   (if (nil xs) xs
3 |       (cons (f (car xs))
4 |             (map (cdr xs) f)))))"#);
}

