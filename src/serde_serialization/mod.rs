#![allow(unused_variables, dead_code)]

#[cfg(test)]
mod test;
mod diagnostics;

use serde;
use serde::de::Visitor;
use serde::de::IntoDeserializer;
use super::Sexpr;
use super::parse::Span;
use super::diagnostic::{DiagnosticBag, Diagnostic};

pub enum DeserializeResult<T> {
    AllGood(T),
    CouldRecover(T, DiagnosticBag),
    CouldntRecover(DiagnosticBag),
}

#[derive(Debug)]
enum DeserError {
    Custom { message: String, },
    DiagnosticAdded,
}

struct SexprDeserializer<'sexpr, 'bag> {
    sexpr: &'sexpr Sexpr,
    bag: &'bag mut DiagnosticBag,
}

struct SeqDeserializer<'sexpr, 'bag> {
    sexprs: &'sexpr[Sexpr],
    bag: &'bag mut DiagnosticBag,
}

struct EnumDeserializer<'sexpr, 'bag> {
    sexprs: &'sexpr[Sexpr],
    bag: &'bag mut DiagnosticBag,
    index: u32,
}

struct VariantDeserializer<'sexpr, 'bag> {
    sexprs: &'sexpr[Sexpr],
    bag: &'bag mut DiagnosticBag,
}

impl <T> DeserializeResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            DeserializeResult::AllGood(t) => t,
            DeserializeResult::CouldRecover(t, b) => {
                b.assert_empty();
                t
            }
            DeserializeResult::CouldntRecover(b) => {
                b.assert_empty();
                unreachable!();
            }
        }
    }
}

