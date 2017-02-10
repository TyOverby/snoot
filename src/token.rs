use tendril::StrTendril;
use regex::{Regex, Matches};
use regex::Error as RegexError;

pub struct TokenizationOptions<'a> {
    pub whitespace: Vec<&'a str>,
    pub list_chars: Vec<(&'a str, &'a str)>,
    pub string_chars: Vec<&'a str>,
    pub string_escape_char: &'a str,
    pub unary_operators: Vec<&'a str>,
    pub numbers: Vec<&'a str>,
    pub identifiers: Vec<&'a str>,
}

pub struct CompiledTokenizationOptions {
    whitespace: Regex,
    list_opening: Regex,
    list_closing: Regex,
    string_chars: Regex,
    _string_escape_char: Regex,
    unary_operators: Regex,
    numbers: Regex,
    identifiers: Regex,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenType {
    ListOpening,
    ListClosing,
    Whitespace,
    UnaryOperator,
    String,
    Number,
    Identifier,
}

pub type TokResult<T> = Result<T, TokError>;

#[derive(Debug, Eq, PartialEq)]
pub enum TokError {
    UnclosedString(TokenInfo),
    NoMatch(StrTendril),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenInfo {
    pub line_number: u32,
    pub column_number: u32,
    pub byte_offset: u32,
    pub typ: TokenType,
    pub string: StrTendril,
}

pub struct TokenIterator<'a> {
    remaining: StrTendril,
    line_number: u32,
    column_number: u32,
    byte_offset: u32,
    cto: &'a CompiledTokenizationOptions,
    errored: bool,
}

impl <'a> Iterator for TokenIterator<'a> {
    type Item = TokResult<TokenInfo>;

    fn next(&mut self) -> Option<TokResult<TokenInfo>> {
        if self.errored { return None; }
        match next_token(&self.remaining, self.cto) {
            None => None,
            Some(Err(e)) => {
                self.errored = true;
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

                for chr in s[..].chars() {
                    if chr == '\n' {
                        self.line_number += 1;
                        self.column_number = 1;
                    } else {
                        self.column_number += 1;
                    }
                }

                let bytes_consumed = s.len() as u32;
                self.byte_offset += bytes_consumed;

                self.remaining.pop_front(bytes_consumed);

                r
            }
        }
    }
}

fn next_token(string: &StrTendril, cto: &CompiledTokenizationOptions) -> Option<TokResult<(TokenType, StrTendril)>> {
    fn take(m: Matches, string: &StrTendril) -> Option<StrTendril> {
        let mut longest_length = 0;

        for m in m {
            if m.start() == 0 {
                longest_length = ::std::cmp::max(longest_length, m.end());
            }
        }

        if longest_length != 0 {
            let l = string.subtendril(0, longest_length as u32);
            Some(l)
        } else {
            None
        }
    }

    if string.is_empty() {
        return None;
    }

    if let Some(s) = take(cto.whitespace.find_iter(&string), &string) {
        return Some(Ok((TokenType::Whitespace, s)));
    }

    if let Some(s) = take(cto.list_opening.find_iter(&string), &string) {
        return Some(Ok((TokenType::ListOpening, s)));
    }

    if let Some(s) = take(cto.list_closing.find_iter(&string), &string) {
        return Some(Ok((TokenType::ListClosing, s)));
    }

    if let Some(_mtch) = cto.string_chars.find(&string) {
        unimplemented!();
    }

    if let Some(s) = take(cto.unary_operators.find_iter(&string), &string) {
        return Some(Ok((TokenType::UnaryOperator, s)));
    }

    if let Some(s) = take(cto.numbers.find_iter(&string), &string) {
        return Some(Ok((TokenType::Number, s)));
    }

    if let Some(s) = take(cto.identifiers.find_iter(&string), &string) {
        return Some(Ok((TokenType::Identifier, s)));
    }

    return Some(Err(TokError::NoMatch(string.clone())));
}

impl <'a> TokenizationOptions<'a> {
    pub fn default() -> TokenizationOptions<'static> {
        TokenizationOptions {
            whitespace: vec![" ", "\t", "\n"],
            list_chars: vec![("\\(", "\\)"), ("\\{", "\\}"), ("\\[", "\\]")],
            string_chars: vec!["\""],
            string_escape_char: "\\\\",
            unary_operators: vec![",", "@", "'"],
            numbers: vec!["[+-]?[0-9]+\\.[0-9]+", "[+-]?[0-9]+"],
            identifiers: vec!["[a-zA-Z_-]+[a-zA-Z0-9_-]*"],
        }
    }

