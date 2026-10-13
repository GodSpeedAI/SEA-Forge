//! Private serde adapter that enforces YAML typing rules at the parsing boundary.
//!
//! serde-saphyr picks a scalar's meaning from the Rust target type, so
//! `"false"` becomes `false`, `~` becomes an empty collection and `1_000`
//! becomes an integer. This adapter decides what each target may accept:
//!
//! - bool: only a real boolean (`visit_bool`) whose source text is exactly `true`
//!   or `false`;
//! - integers: a plain scalar that is a canonical decimal, see below;
//! - floats: a number (`visit_f64` or an integer visit); non-finite values are
//!   rejected by the parser's `reject_non_finite_typeless_float` option;
//! - strings and chars: whatever the parser returns as text; null stays an error;
//! - unit, Option: null (or absent) is `None`, `Some(x)` recurses;
//! - sequences and tuples: only `visit_seq`; maps and structs: only `visit_map`;
//! - enums: only `visit_enum`;
//! - `deserialize_any`, bytes, unit structs, newtype structs, identifiers and
//!   ignored values are forwarded unchanged.
//!
//! Every nested deserializer, sequence element, map key and value, enum variant
//! payload and seed is wrapped in `Strict` again, so the rules apply at every
//! depth. `StrictVisitor` gates each visit by the shape it expects and wraps the
//! access it hands to the caller's visitor.
//!
//! Integer spelling needs the source text, which `deserialize_any` discards. The
//! boundary therefore parses the document twice: `Mode::Shape` checks types with
//! `deserialize_any`, and `Mode::Spelling` reads integer targets through
//! `deserialize_str` and requires canonical decimal text (`-?(0|[1-9][0-9]*)`,
//! `-0` excluded). The value comes from the shape pass; both must succeed.
//!
//! Known hole: values that serde buffers into its private `Content` type
//! (`#[serde(flatten)]`, `#[serde(tag = ...)]`) are read with `deserialize_any`,
//! so the spelling pass cannot see their source text. Such an integer written as
//! `1_000` is accepted there. Quoted strings, null collections and quoted
//! booleans are still rejected, because serde's `Content` deserializer enforces
//! those types itself.

use serde::de::{
    self, DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess, Unexpected,
    VariantAccess, Visitor,
};
use std::fmt;

/// Which of the two parsing passes a deserializer belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    /// Type and shape checks, driven by `deserialize_any`.
    Shape,
    /// Canonical integer spelling, read through `deserialize_str`.
    Spelling,
}

/// The shape a caller's visitor is prepared to accept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expect {
    Any,
    Bool,
    Int,
    Float,
    Str,
    Unit,
    Option,
    Seq,
    Map,
    Enum,
}

/// The shape the parser actually produced for a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Bool,
    Int,
    Float,
    Str,
    Unit,
    Some,
    Seq,
    Map,
    Enum,
    Bytes,
    Newtype,
}

impl Expect {
    fn allows(self, kind: Kind) -> bool {
        match self {
            Expect::Any => true,
            Expect::Bool => kind == Kind::Bool,
            Expect::Int => kind == Kind::Int,
            Expect::Float => matches!(kind, Kind::Float | Kind::Int),
            Expect::Str => kind == Kind::Str,
            Expect::Unit => kind == Kind::Unit,
            Expect::Option => matches!(kind, Kind::Unit | Kind::Some),
            Expect::Seq => kind == Kind::Seq,
            Expect::Map => kind == Kind::Map,
            Expect::Enum => kind == Kind::Enum,
        }
    }
}

/// Deserializer wrapper. Forwards to `inner` and checks the shape it receives.
pub(super) struct Strict<D> {
    inner: D,
    mode: Mode,
}

impl<D> Strict<D> {
    pub(super) fn new(inner: D, mode: Mode) -> Self {
        Self { inner, mode }
    }
}

