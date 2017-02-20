use super::token::*;
use super::Parseable;

mod scopestack;
pub mod test;
pub mod simplified_test;

use self::scopestack::ScopeStack;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Span<S: Parseable> {
    pub text: S,
    pub lines: S,

    pub line_start: usize,
    pub column_start: usize,
    pub byte_start: usize,

    pub line_end: usize,
    pub column_end: usize,
    pub byte_end: usize,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Sexpr<S: Parseable> {
    List {
        opening_token: TokenInfo<S>,
        closing_token: TokenInfo<S>,

        children: Vec<Sexpr<S>>,
        span: Span<S>,
    },
    UnaryOperator {
        op: TokenInfo<S>,
        child: Box<Sexpr<S>>,
        span: Span<S>,
    },
    Terminal(TokenInfo<S>, Span<S>),
    String(TokenInfo<S>, Span<S>),
}

#[derive(Debug)]
pub enum Diagnostic<S: Parseable> {
    TokenizationError(TokError<S>),
    UnclosedList(Span<S>),
    UnmatchedListClosing(Span<S>, Span<S>),
    UnaryOpWithNoArgument(Span<S>),
    ExtraClosing(Span<S>),
}

pub struct ParseResult<S: Parseable> {
    pub roots: Vec<Sexpr<S>>,
    pub diagnostics: Vec<Diagnostic<S>>,
}

impl<S: Parseable> Sexpr<S> {
    pub fn span(&self) -> &Span<S> {
        match self {
            &Sexpr::List { ref span, .. } => span,
            &Sexpr::UnaryOperator { ref span, .. } => span,
            &Sexpr::String(_, ref span) |
            &Sexpr::Terminal(_, ref span) => span,
        }
    }

    pub fn last_token(&self) -> &TokenInfo<S> {
        match self {
            &Sexpr::List { ref closing_token, .. } => closing_token,
            &Sexpr::UnaryOperator { ref child, .. } => child.last_token(),
            &Sexpr::String(ref token, _) |
            &Sexpr::Terminal(ref token, _) => token,
        }
    }

    pub fn first_token(&self) -> &TokenInfo<S> {
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

impl <S: Parseable> Span<S> {
    pub fn from_token(token: &TokenInfo<S>, string: &S) -> Span<S> {
        let chars = token.string.chars().count();
        let bytes = token.string.len();

        let start_line_pos = find_newline(string.as_bytes(), token.byte_offset, -1);
        let end_line_pos = find_newline(string.as_bytes(), token.byte_offset, 1);
        assert!(end_line_pos >= start_line_pos);
        let line = string.substring(start_line_pos, end_line_pos);

        Span {
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

    pub fn from_spans(start: &Span<S>, end: &Span<S>, string: &S) -> Span<S> {
        let (start, end) = if start.byte_start < end.byte_start {
            (start, end)
        } else {
            (end, start)
        };

        let text = string.substring(start.byte_start, end.byte_end);

        let start_line_pos = find_newline(string.as_bytes(), start.byte_start, -1);
        let end_line_pos = find_newline(string.as_bytes(), end.byte_end, 1);
        assert!(end_line_pos >= start_line_pos);
        let lines = string.substring(start_line_pos, end_line_pos);

        Span {
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

pub fn parse<I, S: Parseable>(string: &S, mut tokens: I) -> ParseResult<S>
    where I: Iterator<Item = TokResult<S, TokenInfo<S>>>
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
