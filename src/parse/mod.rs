use super::token::*;
use super::error::{Error, ErrorBuilder, ErrorLevel};
use tendril::StrTendril;

mod scopestack;
pub mod test;
pub mod simplified_test;

use self::scopestack::ScopeStack;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Span {
    pub full_text: StrTendril,
    pub text: StrTendril,
    pub lines: StrTendril,

    pub line_start: usize,
    pub column_start: usize,
    pub byte_start: usize,

    pub line_end: usize,
    pub column_end: usize,
    pub byte_end: usize,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum SexprKind {
    List, UnaryOperator, Terminal, String
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Sexpr {
    List {
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

fn find_newline(t: &[u8], pos: usize, direction: isize) -> usize {
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

    return find_newline(t, ((pos as isize) + (direction as isize)) as usize, direction);
}

impl Span {
    pub fn empty() -> Span {
        Span {
            full_text: "".into(),
            text: "".into(),
            lines: "".into(),

            line_start: 0,
            column_start: 0,
            byte_start: 0,

            line_end: 0,
            column_end: 0,
            byte_end: 0,
        }
    }
    pub fn from_token(token: &TokenInfo, string: &StrTendril) -> Span {
        let chars = token.string.chars().count();
        let bytes = token.string.len();

        let start_line_pos = find_newline(string.as_bytes(), token.byte_offset, -1);
        let end_line_pos = find_newline(string.as_bytes(), token.byte_offset, 1);
        assert!(end_line_pos >= start_line_pos);
        let line = string.subtendril(start_line_pos as u32, (end_line_pos - start_line_pos) as u32);

        Span {
            full_text: string.clone(),
            text: token.string.clone(),
            lines: line,
            line_start: token.line_number,
            column_start: token.column_number,
            byte_start: token.byte_offset,

            line_end: token.line_number,
            column_end: token.column_number + chars,
            byte_end: token.byte_offset + bytes,
        }
    }

    pub fn from_spans(start: &Span, end: &Span) -> Span {
        let string = start.full_text.clone();
        let (start, end) = if start.byte_start < end.byte_start {
            (start, end)
        } else {
            (end, start)
        };

        let text = string.subtendril(start.byte_start as u32, (end.byte_end - start.byte_start) as u32);

        let start_line_pos = find_newline(string.as_bytes(), start.byte_start, -1);
        let end_line_pos = find_newline(string.as_bytes(), end.byte_end, 1);
        assert!(end_line_pos >= start_line_pos);
        let lines = string.subtendril(start_line_pos as u32, (end_line_pos - start_line_pos) as u32);

        Span {
            full_text: start.full_text.clone(),
            text: text,
            lines: lines,
            line_start: start.line_start,
            column_start: start.column_start,
            byte_start: start.byte_start,

            line_end: end.line_end,
            column_end: end.column_end,
            byte_end: end.byte_end,
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
            TokenType::ListOpening(_n) => {
                scopestack.open_list(token);
            }
            TokenType::ListClosing(_n) => {
                scopestack.close(Some(token), &mut diagnostics);
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
