use super::parse::Span;
use tendril::StrTendril;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ListType {
    Paren, // ( and )
    Bracket, // [ and ]
    Brace, // { and }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenType {
    ListOpening(ListType),
    ListClosing(ListType),
    Whitespace,
    String,
    Atom,
}

pub type TokResult<OK> = Result<OK, TokError>;

#[derive(Debug, Eq, PartialEq)]
pub enum TokError {
    UnclosedString(Span),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TokenInfo {
    pub line_number: usize,
    pub column_number: usize,
    pub byte_offset: usize,
    pub length: u32,
    pub typ: TokenType,
}

pub struct TokenIterator<'a> {
    splitters: &'a [&'a str],
    remaining: StrTendril,
    line_number: usize,
    column_number: usize,
    byte_offset: usize,
}

impl ListType {
    #[allow(dead_code)]
    pub(crate) fn to_string(&self, open: bool) -> String {
        match (*self, open) {
                (ListType::Paren, true) => "(",
                (ListType::Brace, true) => "{",
                (ListType::Bracket, true) => "[",
                (ListType::Paren, false) => ")",
                (ListType::Brace, false) => "}",
                (ListType::Bracket, false) => "]",
            }
            .into()
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = TokResult<TokenInfo>;

    fn next(&mut self) -> Option<TokResult<TokenInfo>> {
        match next_token(&self.remaining, self.splitters) {
            None => None,
            Some(Err(e)) => Some(Err(e)),
            Some(Ok((typ, s))) => {
                let r = Some(Ok(TokenInfo {
                                    line_number: self.line_number,
                                    column_number: self.column_number,
                                    byte_offset: self.byte_offset,
                                    typ: typ,
                                    length: s.len32(),
                                }));

                for chr in s.as_ref().chars() {
                    if chr == '\n' {
                        self.line_number += 1;
                        self.column_number = 1;
                    } else {
                        self.column_number += 1;
                    }
                }

                let bytes_consumed = s.len();
                self.byte_offset += bytes_consumed;

                // TODO: is this wrong?
                let bytes_consumed = bytes_consumed as u32;
                self.remaining =
                    self.remaining
                        .subtendril(bytes_consumed, self.remaining.len32() - bytes_consumed);

                r
            }
        }
    }
}

fn next_token(string: &StrTendril,
              splitters: &[&str])
              -> Option<TokResult<(TokenType, StrTendril)>> {
    fn idx_until<F>(s: &str, f: F) -> Option<usize>
        where F: Fn(char) -> bool
    {
        s.char_indices()
            .take_while(|&(_, c)| f(c))
            .last()
            .map(|(p, c)| p + c.len_utf8())
    }

    let first = match string.as_ref().chars().next() {
        Some(c) => c,
        None => return None,
    };

    let next = match first {
        c if c.is_whitespace() => {
            let last_idx = idx_until(string.as_ref(), char::is_whitespace).unwrap();
            Some(Ok((TokenType::Whitespace, string.subtendril(0, last_idx as u32))))
        }

        '(' => Some(Ok((TokenType::ListOpening(ListType::Paren), string.subtendril(0, 1)))),
        '{' => Some(Ok((TokenType::ListOpening(ListType::Brace), string.subtendril(0, 1)))),
        '[' => Some(Ok((TokenType::ListOpening(ListType::Bracket), string.subtendril(0, 1)))),
        ')' => Some(Ok((TokenType::ListClosing(ListType::Paren), string.subtendril(0, 1)))),
        '}' => Some(Ok((TokenType::ListClosing(ListType::Brace), string.subtendril(0, 1)))),
        ']' => Some(Ok((TokenType::ListClosing(ListType::Bracket), string.subtendril(0, 1)))),
        _ => {
            let last_idx = idx_until(string.as_ref(), |c| match c {
                '(' | '{' | '[' | ')' | '}' | ']' => false,
                _ if c.is_whitespace() => false,
                _ => true,
            })
                    .unwrap();
            let mut substr = string.subtendril(0, last_idx as u32);
            let mut lowest = None;
            for splitter in splitters {
                lowest = match (lowest, substr.as_ref().find(splitter)) {
                    (_, Some(0)) => {
                        substr = string.subtendril(0, splitter.len() as u32);
                        lowest = None;
                        break;
                    }
                    (None, Some(l)) => Some(l),
                    (Some(l), None) => Some(l),
                    (Some(c), Some(n)) => Some(::std::cmp::min(c, n)),
                    (None, None) => None,
                };
            }

            if let Some(new_low) = lowest {
                substr = string.subtendril(0, new_low as u32);
            }

            Some(Ok((TokenType::Atom, substr)))
        }
    };
    return next;
}


pub fn tokenize<'a>(string: StrTendril, seps: &'a [&'a str]) -> TokenIterator {
    TokenIterator {
        splitters: seps,
        remaining: string,
        line_number: 1,
        column_number: 1,
        byte_offset: 0,
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn all_ok(string: &str) -> Vec<TokenInfo> {
        tokenize(string.into(), &[])
            .collect::<Result<_, _>>()
            .unwrap()
    }
    fn all_ok_split<'a, 'b>(string: &'a str, sp: &'b [&'b str]) -> Vec<TokenInfo> {
        tokenize(string.into(), sp)
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn empty() {
        assert_eq!(all_ok(""), vec![]);
    }

    #[test]
    fn single_open_paren() {
        assert_eq!(all_ok("("),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            // TODO: change from 0 to something meaningful
                            typ: TokenType::ListOpening(ListType::Paren),
                            length: 1,
                        }]);
    }

    #[test]
    fn single_closing_paren() {
        assert_eq!(all_ok(")"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListClosing(ListType::Paren),
                            length: 1,
                        }]);
    }

    #[test]
    fn paired_parens() {
        assert_eq!(all_ok("()"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListOpening(ListType::Paren),
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 2,
                            byte_offset: 1,
                            typ: TokenType::ListClosing(ListType::Paren),
                            length: 1,
                        }])
    }

