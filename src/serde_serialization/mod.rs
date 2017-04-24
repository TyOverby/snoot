#![allow(unused_variables, dead_code)]

#[cfg(test)]
mod test;
mod diagnostics;

use serde;
use serde::de::Visitor;
use super::Sexpr;
use super::parse::Span;
use super::diagnostic::DiagnosticBag;

pub enum DeserializeResult<T> {
    AllGood(T),
    CouldRecover(T, DiagnosticBag),
    CouldntRecover(DiagnosticBag),
}

pub fn deserialize<'sexpr, T: serde::Deserialize<'sexpr>>(sexprs: &'sexpr[Sexpr]) -> DeserializeResult<T> {
    let mut bag = DiagnosticBag::new();
    let span: Span = sexprs.iter().map(|s| s.span()).collect();
    let res = {
        let deserializer = SexprDeserializer {
            sexprs: sexprs,
            bag: &mut bag,
            persist: true,
            span: span,
        };

        T::deserialize(deserializer)
    };

    match res {
        Ok(t) => {
            if bag.is_empty() {
                DeserializeResult::AllGood(t)
            } else {
                DeserializeResult::CouldRecover(t, bag)
            }
        }
        Err(e) => {
            DeserializeResult::CouldntRecover(bag)
        }
    }
}

#[derive(Debug)]
enum DeserError {
    Custom { message: String, },
    DiagnosticAdded,
}

struct SexprDeserializer<'sexpr, 'bag> {
    sexprs: &'sexpr[Sexpr],
    bag: &'bag mut DiagnosticBag,
    span: Span,
    persist: bool,
}

impl <'sexpr, 'bag> SexprDeserializer<'sexpr, 'bag> {
    fn with_same<'a>(&'a mut self) -> SexprDeserializer<'a, 'a> {
        SexprDeserializer {
            sexprs: self.sexprs,
            bag: self.bag,
            span: self.span.clone(),
            persist: self.persist,
        }
    }
    fn with_sexprs<'a>(&'a mut self, sexprs: &'sexpr[Sexpr], newspan: Span) -> SexprDeserializer<'a, 'a> {
        SexprDeserializer {
            sexprs: sexprs,
            bag: self.bag,
            span: newspan,
            persist: self.persist,
        }
    }
}

impl serde::de::Error for DeserError {
    fn custom<T: ::std::fmt::Display>(msg: T) -> Self {
        DeserError::Custom { message: format!("{}", msg) }
    }
}

impl ::std::error::Error for DeserError {
    fn description(&self) -> &str {
        match self {
            &DeserError::Custom{ref message} => message,
            &DeserError::DiagnosticAdded => "diagnostic added",
        }
    }
}

impl ::std::fmt::Display for DeserError {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            &DeserError::Custom{ref message} => write!(formatter, "{}", message),
            &DeserError::DiagnosticAdded => write!(formatter, "Diagnostic Added"),
        }
    }
}

fn wrap_visitor_result<T>(result: Result<T, DeserError>, span: &Span, bag: &mut DiagnosticBag) -> Result<T, DeserError> {
    match result {
        Ok(t) => Ok(t),
        Err(DeserError::DiagnosticAdded) => Err(DeserError::DiagnosticAdded),
        Err(DeserError::Custom{message}) => {
            bag.add(diagnostic!(span, "{}", message));
            Err(DeserError::DiagnosticAdded)
        }
    }
}

macro_rules! deserialize_value {
    ($this: expr, $visitor: expr, $func: ident, $typ: ty, $parser: path, $descr: expr) => {{
        let error = |span: &Span| diagnostic!(span, "expected to parse {} but found {}", $descr, span.text());
        let sexpr = match $this.sexprs.len() {
            0 => {
                $this.bag.add(diagnostics::nothing_found(&$this.span, $descr));
                return wrap_visitor_result($visitor.$func(Default::default()), &$this.span, &mut $this.bag);
            }
            1 => &$this.sexprs[0],
            n => {
                $this.bag.add(diagnostics::multiple_values_found(&$this.span, $descr));
                &$this.sexprs[0]
            }
        };

        if let &Sexpr::Terminal(_, ref span) = sexpr {
            let text = span.text();
            let text2 = text.as_ref();
            let x: Result<$typ, _> = $parser(text2);
            match x {
                Ok(x) => wrap_visitor_result($visitor.$func(x), span, &mut $this.bag),
                Err(e) => {
                    $this.bag.add(diagnostic!(span, "could not parse `{}` as a {}", span.text(), $descr));
                    wrap_visitor_result($visitor.$func(Default::default()), span, &mut $this.bag)
                }
            }
        } else {
            $this.bag.add(error(sexpr.span()));
            wrap_visitor_result($visitor.$func(Default::default()), sexpr.span(), &mut $this.bag)
        }
    }
}}

