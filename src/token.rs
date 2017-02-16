use super::Parseable;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenType {
    ListOpening(u8),
    ListClosing(u8),
    Whitespace,
    String,
    Atom,
}

pub type TokResult<S, OK> = Result<OK, TokError<S>>;

#[derive(Debug, Eq, PartialEq)]
pub enum TokError<S: Parseable> {
    UnclosedString(S),
    NoMatch(S),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenInfo<S: Parseable> {
    pub line_number: usize,
    pub column_number: usize,
    pub byte_offset: usize,
    pub typ: TokenType,
    pub string: S,
}

pub struct TokenIterator<S: Parseable> {
    remaining: S,
    line_number: usize,
    column_number: usize,
    byte_offset: usize,
}

impl<S: Parseable> Iterator for TokenIterator<S> {
    type Item = TokResult<S, TokenInfo<S>>;

    fn next(&mut self) -> Option<TokResult<S, TokenInfo<S>>> {
        match next_token(&self.remaining) {
            None => None,
            Some(Err(e)) => {
                Some(Err(e))
            }
            Some(Ok((typ, s))) => {
                let r = Some(Ok(TokenInfo {
                    line_number: self.line_number,
                    column_number: self.column_number,
                    byte_offset: self.byte_offset,
                    typ: typ,
                    // TODO(tyoverby): get rid of this clone
                    string: s.clone(),
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
                println!("consumed: {}", bytes_consumed);
                self.byte_offset += bytes_consumed;

                println!("remaining-a : {:?}", self.remaining);
                self.remaining = self.remaining.drop_front(bytes_consumed);
                println!("remaining-b : {:?}", self.remaining);

                r
            }
        }
    }
}

fn next_token<S: Parseable>(string: &S) -> Option<TokResult<S, (TokenType, S)>> {
    fn idx_until<F>(s: &str, f: F) -> Option<usize> where F: Fn(char) -> bool {
        s.char_indices().take_while(|&(_, c)| f(c)).last().map(|(p, _)| p + 1)
    }

    let first = match string.as_ref().chars().next() {
        Some(c) => c,
        None => return None,
    };

    let next = match first {
        c if c.is_whitespace() => {
            let last_idx = idx_until(string.as_ref(), char::is_whitespace).unwrap();
            Some(Ok((TokenType::Whitespace, string.substring(0, last_idx))))
        }

        '(' => Some(Ok((TokenType::ListOpening(0), string.substring(0, 1)))),
        '{' => Some(Ok((TokenType::ListOpening(1), string.substring(0, 1)))),
        '[' => Some(Ok((TokenType::ListOpening(2), string.substring(0, 1)))),
        ')' => Some(Ok((TokenType::ListClosing(0), string.substring(0, 1)))),
        '}' => Some(Ok((TokenType::ListClosing(1), string.substring(0, 1)))),
        ']' => Some(Ok((TokenType::ListClosing(2), string.substring(0, 1)))),
        _ => {
            let last_idx = idx_until(string.as_ref(), |c| {
                match c {
                    '(' | '{' | '[' | ')' | '}' | ']' => false,
                    _ if c.is_whitespace() => false,
                    _ => true,
                }
            }).unwrap();
            Some(Ok((TokenType::Atom, string.substring(0, last_idx))))
        }
    };
    println!("{:?}", next);
    return next
}


pub fn tokenize<S: Parseable>(string: S) -> TokenIterator<S> {
    TokenIterator {
        remaining: string,
        line_number: 1,
        column_number: 1,
        byte_offset: 0,
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn all_ok(string: &str) -> Vec<TokenInfo<&str>> {
        tokenize(string.into()).collect::<Result<_, _>>().unwrap()
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
                            typ: TokenType::ListOpening(0),
                            string: "(".into(),
                        }]);
    }

    #[test]
    fn single_closing_paren() {
        assert_eq!(all_ok(")"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListClosing(0),
                            string: ")".into(),
                        }]);
    }

    #[test]
    fn paired_parens() {
        assert_eq!(all_ok("()"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListOpening(0),
                            string: "(".into(),
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 2,
                            byte_offset: 1,
                            typ: TokenType::ListClosing(0),
                            string: ")".into(),
                        }])
    }

    #[test]
    fn nested_parens() {

        assert_eq!(all_ok("(())"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListOpening(0),
                            string: "(".into(),
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 2,
                            byte_offset: 1,
                            typ: TokenType::ListOpening(0),
                            string: "(".into(),
                        },

                        TokenInfo {
                            line_number: 1,
                            column_number: 3,
                            byte_offset: 2,
                            typ: TokenType::ListClosing(0),
                            string: ")".into(),
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 4,
                            byte_offset: 3,
                            typ: TokenType::ListClosing(0),
                            string: ")".into(),
                        }])
    }

    #[test]
    fn double_parens() {
        assert_eq!(all_ok("(("),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::ListOpening(0),
                            string: "(".into(),
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 2,
                            byte_offset: 1,
                            typ: TokenType::ListOpening(0),
                            string: "(".into(),
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
                            string: "@".into(),
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
                            string: "123".into(),
                        }]);

        assert_eq!(all_ok("-123"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            string: "-123".into(),
                        }]);

        assert_eq!(all_ok("123.456"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            string: "123.456".into(),
                        }]);

        assert_eq!(all_ok("+123.456"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            string: "+123.456".into(),
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
                            string: "hello-world".into(),
                        }]);

        assert_eq!(all_ok("a"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            string: "a".into(),
                        }]);

        assert_eq!(all_ok("-"),
                   vec![TokenInfo {
                            line_number: 1,
                            column_number: 1,
                            byte_offset: 0,
                            typ: TokenType::Atom,
                            string: "-".into(),
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
                            string: "hello".into(),
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 6,
                            byte_offset: 5,
                            typ: TokenType::Whitespace,
                            string: " ".into(),
                        },
                        TokenInfo {
                            line_number: 1,
                            column_number: 7,
                            byte_offset: 6,
                            typ: TokenType::Atom,
                            string: "world".into(),
                        }]);
    }
}
