use tendril::StrTendril;
use super::token::*;

mod scopestack;
mod test;
mod simplified_test;

use self::scopestack::ScopeStack;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Span {
    text: StrTendril,

    line_start: u32,
    column_start: u32,
    byte_start: u32,

    line_end: u32,
    column_end: u32,
    byte_end: u32,
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

impl Span {
    pub fn from_token(token: &TokenInfo) -> Span {
        let chars = token.string.chars().count() as u32;
        let bytes = token.string.len() as u32;

        Span {
            text: token.string.clone(),
            line_start: token.line_number,
            column_start: token.column_number,
            byte_start: token.byte_offset,

            line_end: token.line_number,
            column_end: token.column_number + chars,
            byte_end: token.byte_offset + bytes,
        }
    }

    pub fn from_spans(start: &Span, end: &Span, string: &StrTendril) -> Span {
        let (start, end) = if start.byte_start < end.byte_start {
            (start, end)
        } else {
            (end, start)
        };

        let text = string.subtendril(start.byte_start, end.byte_end - start.byte_start);
        println!("combining {:#?} and {:#?} to get {}", start, end, text);

        Span {
            text: text,
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
                let span = Span::from_token(&token);
                scopestack.put(Sexpr::String(token, span));
            }
            TokenType::Number => {
                let span = Span::from_token(&token);
                scopestack.put(Sexpr::Number(token, span));
            }
            TokenType::Identifier => {
                let span = Span::from_token(&token);
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