impl <'sexpr, 'bag, 'de> serde::Deserializer<'de> for SexprDeserializer<'sexpr, 'bag> {
    type Error = DeserError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        // Can this even be implemented?
        unimplemented!();
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_bool, bool, str::parse, "boolean value")
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_u8, u8, str::parse, "unsigned integer (u8)")
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_u16, u16, str::parse, "unsigned integer (u16)")
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_u32, u32, str::parse, "unsigned integer (u32)")
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_u64, u64, str::parse, "unsigned integer (u64)")
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_i8, i8, str::parse, "signed integer (i8)")
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_i16, i16, str::parse, "signed integer (i16)")
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_i32, i32, str::parse, "signed integer (i32)")
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_i64, i64, str::parse, "signed integer (i64)")
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_f32, f32, str::parse, "floating point number (f32)")
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        deserialize_value!(self, visitor, visit_f64, f64, str::parse, "floating point number (f64)")
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> { unimplemented!(); }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {unimplemented!();}

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {unimplemented!()}

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {unimplemented!()}

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        unimplemented!();
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        unimplemented!();
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {unimplemented!()}

    fn deserialize_unit_struct<V>(self,
                                  name: &'static str,
                                  visitor: V)
                                  -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        unimplemented!();
    }
    fn deserialize_newtype_struct<V>(self,
                                     name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        wrap_visitor_result(visitor.visit_seq(self.with_same()), &self.span, &mut self.bag)
    }
/*
    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        unimplemented!();
    }
*/
    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let child = if self.sexprs.len() == 0 {
            self.bag.add(diagnostics::nothing_found(&self.span, "tuple"));
            return Err(DeserError::DiagnosticAdded);
        } else if self.sexprs.len() > 1 {
            self.bag.add(diagnostics::multiple_values_found(&self.span, "tuple"));
            &self.sexprs[0]
        } else {
            &self.sexprs[0]
        };

        if let &Sexpr::List{ref children, ref span, ..} = child {
            wrap_visitor_result(visitor.visit_seq(self.with_sexprs(children, span.clone())), &self.span, &mut self.bag)
        } else {
            self.bag.add(diagnostic!(&child.span(), "expected list, found {:?}", child.kind()));
            return Err(DeserError::DiagnosticAdded);
        }
    }
    fn deserialize_tuple_struct<V>(mut self,
                                   name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let struct_descr = || format!("tuple struct {}", name);
        let child = if self.sexprs.len() == 0 {
            self.bag.add(diagnostics::nothing_found(&self.span, struct_descr()));
            return Err(DeserError::DiagnosticAdded);
        } else if self.sexprs.len() > 1 {
            self.bag.add(diagnostics::multiple_values_found(&self.span, struct_descr()));
            &self.sexprs[0]
        } else {
            &self.sexprs[0]
        };

        if let &Sexpr::List{ref children, ref span, ..} = child {
            if children.len() == 0 {
                self.bag.add(diagnostics::nothing_found(span, struct_descr()));
                Err(DeserError::DiagnosticAdded)
            } else {
                if let &Sexpr::Terminal(_, ref span) = &children[0] {
                    if span.text().as_ref() != name {
                        self.bag.add(diagnostic!(span, "expected tuple struct name `{}`, but found `{}`", name, span.text()));
                        Err(DeserError::DiagnosticAdded)
                    } else {
                        wrap_visitor_result(visitor.visit_seq(self.with_sexprs(&children[1..], span.clone())), &self.span, &mut self.bag)
                    }
                } else {
                    self.bag.add(diagnostic!(span, "expected tuple struct name `{}`, but found `{}`", name, span.text()));
                    Err(DeserError::DiagnosticAdded)
                }
            }
        } else {
            self.bag.add(diagnostic!(&child.span(), "expected list, found {:?}", child.kind()));
            return Err(DeserError::DiagnosticAdded);
        }
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        wrap_visitor_result(visitor.visit_map(self.with_same()), &self.span, &mut self.bag)
    }
    fn deserialize_struct<V>(mut self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let struct_descr = || format!("struct {}", name);
        let struct_sexpr = if self.sexprs.len() == 0 {
            self.bag.add(diagnostics::nothing_found(&self.span, struct_descr()));
            return Err(DeserError::DiagnosticAdded)
        } else if self.sexprs.len() > 1 {
            self.bag.add(diagnostics::multiple_values_found(&self.span, struct_descr()));
            &self.sexprs[0]
        } else {
            &self.sexprs[0]
        };

        if let &Sexpr::List{ref children, ref span, ..} = struct_sexpr {
            if children.len() == 0 {
                self.bag.add(diagnostics::nothing_found(span, struct_descr()));
                Err(DeserError::DiagnosticAdded)
            } else {
                let first_child = &children[0];
                let rest_span: Span = children[1..].iter().map(Sexpr::span).collect();
                if let &Sexpr::Terminal(_, ref span) = first_child {
                    if span.text().as_ref() == name {
                        wrap_visitor_result(visitor.visit_map(
                            self.with_sexprs( &children[1..], rest_span.clone())
                        ), &rest_span, &mut self.bag)
                    } else {
                        self.bag.add(diagnostic!(
                            first_child.span(),
                            "Expected structure name identifier `{}`, found `{}`",
                            name, first_child.span().text()));
                        Err(DeserError::DiagnosticAdded)
                    }
                } else {
                    self.bag.add(diagnostic!(
                        first_child.span(),
                        "Expected structure name identifier `{}`, found `{}`",
                        name, first_child.span().text()));
                    Err(DeserError::DiagnosticAdded)
                }
            }
        } else {
            self.bag.add(diagnostic!(&self.span, "expected {}, found {:?}", struct_descr(), struct_sexpr.kind()));
            Err(DeserError::DiagnosticAdded)
        }

        //wrap_visitor_result(visitor.visit_map(self.with_same()), self.span, &mut self.bag)
        //unimplemented!()
    }
    fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        fn id<T>(x: T) -> Result<T, ()>{ Ok(x) }
        deserialize_value!(self, visitor, visit_str, &str, id, "identifier")
    }

    fn deserialize_enum<V>(self,
                           name: &'static str,
                           variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        self.bag.add(diagnostic!(&self.span, "ignored value"));
        Err(DeserError::DiagnosticAdded)
    }
}

