extern crate tendril;
extern crate regex;
extern crate itertools;

pub mod token;
pub mod parse;
pub mod error;

pub use parse::ParseResult;

pub fn simple_parse<'a, S: Into<tendril::StrTendril>>(string: S, splitters: &'a[&'a str]) -> parse::ParseResult {
    let tendril = string.into();
    let tokens = token::tokenize(tendril.clone(), splitters);
    parse::parse(&tendril, tokens)
}
