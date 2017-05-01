use std::rc::Rc;

use super::token::*;
use super::diagnostic::{Diagnostic, DiagnosticLevel};
use tendril::StrTendril;
use {Result, Sexpr};

mod scopestack;
pub mod test;
pub mod simplified_test;

use self::scopestack::ScopeStack;

#[derive(Eq, PartialEq, Debug, Clone, Copy, Ord, PartialOrd)]
pub struct StartEnd {
    pub start: u32,
    pub end: u32,
}

#[derive(Eq, PartialEq, Debug, Clone, PartialOrd, Ord)]
pub struct Span {
    pub text_bytes: StartEnd,

    pub lines_covered: StartEnd,
    pub columns: StartEnd,

    pub full_text: StrTendril,
    pub file: Option<Rc<String>>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum SexprKind {
    List,
    UnaryOperator,
    Terminal,
    String,
}

#[derive(Debug)]
pub enum ParseDiagnostic {
    TokenizationError(TokError),
    UnclosedList(Span),
    ExtraClosing(Span),
    WrongClosing {
        opening_span: Span,
        closing_span: Span,
        expected_list_type: ListType,
        actual_list_type: ListType,
    },
}

impl ParseDiagnostic {
    pub fn into_diagnostic(self) -> Diagnostic {
        match self {
            ParseDiagnostic::TokenizationError(TokError::UnclosedString(_span)) => {
                unreachable!();
            }
            ParseDiagnostic::ExtraClosing(span) => {
                let builder = Diagnostic::new("extra list closing", &span);
                builder.with_error_level(DiagnosticLevel::Error)
            }
            ParseDiagnostic::UnclosedList(span) => {
                let builder = Diagnostic::new("unclosed list", &span);
                builder.with_error_level(DiagnosticLevel::Error)
            }
            ParseDiagnostic::WrongClosing {
                opening_span,
                closing_span,
                expected_list_type,
                actual_list_type,
            } => {
                let text = format!("Expected {} but found {}",
                    expected_list_type.to_string(false),
                    actual_list_type.to_string(false));
                let builder =
                    Diagnostic::new(text, &Span::from_spans(&opening_span, &closing_span));
                builder.with_error_level(DiagnosticLevel::Error)
            }
        }
    }
}


fn find_newline(t: &[u8], mut pos: u32, direction: isize) -> u32 {
    loop {
        // We're searching backwards and we've hit the start of the buffer
        if pos == 0 && direction == -1 {
            if t[0] == b'\n' {
                return 1;
            } else {
                return 0;
            }
        }

        // We're searching forwards and we've hit the end of the buffer
        if pos as usize == t.len() && direction == 1 {
            return pos;
        }

        match (t[pos as usize], direction) {
            (b'\n', -1) => return pos + 1,
            (b'\n', 1) => return pos,
            _ => {}
        }

        pos = (pos as isize + direction as isize) as u32;
    }
}

impl <'a> ::std::iter::FromIterator<&'a Span> for Span {
    fn from_iter<I: IntoIterator<Item=&'a Span>>(iter: I) -> Span {
        let mut base = None;

        for s in iter {
            base = Some(match base.take() {
                Some(b) => Span::from_spans(&b, s),
                None => s.clone()
            })
        }

        base.unwrap_or_else(Span::empty)
    }
}

impl Span {
    pub fn empty() -> Span {
        Span {
            full_text: "".into(),
            file: None,

            text_bytes: StartEnd { start: 0, end: 0 },
            lines_covered: StartEnd { start: 0, end: 0 },
            columns: StartEnd { start: 0, end: 0 },
        }
    }

    pub fn lines(&self) -> StrTendril {
        let start = find_newline(self.text().as_bytes(), self.text_bytes.start, -1);
        let end = find_newline(self.text().as_bytes(), self.text_bytes.end, 1);
        self.full_text.subtendril(start, end - start)
    }

    pub fn text(&self) -> StrTendril {
        let StartEnd { start, end } = self.text_bytes;
        self.full_text.subtendril(start, end - start)
    }