    #[test]
    fn nested_parens() {

        assert_eq!(all_ok("(())"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListOpening(ListType::Paren),
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 2,
                            byte_offset: 1,
                            typ: TokenType::ListOpening(ListType::Paren),
                            length: 1,
                        },

                        TokenInfo {
                            line_number: 1,
                            column_number: 3,
                            byte_offset: 2,
                            typ: TokenType::ListClosing(ListType::Paren),
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 4,
                            byte_offset: 3,
                            typ: TokenType::ListClosing(ListType::Paren),
                            length: 1,
                        }])
    }

    #[test]
    fn double_parens() {
        assert_eq!(all_ok("(("),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListOpening(ListType::Paren),
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 2,
                            byte_offset: 1,
                            typ: TokenType::ListOpening(ListType::Paren),
                            length: 1,
                        }])
    }

    #[test]
    fn unary_literal() {
        assert_eq!(all_ok("@"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 1,
                        }]);
    }

    #[test]
    fn numbers() {
        assert_eq!(all_ok("123"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 3,
                        }]);

        assert_eq!(all_ok("-123"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 4,
                        }]);

        assert_eq!(all_ok("123.456"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 7,
                        }]);

        assert_eq!(all_ok("+123.456"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 8,
                        }]);
    }

    #[test]
    fn identifier() {
        assert_eq!(all_ok("hello-world"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 11,
                        }]);

        assert_eq!(all_ok("a"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 1,
                        }]);

        assert_eq!(all_ok("片仮名"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: "片仮名".len() as u32,
                        }]);

        assert_eq!(all_ok("-"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 1,
                        }]);
    }

    #[test]
    fn ident_white_ident() {
        assert_eq!(all_ok("hello world"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 5,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 6,
                            byte_offset: 5,
                            typ: TokenType::Whitespace,
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 7,
                            byte_offset: 6,
                            typ: TokenType::Atom,
                            length: 5,
                        }]);
    }

    #[test]
    fn split() {
        assert_eq!(all_ok_split("hello-world", &["-"]),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 5,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 6,
                            byte_offset: 5,
                            typ: TokenType::Atom,
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 7,
                            byte_offset: 6,
                            typ: TokenType::Atom,
                            length: 5,
                        }]);

        assert_eq!(all_ok_split("a:b", &[":"]),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 2,
                            byte_offset: 1,
                            typ: TokenType::Atom,
                            length: 1,
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 3,
                            byte_offset: 2,
                            typ: TokenType::Atom,
                            length: 1,
                        }]);
    }
}