/// Visitor wrapper. Rejects any visit the expected shape does not allow and
/// wraps every nested deserializer or access it passes on.
struct StrictVisitor<V> {
    inner: V,
    expect: Expect,
    mode: Mode,
}

impl<V> StrictVisitor<V> {
    fn new(inner: V, expect: Expect, mode: Mode) -> Self {
        Self {
            inner,
            expect,
            mode,
        }
    }
}

impl<'de, V: Visitor<'de>> StrictVisitor<V> {
    fn gate<E: de::Error>(&self, kind: Kind, found: Unexpected) -> Result<(), E> {
        if self.expect.allows(kind) {
            Ok(())
        } else {
            Err(E::invalid_type(found, self))
        }
    }
}

macro_rules! gated_int_visits {
    ($($name:ident($ty:ty)),* $(,)?) => {$(
        fn $name<E: de::Error>(self, v: $ty) -> Result<V::Value, E> {
            self.gate::<E>(Kind::Int, Unexpected::Other("integer"))?;
            self.inner.$name(v)
        }
    )*};
}

macro_rules! gated_float_visits {
    ($($name:ident($ty:ty)),* $(,)?) => {$(
        fn $name<E: de::Error>(self, v: $ty) -> Result<V::Value, E> {
            self.gate::<E>(Kind::Float, Unexpected::Other("float"))?;
            self.inner.$name(v)
        }
    )*};
}

macro_rules! gated_str_visits {
    ($($name:ident($ty:ty)),* $(,)?) => {$(
        fn $name<E: de::Error>(self, v: $ty) -> Result<V::Value, E> {
            self.gate::<E>(Kind::Str, Unexpected::Str(&v[..]))?;
            self.inner.$name(v)
        }
    )*};
}

