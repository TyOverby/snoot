use tendril::StrTendril;
use super::token::*;

mod scopestack;
pub mod test;
pub mod simplified_test;

use self::scopestack::ScopeStack;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Span {
    pub text: StrTendril,
    pub lines: StrTendril,

    pub line_start: u32,
    pub column_start: u32,
    pub byte_start: u32,

    pub line_end: u32,
    pub column_end: u32,
    pub byte_end: u32,
}

#[derive(Eq, PartialEq, Debug)]
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
    Number(TokenInfo, Span),
    String(TokenInfo, Span),
    Ident(TokenInfo, Span),
}

#[derive(Debug)]
pub enum Diagnostic {
    TokenizationError(TokError),
    UnclosedList(Span),
    UnmatchedListClosing(Span, Span),
    UnaryOpWithNoArgument(Span),
    ExtraClosing(Span),
}

pub struct ParseResult {
    pub roots: Vec<Sexpr>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Sexpr {
    pub fn span(&self) -> &Span {
        match self {
            &Sexpr::List { ref span, .. } => span,
            &Sexpr::UnaryOperator { ref span, .. } => span,
            &Sexpr::Number(_, ref span) |
            &Sexpr::String(_, ref span) |
            &Sexpr::Ident(_, ref span) => span,
        }
    }

    pub fn last_token(&self) -> &TokenInfo {
        match self {
            &Sexpr::List { ref closing_token, .. } => closing_token,
            &Sexpr::UnaryOperator { ref child, .. } => child.last_token(),
            &Sexpr::Number(ref token, _) |
            &Sexpr::String(ref token, _) |
            &Sexpr::Ident(ref token, _) => token,
        }
    }

    pub fn first_token(&self) -> &TokenInfo {
        match self {
            &Sexpr::List { ref opening_token, .. } => opening_token,
            &Sexpr::UnaryOperator { ref op, .. } => op,
            &Sexpr::Number(ref token, _) |
            &Sexpr::String(ref token, _) |
            &Sexpr::Ident(ref token, _) => token,
        }
    }
}

fn find_newline(
    t: &[u8],
    pos: u32,
    direction: i32
) -> u32 {
    // We're searching backwards and we've hit the start of the buffer
    if pos == 0 && direction == -1 {
        return 0;
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

    return find_newline(t, ((pos as i64) + (direction as i64)) as u32, direction);
}

impl Span {
    pub fn from_token(
        token: &TokenInfo,
        string: &StrTendril
    ) -> Span {
        let chars = token.string.chars().count() as u32;
        let bytes = token.string.len() as u32;

        let start_line_pos = find_newline(string.as_bytes(), token.byte_offset, -1);
        let end_line_pos = find_newline(string.as_bytes(), token.byte_offset, 1);
        assert!(end_line_pos >= start_line_pos);
        let line = string.subtendril(start_line_pos, end_line_pos - start_line_pos);

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

    pub fn from_spans(
        start: &Span,
        end: &Span,
        string: &StrTendril
    ) -> Span {
        let (start, end) = if start.byte_start < end.byte_start {
            (start, end)
        } else {
            (end, start)
        };

        let text = string.subtendril(start.byte_start, end.byte_end - start.byte_start);

        let start_line_pos = find_newline(string.as_bytes(), start.byte_start, -1);
        let end_line_pos = find_newline(string.as_bytes(), end.byte_end, 1);
        assert!(end_line_pos >= start_line_pos);
        let lines = string.subtendril(start_line_pos, end_line_pos - start_line_pos);

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

pub fn parse<I>(
    string: &StrTendril,
    mut tokens: I
) -> ParseResult
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
            TokenType::Number => {
                let span = Span::from_token(&token, string);
                scopestack.put(Sexpr::Number(token, span));
            }
            TokenType::Identifier => {
                let span = Span::from_token(&token, string);
                scopestack.put(Sexpr::Ident(token, span));
            }
            TokenType::UnaryOperator => {
                scopestack.open_unary(token);
            }
            TokenType::Whitespace => { /* do nothing for now */ }
            TokenType::ListOpening => {
                scopestack.open_list(token);
            }
            TokenType::ListClosing => {
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