impl <'sexpr, 'bag, 'de> serde::de::SeqAccess<'de> for SexprDeserializer<'sexpr, 'bag> {
    type Error = DeserError;

    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> {
        if self.sexprs.len() == 0 {
            return Ok(None);
        }

        let first = &self.sexprs[..1];
        let rest_span: Span = self.sexprs[1..].iter().map(Sexpr::span).collect();
        let res = seed.deserialize(self.with_sexprs(first, rest_span)).map(Some);
        self.sexprs = &self.sexprs[1..];
        res
    }
}


impl <'sexpr, 'bag, 'de> serde::de::MapAccess<'de> for SexprDeserializer<'sexpr, 'bag> {
    type Error = DeserError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where K: serde::de::DeserializeSeed<'de> {
        if self.sexprs.len() == 0 {
            return Ok(None);
        }

        if self.sexprs.len() == 1 {
            self.bag.add(diagnostic!(&self.span, "expected key followed by `:`"));
            return Err(DeserError::DiagnosticAdded);
        }

        let first = &self.sexprs[0..1];
        let colon = &self.sexprs[1];

        if let &Sexpr::Terminal(_, ref span) = colon {
            if span.text().as_ref() != ":" {
                self.bag.add(diagnostic!(span, "expected `:`, found `{}`", span.text()));
            }
        } else {
            self.bag.add(diagnostic!(colon.span(), "expected terminal `:`, found `{:?}`", colon.kind()));
        }

        let rest_span = self.sexprs[2..].iter().map(Sexpr::span).collect();
        let res = seed.deserialize(self.with_sexprs(first, rest_span)).map(Some);

        self.sexprs = &self.sexprs[2..];

        res
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where V: serde::de::DeserializeSeed<'de> {
        if self.sexprs.len() == 0 {
            self.bag.add(diagnostic!(&self.span, "expected value"));
            return Err(DeserError::DiagnosticAdded);
        }

        let first = &self.sexprs[..1];
        let rest_span = self.sexprs[1..].iter().map(Sexpr::span).collect();
        let res = seed.deserialize(self.with_sexprs(first, rest_span));
        self.sexprs = &self.sexprs[1..];
        res
    }
}