impl<'de, V: Visitor<'de>> Visitor<'de> for StrictVisitor<V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<V::Value, E> {
        self.gate::<E>(Kind::Bool, Unexpected::Bool(v))?;
        self.inner.visit_bool(v)
    }

    gated_int_visits!(
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_i128(i128),
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_u128(u128),
    );

    gated_float_visits!(visit_f32(f32), visit_f64(f64));

    fn visit_char<E: de::Error>(self, v: char) -> Result<V::Value, E> {
        self.gate::<E>(Kind::Str, Unexpected::Char(v))?;
        self.inner.visit_char(v)
    }

    gated_str_visits!(
        visit_str(&str),
        visit_borrowed_str(&'de str),
        visit_string(String)
    );

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<V::Value, E> {
        self.gate::<E>(Kind::Bytes, Unexpected::Bytes(v))?;
        self.inner.visit_bytes(v)
    }

    fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<V::Value, E> {
        self.gate::<E>(Kind::Bytes, Unexpected::Bytes(v))?;
        self.inner.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<V::Value, E> {
        self.gate::<E>(Kind::Bytes, Unexpected::Bytes(&v))?;
        self.inner.visit_byte_buf(v)
    }

    fn visit_unit<E: de::Error>(self) -> Result<V::Value, E> {
        self.gate::<E>(Kind::Unit, Unexpected::Unit)?;
        self.inner.visit_unit()
    }

    fn visit_none<E: de::Error>(self) -> Result<V::Value, E> {
        self.gate::<E>(Kind::Unit, Unexpected::Option)?;
        self.inner.visit_none()
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<V::Value, D::Error> {
        self.gate::<D::Error>(Kind::Some, Unexpected::Option)?;
        self.inner.visit_some(Strict::new(deserializer, self.mode))
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<V::Value, D::Error> {
        self.gate::<D::Error>(Kind::Newtype, Unexpected::NewtypeStruct)?;
        self.inner
            .visit_newtype_struct(Strict::new(deserializer, self.mode))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<V::Value, A::Error> {
        self.gate::<A::Error>(Kind::Seq, Unexpected::Seq)?;
        self.inner.visit_seq(StrictSeq {
            inner: seq,
            mode: self.mode,
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<V::Value, A::Error> {
        self.gate::<A::Error>(Kind::Map, Unexpected::Map)?;
        self.inner.visit_map(StrictMap {
            inner: map,
            mode: self.mode,
        })
    }

    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<V::Value, A::Error> {
        self.gate::<A::Error>(Kind::Enum, Unexpected::Enum)?;
        self.inner.visit_enum(StrictEnum {
            inner: data,
            mode: self.mode,
        })
    }
}

/// Visitor used for boolean targets in `Mode::Spelling`. It receives the scalar's
/// source text and accepts only the exact words `true` and `false`, so `tRue`,
/// `True` and `TRUE` are rejected even though the shape pass reads them as booleans.
struct BoolSpelling<V>(V);

impl<'de, V: Visitor<'de>> Visitor<'de> for BoolSpelling<V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<V::Value, E> {
        match text {
            "true" => self.0.visit_bool(true),
            "false" => self.0.visit_bool(false),
            _ => Err(E::invalid_value(
                Unexpected::Str(text),
                &"boolean spelled exactly `true` or `false`",
            )),
        }
    }
}

/// Visitor used for integer targets in `Mode::Spelling`. It receives the scalar's
/// source text, accepts only canonical decimal text and then forwards the parsed
/// value to the caller's integer visitor.
struct Spelling<V>(V);

const CANONICAL_INTEGER: &str =
    "canonical decimal integer (no sign, underscore, leading zero, radix prefix, exponent or -0)";

/// `-?(0|[1-9][0-9]*)`, excluding `-0`.
fn is_canonical_integer(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let shape_ok = match digits.as_bytes() {
        [b'0'] => true,
        [b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
        _ => false,
    };
    shape_ok && text != "-0"
}

impl<'de, V: Visitor<'de>> Visitor<'de> for Spelling<V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<V::Value, E> {
        if !is_canonical_integer(text) {
            return Err(E::invalid_value(Unexpected::Str(text), &CANONICAL_INTEGER));
        }
        if text.starts_with('-') {
            if let Ok(value) = text.parse::<i64>() {
                return self.0.visit_i64(value);
            }
            let value: i128 = text
                .parse()
                .map_err(|_| E::invalid_value(Unexpected::Str(text), &"integer within i128"))?;
            return self.0.visit_i128(value);
        }
        if let Ok(value) = text.parse::<u64>() {
            return self.0.visit_u64(value);
        }
        let value: u128 = text
            .parse()
            .map_err(|_| E::invalid_value(Unexpected::Str(text), &"integer within u128"))?;
        self.0.visit_u128(value)
    }
}

impl<'de, D: Deserializer<'de>> Deserializer<'de> for Strict<D> {
    type Error = D::Error;

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Any, self.mode))
    }

    /// Booleans: shape pass goes through `deserialize_any`, because the typed path
    /// would turn a quoted `"false"` into `false`. Spelling pass reads the source
    /// text with `deserialize_str` and requires exactly `true` or `false`.
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        match self.mode {
            Mode::Shape => {
                self.inner
                    .deserialize_any(StrictVisitor::new(visitor, Expect::Bool, Mode::Shape))
            }
            Mode::Spelling => self.inner.deserialize_str(BoolSpelling(visitor)),
        }
    }

    /// Integers: shape pass reads the value with `deserialize_any`; spelling pass
    /// reads the source text with `deserialize_str`.
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }
    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.deserialize_integer(visitor)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Float, self.mode))
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Float, self.mode))
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_char(StrictVisitor::new(visitor, Expect::Str, self.mode))
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_str(StrictVisitor::new(visitor, Expect::Str, self.mode))
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_string(StrictVisitor::new(visitor, Expect::Str, self.mode))
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_bytes(StrictVisitor::new(visitor, Expect::Any, self.mode))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_byte_buf(StrictVisitor::new(visitor, Expect::Any, self.mode))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_option(StrictVisitor::new(visitor, Expect::Option, self.mode))
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_unit(StrictVisitor::new(visitor, Expect::Unit, self.mode))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_unit_struct(name, StrictVisitor::new(visitor, Expect::Any, self.mode))
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_newtype_struct(name, StrictVisitor::new(visitor, Expect::Any, self.mode))
    }

    /// Sequences, tuples, maps and structs also go through `deserialize_any`: the
    /// typed paths turn a null value into an empty collection. `deserialize_any`
    /// reports null as `visit_unit`, which these expectations reject.
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Seq, self.mode))
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Seq, self.mode))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Seq, self.mode))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Map, self.mode))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_any(StrictVisitor::new(visitor, Expect::Map, self.mode))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.inner.deserialize_enum(
            name,
            variants,
            StrictVisitor::new(visitor, Expect::Enum, self.mode),
        )
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_identifier(StrictVisitor::new(visitor, Expect::Any, self.mode))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.inner
            .deserialize_ignored_any(StrictVisitor::new(visitor, Expect::Any, self.mode))
    }
}

