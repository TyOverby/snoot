extern crate tendril;
extern crate regex;
extern crate itertools;

pub mod token;
pub mod parse;
pub mod error;

pub trait Parseable: Clone + AsRef<str> + ::std::fmt::Debug {
    fn substring(&self, start: usize, end: usize) -> Self;

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn drop_front(&self, count: usize) -> Self {
        self.substring(count, self.len())
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }

    fn chars(&self) -> ::std::str::Chars {
        self.as_ref().chars()
    }
}

impl <'a> Parseable for &'a str {
    fn substring(&self, start: usize, end: usize) -> Self {
        &self[start .. end]
    }
}

impl <'a> Parseable for String {
    fn substring(&self, start: usize, end: usize) -> Self {
        (&self[start .. end]).into()
    }
}