    pub fn from_token(token: &TokenInfo, string: &StrTendril, file: &Option<Rc<String>>) -> Span {
        let chars = string
            .subtendril(token.byte_offset as u32, token.length)
            .len();
        let bytes = token.length;

        let start_line_pos = find_newline(string.as_bytes(), token.byte_offset as u32, -1);
        let end_line_pos = find_newline(string.as_bytes(), token.byte_offset as u32, 1);
        assert!(end_line_pos >= start_line_pos);

        Span {
            file: file.clone(),
            full_text: string.clone(),
            text_bytes: StartEnd {
                start: token.byte_offset as u32,
                end: token.byte_offset as u32 + bytes as u32,
            },
            lines_covered: StartEnd {
                start: token.line_number as u32,
                end: token.line_number as u32,
            },
            columns: StartEnd {
                start: token.column_number as u32,
                end: token.column_number as u32 + chars as u32,
            },
        }
    }

    pub fn from_spans(start: &Span, end: &Span) -> Span {
        let string = start.full_text.clone();
        let (start, end) = if start.text_bytes.start < end.text_bytes.start {
            (start, end)
        } else {
            (end, start)
        };

        let start_line_pos = find_newline(string.as_bytes(), start.text_bytes.start, -1);
        let end_line_pos = find_newline(string.as_bytes(), end.text_bytes.end, 1);
        assert!(end_line_pos >= start_line_pos);
        debug_assert!(start.file == end.file);

        Span {
            full_text: start.full_text.clone(),
            file: start.file.clone(),
            text_bytes: StartEnd {
                start: start.text_bytes.start,
                end: end.text_bytes.end,
            },
            lines_covered: StartEnd {
                start: start.lines_covered.start,
                end: end.lines_covered.end,
            },
            columns: StartEnd {
                start: start.columns.start,
                end: end.columns.end,
            },
        }

    }
}

pub fn parse<I>(string: &StrTendril, mut tokens: I, file: Option<String>) -> Result
    where I: Iterator<Item = TokResult<TokenInfo>>
{
    let file = file.map(Rc::new);
    let mut diagnostics = vec![];
    let mut scopestack = ScopeStack::new(string.clone(), &file);

    loop {
        let token = match tokens.next() {
            Some(Ok(t)) => t,
            Some(Err(e)) => {
                diagnostics.push(ParseDiagnostic::TokenizationError(e));
                continue;
            }
            None => break,
        };

        match token.typ {
            TokenType::String => {
                let span = Span::from_token(&token, string, &file);
                scopestack.put(Sexpr::String(token, span));
            }
            TokenType::Atom => {
                let span = Span::from_token(&token, string, &file);
                scopestack.put(Sexpr::Terminal(token, span));
            }
            // TODO
            //TokenType::UnaryOperator => {
            //    scopestack.open_unary(token);
            //}
            TokenType::Whitespace => { /* do nothing for now */ }
            TokenType::ListOpening(typ) => {
                scopestack.open_list(typ, token);
            }
            TokenType::ListClosing(typ) => {
                scopestack.close(Some((typ, token)), &mut diagnostics);
            }
        }
    }

    let out = scopestack.end(&mut diagnostics);

    Result {
        roots: out,
        diagnostics: diagnostics
            .into_iter()
            .map(ParseDiagnostic::into_diagnostic)
            .collect(),
    }
}

#[test]
fn find_newline_test() {
    let string = b"abc\n123\nxyz";
    {
        let st = find_newline(string, 5, -1) as usize;
        let en = find_newline(string, 5, 1) as usize;
        assert_eq!(st, 4);
        assert_eq!(en, 7);
        assert_eq!(&string[st..en], b"123");
    }
    {
        let st = find_newline(string, 1, -1) as usize;
        let en = find_newline(string, 1, 1) as usize;
        assert_eq!(st, 0);
        assert_eq!(en, 3);
        assert_eq!(&string[st..en], b"abc");
    }
    {
        let st = find_newline(string, 9, -1) as usize;
        let en = find_newline(string, 9, 1) as usize;
        assert_eq!(st, 8);
        assert_eq!(en, 11);
        assert_eq!(&string[st..en], b"xyz");
    }
}