impl<'de, D: Deserializer<'de>> Strict<D> {
    fn deserialize_integer<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        match self.mode {
            Mode::Shape => {
                self.inner
                    .deserialize_any(StrictVisitor::new(visitor, Expect::Int, Mode::Shape))
            }
            Mode::Spelling => self.inner.deserialize_str(Spelling(visitor)),
        }
    }
}

/// Wraps a seed so the value it deserializes is also checked.
struct StrictSeed<S> {
    seed: S,
    mode: Mode,
}

impl<'de, S: DeserializeSeed<'de>> DeserializeSeed<'de> for StrictSeed<S> {
    type Value = S::Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<S::Value, D::Error> {
        self.seed.deserialize(Strict::new(deserializer, self.mode))
    }
}

struct StrictSeq<A> {
    inner: A,
    mode: Mode,
}

impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for StrictSeq<A> {
    type Error = A::Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, A::Error> {
        self.inner.next_element_seed(StrictSeed {
            seed,
            mode: self.mode,
        })
    }

    fn size_hint(&self) -> Option<usize> {
        self.inner.size_hint()
    }
}

struct StrictMap<A> {
    inner: A,
    mode: Mode,
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for StrictMap<A> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, A::Error> {
        self.inner.next_key_seed(StrictSeed {
            seed,
            mode: self.mode,
        })
    }

    fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value, A::Error> {
        self.inner.next_value_seed(StrictSeed {
            seed,
            mode: self.mode,
        })
    }

    fn size_hint(&self) -> Option<usize> {
        self.inner.size_hint()
    }
}

struct StrictEnum<A> {
    inner: A,
    mode: Mode,
}

impl<'de, A: EnumAccess<'de>> EnumAccess<'de> for StrictEnum<A> {
    type Error = A::Error;
    type Variant = StrictVariant<A::Variant>;

    fn variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<(T::Value, Self::Variant), A::Error> {
        let mode = self.mode;
        let (value, variant) = self.inner.variant_seed(StrictSeed { seed, mode })?;
        Ok((
            value,
            StrictVariant {
                inner: variant,
                mode,
            },
        ))
    }
}

struct StrictVariant<A> {
    inner: A,
    mode: Mode,
}

impl<'de, A: VariantAccess<'de>> VariantAccess<'de> for StrictVariant<A> {
    type Error = A::Error;

    fn unit_variant(self) -> Result<(), A::Error> {
        self.inner.unit_variant()
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, A::Error> {
        self.inner.newtype_variant_seed(StrictSeed {
            seed,
            mode: self.mode,
        })
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, A::Error> {
        self.inner
            .tuple_variant(len, StrictVisitor::new(visitor, Expect::Seq, self.mode))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, A::Error> {
        self.inner
            .struct_variant(fields, StrictVisitor::new(visitor, Expect::Map, self.mode))
    }
}
