use ::diagnostic::Diagnostic;
use std::fmt::Display;
use ::parse::Span;

pub fn nothing_found<S: Display>(span: &Span, expected: S) -> Diagnostic {
    diagnostic!(span, "expected {} but found no values", expected)
}

pub fn multiple_values_found<S: Display>(span: &Span, expected: S) -> Diagnostic {
    diagnostic!(span, "expected {} but found multiple values", expected)
}