impl <'a, T> ::std::iter::FromIterator<DeserializeResult<T>> for DeserializeResult<Vec<T>> {
    fn from_iter<I: IntoIterator<Item=DeserializeResult<T>>>(iter: I) -> DeserializeResult<Vec<T>> {
        let mut out_items = vec![];
        let mut out_bag = DiagnosticBag::new();
        for res in iter {
            match res {
                DeserializeResult::AllGood(t) => {
                    out_items.push(t);
                }
                DeserializeResult::CouldRecover(t, b) => {
                    out_items.push(t);
                    out_bag.append(b);
                }
                DeserializeResult::CouldntRecover(b) => {
                    out_bag.append(b);
                }
            }
        }

        match (out_items.len(), out_bag.len()) {
            (_, 0) => DeserializeResult::AllGood(out_items),
            (0, _) => DeserializeResult::CouldntRecover(out_bag),
            (_, _) => DeserializeResult::CouldRecover(out_items, out_bag),
        }
    }
}
pub fn deserialize<'sexpr, T: serde::Deserialize<'sexpr>>(sexpr: &'sexpr Sexpr) -> DeserializeResult<T> {
    let mut bag = DiagnosticBag::new();
    let res = {
        let deserializer = SexprDeserializer {
            sexpr: sexpr,
            bag: &mut bag,
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

impl <'sexpr, 'bag> SeqDeserializer<'sexpr, 'bag> {
    fn all_spans(&self) -> Span {
        self.sexprs.iter().map(|x|x.span()).collect()
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

fn add<T>(bag: &mut DiagnosticBag, diagnostic: Diagnostic) -> Result<T, DeserError> {
    bag.add(diagnostic);
    Err(DeserError::DiagnosticAdded)
}

macro_rules! deserialize_value {
    ($this: expr, $visitor: expr, $func: ident, $typ: ty, $parser: path, $descr: expr) => {{
        let error = |span: &Span| diagnostic!(span, "expected to parse {} but found {}", $descr, span.text());
        if let &Sexpr::Terminal(_, ref span) = $this.sexpr {
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
            $this.bag.add(error($this.sexpr.span()));
            wrap_visitor_result($visitor.$func(Default::default()), $this.sexpr.span(), &mut $this.bag)
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
        if let &Sexpr::Terminal(_, ref span)  = self.sexpr {
            if span.text().as_ref() == "nil" {
                wrap_visitor_result(visitor.visit_none(), self.sexpr.span(), self.bag)
            } else {
                let r = visitor.visit_some(SexprDeserializer{sexpr: self.sexpr, bag: self.bag});
                wrap_visitor_result(r, &self.sexpr.span(), self.bag)
            }
        } else {
            let r = visitor.visit_some(SexprDeserializer{sexpr: self.sexpr, bag: self.bag});
            wrap_visitor_result(r, &self.sexpr.span(), self.bag)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        if let &Sexpr::List{ref children, ref span, ..} = self.sexpr {
            if children.len() == 0 {
                wrap_visitor_result(visitor.visit_unit(), self.sexpr.span(), self.bag)
            } else {
                self.bag.add(diagnostic!(self.sexpr.span(), "expected unit tuple, found {} items", children.len()));
                Err(DeserError::DiagnosticAdded)
            }
        } else {
            self.bag.add(diagnostic!(self.sexpr.span(), "expected unit tuple"));
            Err(DeserError::DiagnosticAdded)
        }
    }

    fn deserialize_unit_struct<V>(self,
                                  name: &'static str,
                                  visitor: V)
                                  -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        self.deserialize_tuple_struct(name, 0, visitor)
    }
    fn deserialize_newtype_struct<V>(self,
                                     name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        self.deserialize_tuple_struct(name, 1, visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        if let &Sexpr::List{ref children, ref span, ..} = self.sexpr {
            wrap_visitor_result(visitor.visit_seq(SeqDeserializer{sexprs: children, bag: self.bag}), &self.sexpr.span(), self.bag)
        } else {
            self.bag.add(diagnostic!(self.sexpr.span(), "expected list, found {:?}", self.sexpr.kind()));
            return Err(DeserError::DiagnosticAdded);
        }
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        if let &Sexpr::List{ref children, ref span, ..} = self.sexpr {
            wrap_visitor_result(visitor.visit_seq(SeqDeserializer{sexprs: children, bag: self.bag}), &self.sexpr.span(), self.bag)
        } else {
            self.bag.add(diagnostic!(self.sexpr.span(), "expected list, found {:?}", self.sexpr.kind()));
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
        if let &Sexpr::List{ref children, ref span, ..} = self.sexpr {
            if children.len() == 0 {
                self.bag.add(diagnostics::nothing_found(span, struct_descr()));
                Err(DeserError::DiagnosticAdded)
            } else {
                if let &Sexpr::Terminal(_, ref span) = &children[0] {
                    if span.text().as_ref() != name {
                        self.bag.add(diagnostic!(span, "expected tuple struct name `{}`, but found `{}`", name, span.text()));
                        Err(DeserError::DiagnosticAdded)
                    } else {
                        let vr = {
                            let seqd = SeqDeserializer{ sexprs: &children[1..], bag: self.bag};
                            visitor.visit_seq(seqd)
                        };
                        wrap_visitor_result(vr, span, self.bag)
                    }
                } else {
                    self.bag.add(diagnostic!(span, "expected tuple struct name `{}`, but found `{}`", name, span.text()));
                    Err(DeserError::DiagnosticAdded)
                }
            }
        } else {
            self.bag.add(diagnostic!(&self.sexpr.span(), "expected list, found {:?}", self.sexpr.kind()));
            return Err(DeserError::DiagnosticAdded);
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        if let &Sexpr::List{ref children, ref span, ..} = self.sexpr {
            let vr = {
                let map_deser = SeqDeserializer{sexprs: children, bag: self.bag};
                visitor.visit_map(map_deser)
            };
            wrap_visitor_result(vr, &self.sexpr.span(), self.bag)
        } else {
            self.bag.add(diagnostic!(self.sexpr.span(), "expected map, found `{:?}`", self.sexpr.kind()));
            Err(DeserError::DiagnosticAdded)
        }
    }
    fn deserialize_struct<V>(mut self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let struct_descr = || format!("struct {}", name);
        if let &Sexpr::List{ref children, ref span, ..} = self.sexpr {
            if children.len() == 0 {
                self.bag.add(diagnostics::nothing_found(span, struct_descr()));
                Err(DeserError::DiagnosticAdded)
            } else {
                let first_child = &children[0];
                let rest_span: Span = children[1..].iter().map(Sexpr::span).collect();
                if let &Sexpr::Terminal(_, ref span) = first_child {
                    if span.text().as_ref() == name {
                        wrap_visitor_result(visitor.visit_map(
                            SeqDeserializer{sexprs: &children[1..], bag: self.bag}), &rest_span, self.bag)
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
            self.bag.add(diagnostic!(&self.sexpr.span(), "expected {}, found {:?}", struct_descr(), self.sexpr.kind()));
            Err(DeserError::DiagnosticAdded)
        }
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
        let desc = || format!("enum {}", name);
        if let &Sexpr::List{ref children, ref span, ..} = self.sexpr {
            if children.len() == 0 {
                add(self.bag, diagnostic!(span, "expected {}, found empty list", desc()))
            } else {
                let first = &children[0];
                if let &Sexpr::Terminal(_, ref span) = first {
                    if let Some(idx) = variants.iter().position(|&c| c == span.text().as_ref()) {
                        let res = visitor.visit_enum(EnumDeserializer{sexprs: &children[1..], bag: self.bag, index: idx as u32});
                        wrap_visitor_result(res, span, self.bag)
                    } else {
                        add(self.bag, diagnostic!(span, "{} is not a variant name for {}", span.text(), desc()))
                    }
                } else {
                    add(self.bag, diagnostic!(span, "expected variant name for {}, found empty list", desc()))
                }
            }
        } else {
            add(self.bag, diagnostic!(self.sexpr.span(), "expected {}, found {:?}", desc(), self.sexpr.kind()))
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        self.bag.add(diagnostic!(&self.sexpr.span(), "ignored value"));
        Err(DeserError::DiagnosticAdded)
    }
}

impl <'sexpr, 'bag, 'de> serde::de::SeqAccess<'de> for SeqDeserializer <'sexpr, 'bag> {

    type Error = DeserError;

    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> {
        if self.sexprs.len() == 0 {
            return Ok(None);
        }

        let first = &self.sexprs[0];
        let res = seed.deserialize(SexprDeserializer {sexpr: first, bag: self.bag}).map(Some);
        self.sexprs = &self.sexprs[1..];
        res
    }
}


impl <'sexpr, 'bag, 'de> serde::de::MapAccess<'de> for SeqDeserializer<'sexpr, 'bag> {
    type Error = DeserError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where K: serde::de::DeserializeSeed<'de> {
        if self.sexprs.len() == 0 {
            return Ok(None);
        }

        if self.sexprs.len() == 1 {
            let all_spans = self.all_spans();
            self.bag.add(diagnostic!(&all_spans, "expected key followed by `:`"));
            return Err(DeserError::DiagnosticAdded);
        }

        let first = &self.sexprs[0];
        let colon = &self.sexprs[1];

        if let &Sexpr::Terminal(_, ref span) = colon {
            if span.text().as_ref() != ":" {
                self.bag.add(diagnostic!(span, "expected `:`, found `{}`", span.text()));
            }
        } else {
            self.bag.add(diagnostic!(colon.span(), "expected terminal `:`, found `{:?}`", colon.kind()));
        }

        let res = seed.deserialize(SexprDeserializer{sexpr: first, bag: self.bag}).map(Some);

        self.sexprs = &self.sexprs[2..];

        res
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where V: serde::de::DeserializeSeed<'de> {
        if self.sexprs.len() == 0 {
            let all_spans = self.all_spans();
            self.bag.add(diagnostic!(&all_spans, "expected value"));
            return Err(DeserError::DiagnosticAdded);
        }

        let first = &self.sexprs[0];
        let res = seed.deserialize(SexprDeserializer{sexpr: first, bag: self.bag});
        self.sexprs = &self.sexprs[1..];
        res
    }
}

impl <'sexpr, 'bag, 'de> serde::de::EnumAccess<'de> for EnumDeserializer<'sexpr, 'bag> {
    type Error = DeserError;
    type Variant = VariantDeserializer<'sexpr, 'bag>;
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), DeserError>
                where V: serde::de::DeserializeSeed<'de>,
    {
        let idx = seed.deserialize(self.index.into_deserializer())?;
        Ok((idx, VariantDeserializer{sexprs: self.sexprs, bag: self.bag }))
    }
}
impl<'sexpr, 'bag, 'de> serde::de::VariantAccess<'de> for VariantDeserializer<'sexpr, 'bag>{
    type Error = DeserError;

    fn unit_variant(self) -> Result<(), DeserError> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, DeserError>
        where T: serde::de::DeserializeSeed<'de>,
    {
        // TODO: check count of sexprs
        seed.deserialize(SexprDeserializer{sexpr: &self.sexprs[0], bag: self.bag})
    }

    fn tuple_variant<V>(self,
                      len: usize,
                      visitor: V) -> Result<V::Value, DeserError>
        where V: serde::de::Visitor<'de>,
    {
        let map_deser = SeqDeserializer{sexprs: self.sexprs, bag: self.bag};
        visitor.visit_seq(map_deser)
    }

    fn struct_variant<V>(self,
                       fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value, DeserError>
        where V: serde::de::Visitor<'de>,
    {
        let map_deser = SeqDeserializer{sexprs: self.sexprs, bag: self.bag};
        visitor.visit_map(map_deser)
    }
}
