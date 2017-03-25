#![feature(pub_restricted)]
extern crate tendril;
extern crate regex;
extern crate itertools;

pub mod token;
pub mod parse;
pub mod diagnostic;

pub struct Result {
    pub roots: Vec<parse::Sexpr>,
    pub diagnostics: diagnostic::DiagnosticBag,
}

pub fn simple_parse<'a, S: Into<tendril::StrTendril>>(string: S, splitters: &'a[&'a str]) -> Result {
    let tendril = string.into();
    let tokens = token::tokenize(tendril.clone(), splitters);
    parse::parse(&tendril, tokens)
}
