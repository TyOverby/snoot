use super::token::*;
use super::error::{Error, ErrorBuilder, ErrorLevel};
use tendril::StrTendril;

mod scopestack;
pub mod test;
pub mod simplified_test;

use self::scopestack::ScopeStack;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub(crate) struct StartEnd {
    pub start: u32,
    pub end: u32,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Span {
    pub(crate) full_text: StrTendril,

    pub(crate) text_bytes: StartEnd,
    pub(crate) lines_bytes: StartEnd,

    pub(crate) lines_covered: StartEnd,
    pub(crate) columns: StartEnd,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum SexprKind {
    List, UnaryOperator, Terminal, String
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Sexpr {
    List {
        list_type: ListType,
        opening_token: TokenInfo,
        closing_token: TokenInfo,

        children: Vec<Sexpr>,
        span: Span,
    },
    UnaryOperator {
        op: TokenInfo,
        child: Box<Sexpr>,
        span: Span,
    },
    Terminal(TokenInfo, Span),
    String(TokenInfo, Span),
}

#[derive(Debug)]
pub enum Diagnostic {
    TokenizationError(TokError),
    UnclosedList(Span),
    UnmatchedListClosing(Span, Span),
    ExtraClosing(Span),
    WrongClosing {
        opening_span: Span,
        closing_span: Span,
        expected_list_type: ListType,
        actual_list_type: ListType,
    }
}

pub struct ParseResult {
    pub roots: Vec<Sexpr>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Diagnostic {
    pub fn into_error(self, filename: Option<String>) -> Error {
        match self {
            Diagnostic::TokenizationError(TokError::UnclosedString(_span)) => {
                unreachable!();
            }
            Diagnostic::ExtraClosing(span) => {
                let builder = ErrorBuilder::new("extra list closing", &span);
                let builder = if let Some(f) = filename {
                    builder.with_file_name(f)
                } else { builder };

                builder.with_error_level(ErrorLevel::Error).build()
            }
            Diagnostic::UnmatchedListClosing(start, end) => {
                let span = Span::from_spans(&start, &end);
                let builder = ErrorBuilder::new("unmatched list closing", &span);
                let builder = if let Some(f) = filename {
                    builder.with_file_name(f)
                } else { builder };

                builder.with_error_level(ErrorLevel::Error).build()
            }
            Diagnostic::UnclosedList(span) => {
                let builder = ErrorBuilder::new("unclosed list", &span);
                let builder = if let Some(f) = filename {
                    builder.with_file_name(f)
                } else { builder };

                builder.with_error_level(ErrorLevel::Error).build()
            }
            Diagnostic::WrongClosing {
                opening_span,
                closing_span,
                expected_list_type,
                actual_list_type,
            } => {
                let text = format!("Expected {} but found {}",
                    expected_list_type.to_string(false),
                    actual_list_type.to_string(false));
                let builder = ErrorBuilder::new(text, &Span::from_spans(&opening_span, &closing_span));
                let builder = if let Some(f) = filename {
                    builder.with_file_name(f)
                } else { builder };
                builder.with_error_level(ErrorLevel::Error).build()
            }
        }
    }
}

impl Sexpr {
    pub fn kind(&self) -> SexprKind {
        match self {
            &Sexpr::List {..} => SexprKind::List,
            &Sexpr::UnaryOperator { .. } => SexprKind::UnaryOperator,
            &Sexpr::String(_, _) => SexprKind::String,
            &Sexpr::Terminal(_, _) => SexprKind::Terminal,
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            &Sexpr::List { ref span, .. } => span,
            &Sexpr::UnaryOperator { ref span, .. } => span,
            &Sexpr::String(_, ref span) |
            &Sexpr::Terminal(_, ref span) => span,
        }
    }

    pub fn last_token(&self) -> &TokenInfo {
        match self {
            &Sexpr::List { ref closing_token, .. } => closing_token,
            &Sexpr::UnaryOperator { ref child, .. } => child.last_token(),
            &Sexpr::String(ref token, _) |
            &Sexpr::Terminal(ref token, _) => token,
        }
    }

    pub fn first_token(&self) -> &TokenInfo {
        match self {
            &Sexpr::List { ref opening_token, .. } => opening_token,
            &Sexpr::UnaryOperator { ref op, .. } => op,
            &Sexpr::String(ref token, _) |
            &Sexpr::Terminal(ref token, _) => token,
        }
    }
}

fn find_newline(t: &[u8], pos: u32, direction: isize) -> u32 {
    // We're searching backwards and we've hit the start of the buffer
    if pos == 0 && direction == -1 {
        if t[0] == b'\n' {
            return 1
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

    return find_newline(t, ((pos as isize) + (direction as isize)) as u32, direction);
}

impl Span {
    pub fn empty() -> Span {
        Span {
            full_text: "".into(),

            text_bytes: StartEnd {start: 0, end: 0},
            lines_bytes: StartEnd { start: 0, end: 0},
            lines_covered: StartEnd { start: 0, end: 0},
            columns: StartEnd { start: 0, end: 0 },
        }
    }

    pub fn lines(&self) -> StrTendril {
        let StartEnd {start, end} = self.lines_bytes;
        self.full_text.subtendril(start, end - start)
    }

    pub fn text(&self) -> StrTendril {
        let StartEnd {start, end} = self.text_bytes;
        self.full_text.subtendril(start, end - start)
    }

    pub fn from_token(token: &TokenInfo, string: &StrTendril) -> Span {
        let chars = string.subtendril(token.byte_offset as u32, token.length).len();
        let bytes = token.length;

        let start_line_pos = find_newline(string.as_bytes(), token.byte_offset as u32, -1);
        let end_line_pos = find_newline(string.as_bytes(), token.byte_offset as u32, 1);
        assert!(end_line_pos >= start_line_pos);

        Span {
            full_text: string.clone(),
            text_bytes: StartEnd {
                start: token.byte_offset as u32,
                end: token.byte_offset as u32 + bytes as u32,
            },
            lines_bytes: StartEnd {
                start: start_line_pos as u32,
                end: end_line_pos as u32,
            },
            lines_covered: StartEnd {
                start: token.line_number as u32,
                end: token.line_number as u32,
            },
            columns: StartEnd {
                start: token.column_number as u32,
                end: token.column_number as u32 + chars as u32,
            }
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

        Span {

            full_text: start.full_text.clone(),
            text_bytes: StartEnd {
                start: start.text_bytes.start,
                end: end.text_bytes.end,
            },
            lines_bytes: StartEnd {
                start: start_line_pos as u32,
                end: end_line_pos as u32,
            },
            lines_covered: StartEnd {
                start: start.lines_covered.start,
                end: end.lines_covered.end,
            },
            columns: StartEnd {
                start: start.columns.start,
                end: end.columns.end,
            }
        }

    }
}

pub fn parse<I>(string: &StrTendril, mut tokens: I) -> ParseResult
    where I: Iterator<Item = TokResult<TokenInfo>>
{

    let mut diagnostics = vec![];
    let mut scopestack = ScopeStack::new(string.clone());

    loop {
        let token = match tokens.next() {
            Some(Ok(t)) => t,
            Some(Err(e)) => {
                diagnostics.push(Diagnostic::TokenizationError(e));
                continue;
            }
            None => break,
        };

        match token.typ {
            TokenType::String => {
                let span = Span::from_token(&token, string);
                scopestack.put(Sexpr::String(token, span));
            }
            TokenType::Atom => {
                let span = Span::from_token(&token, string);
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

    ParseResult {
        roots: out,
        diagnostics: diagnostics,
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
