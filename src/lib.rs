#![feature(pub_restricted)]
extern crate tendril;
extern crate regex;
extern crate itertools;

pub mod token;
pub mod parse;
pub mod diagnostic;
mod sexpr;

pub use sexpr::Sexpr;

/// The result of a text parse.
///
/// Since Snoot has good error recovery, parses can produce
/// *both* valid trees, *and* error messages.
pub struct Result {
    /// A list of Sexpr tree roots
    pub roots: Vec<Sexpr>,
    /// A bag of diagnostics collected during parse.
    ///
    /// All parse errors in the bag are ErrorLevel::Error.
    pub diagnostics: diagnostic::DiagnosticBag,
}

/// Parses some text with the builtin tokenizer.
///
/// `splitters` is a list of strings that should be split on the tokenization level.
/// As an example: [":"] will make "foo:bar" split into ["foo", ":", "bar"] during tokenization.
pub fn simple_parse<'a, S: Into<tendril::StrTendril>>(string: S, splitters: &'a[&'a str]) -> Result {
    let tendril = string.into();
    let tokens = token::tokenize(tendril.clone(), splitters);
    parse::parse(&tendril, tokens)
}