    pub fn compile(self) -> Result<CompiledTokenizationOptions, RegexError> {
        fn group<'a, I: Iterator<Item=&'a str>>(strings: I, conjoin: bool) -> Result<Regex, RegexError>{
            let mut all: String = "\\A(".into();
            for string in strings {
                all.push('(');
                all.push_str(string);
                all.push(')');
                all.push('|');
            }
            all.pop();
            all.push(')');

            if conjoin {
                all.push('+');
            }


            Regex::new(&all)
        }

        let TokenizationOptions {
            whitespace,
            list_chars,
            string_chars,
            string_escape_char,
            unary_operators,
            numbers,
            identifiers
        } = self;

        let (list_op, list_close): (Vec<_>, Vec<_>) = list_chars.into_iter().unzip();

        Ok(CompiledTokenizationOptions {
            whitespace: try!(group(whitespace.into_iter(), true)),
            list_opening: try!(group(list_op.into_iter(), false)),
            list_closing: try!(group(list_close.into_iter(), false)),
            string_chars: try!(group(string_chars.into_iter(), false)),
            _string_escape_char: try!(Regex::new(string_escape_char)),
            unary_operators: try!(group(unary_operators.into_iter(), false)),
            numbers: try!(group(numbers.into_iter(), false)),
            identifiers: try!(group(identifiers.into_iter(), false)),
        })
    }
}

pub fn tokenize<'a>(string: StrTendril, cto: &'a CompiledTokenizationOptions) -> TokenIterator<'a> {
    TokenIterator {
        remaining: string,
        line_number: 1,
        column_number: 1,
        byte_offset: 0,
        cto: cto,
        errored: false
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn all_ok(string: &str) -> Vec<TokenInfo> {
        let to = TokenizationOptions::default();
        let cto = to.compile().unwrap();
        tokenize(string.into(), &cto).collect::<Result<_, _>>().unwrap()
    }

    #[test]
    fn empty() {
        assert_eq!(all_ok(""), vec![]);
    }

    #[test]
    fn single_open_paren() {
        assert_eq!(all_ok("("), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::ListOpening,
                string: "(".into(),
            }]);
    }

    #[test]
    fn single_closing_paren() {
        assert_eq!(all_ok(")"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::ListClosing,
                string: ")".into(),
            }]);
    }

    #[test]
    fn paired_parens() {
        assert_eq!(all_ok("()"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::ListOpening,
                string: "(".into(),
            },
            TokenInfo {
                line_number: 1,
                column_number: 2,
                byte_offset: 1,
                typ: TokenType::ListClosing,
                string: ")".into(),
            },
        ])
    }

    #[test]
    fn nested_parens() {

        assert_eq!(all_ok("(())"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::ListOpening,
                string: "(".into(),
            },
            TokenInfo {
                line_number: 1,
                column_number: 2,
                byte_offset: 1,
                typ: TokenType::ListOpening,
                string: "(".into(),
            },

            TokenInfo {
                line_number: 1,
                column_number: 3,
                byte_offset: 2,
                typ: TokenType::ListClosing,
                string: ")".into(),
            },
            TokenInfo {
                line_number: 1,
                column_number: 4,
                byte_offset: 3,
                typ: TokenType::ListClosing,
                string: ")".into(),
            },
        ])
    }

    #[test]
    fn double_parens() {
        assert_eq!(all_ok("(("), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::ListOpening,
                string: "(".into(),
            },
            TokenInfo {
                line_number: 1,
                column_number: 2,
                byte_offset: 1,
                typ: TokenType::ListOpening,
                string: "(".into(),
            },
        ])
    }

    #[test]
    fn unary_literal() {
        assert_eq!(all_ok("@"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::UnaryOperator,
                string: "@".into(),
            }]);
    }

    #[test]
    fn numbers() {
        assert_eq!(all_ok("123"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Number,
                string: "123".into(),
            }]);

        assert_eq!(all_ok("-123"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Number,
                string: "-123".into(),
            }]);

        assert_eq!(all_ok("123.456"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Number,
                string: "123.456".into(),
            }]);

        assert_eq!(all_ok("+123.456"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Number,
                string: "+123.456".into(),
            }]);
    }

    #[test]
    fn identifier() {
        assert_eq!(all_ok("hello-world"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Identifier,
                string: "hello-world".into(),
            }]);

        assert_eq!(all_ok("a"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Identifier,
                string: "a".into(),
            }]);

        assert_eq!(all_ok("-"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Identifier,
                string: "-".into(),
            }]);
    }

    #[test]
    fn ident_white_ident() {
        assert_eq!(all_ok("hello world"), vec![
            TokenInfo {
                line_number: 1,
                column_number: 1,
                byte_offset: 0,
                typ: TokenType::Identifier,
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
                typ: TokenType::Identifier,
                string: "world".into(),
            },
        ]);
    }
}

