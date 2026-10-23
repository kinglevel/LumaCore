// ███████╗██╗███╗   ██╗ ██████╗ ███████╗██████╗ ██████╗ ██████╗ ██╗███╗   ██╗████████╗
// ██╔════╝██║████╗  ██║██╔════╝ ██╔════╝██╔══██╗██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝
// █████╗  ██║██╔██╗ ██║██║  ███╗█████╗  ██████╔╝██████╔╝██████╔╝██║██╔██╗ ██║   ██║
// ██╔══╝  ██║██║╚██╗██║██║   ██║██╔══╝  ██╔══██╗██╔═══╝ ██╔══██╗██║██║╚██╗██║   ██║
// ██║     ██║██║ ╚████║╚██████╔╝███████╗██║  ██║██║     ██║  ██║██║██║ ╚████║   ██║
// ╚═╝     ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝
// FILE: fingerprint
//! A fingerprint of anything `Serialize` (decisions §40 "40f built": the change path on a big
//! show): the value walks into a hasher as serde hands it over — no text, no allocation, no
//! float formatting — so hashing a subtree costs what visiting it costs. Every kind of value is
//! tagged and every sequence, map and string carries its length, so two different trees never
//! hash alike by accident (`[1, 2]` and `[12]`, `Some(0)` and `None`, a field moved between
//! structs). The same value hashes the same on every station of a build; the hash is never
//! stored.
//!
//! Owns: the serializer and `of`.
//! Doesn't own: what a subtree is (`lumacore-tree`'s `ShowSubtree`), the images the change
//! path keeps as JSON (`oops::snapshot_subtree`).
//! Entry points: `of`, `into`.
//! Decisions: §26 (the one change path), §40 "40f built".

use std::hash::Hasher;

use serde::Serialize;
use serde::ser;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The serializer: every call appends to the buffer (one hasher pass over it at the end — a
/// hasher call per value would cost more than the walk).
pub struct Fingerprint<'a>(&'a mut Vec<u8>);

/// Serialization never fails here; the type exists for serde's signatures.
#[derive(Debug)]
pub struct Never;

impl std::fmt::Display for Never {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fingerprint")
    }
}

impl std::error::Error for Never {}

impl ser::Error for Never {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Never
    }
}

/// One byte per kind of value, so a change of kind is a change.
#[repr(u8)]
enum Tag {
    Bool = 1,
    Int = 2,
    Uint = 3,
    Float = 4,
    Char = 5,
    Str = 6,
    Bytes = 7,
    None = 8,
    Some = 9,
    Unit = 10,
    UnitStruct = 11,
    UnitVariant = 12,
    NewtypeStruct = 13,
    NewtypeVariant = 14,
    Seq = 15,
    Tuple = 16,
    TupleStruct = 17,
    TupleVariant = 18,
    Map = 19,
    Struct = 20,
    StructVariant = 21,
    Field = 22,
    Key = 23,
    Value = 24,
    End = 25,
    Unknown = 26,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The fingerprint of a value (`DefaultHasher`, the same for one build).
pub fn of<T: Serialize + ?Sized>(value: &T) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    into(value, &mut h);
    h.finish()
}

/// Feed a value into `hasher`.
pub fn into<T: Serialize + ?Sized, H: Hasher>(value: &T, hasher: &mut H) {
    thread_local! {
        static BUF: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        let _ = value.serialize(Fingerprint(&mut buf));
        hasher.write(&buf);
    });
}

impl Fingerprint<'_> {
    fn tag(&mut self, t: Tag) {
        self.0.push(t as u8);
    }

    fn u64(&mut self, n: u64) {
        self.0.extend_from_slice(&n.to_le_bytes());
    }

    fn len(&mut self, n: Option<usize>) {
        match n {
            Some(n) => self.u64(n as u64),
            None => self.tag(Tag::Unknown),
        }
    }

    fn str(&mut self, s: &str) {
        self.u64(s.len() as u64);
        self.0.extend_from_slice(s.as_bytes());
    }
}

impl<'a> ser::Serializer for Fingerprint<'a> {
    type Ok = ();
    type Error = Never;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(mut self, v: bool) -> Result<(), Never> {
        self.tag(Tag::Bool);
        self.0.push(v as u8);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Never> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<(), Never> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<(), Never> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(mut self, v: i64) -> Result<(), Never> {
        self.tag(Tag::Int);
        self.u64(v as u64);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Never> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<(), Never> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<(), Never> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(mut self, v: u64) -> Result<(), Never> {
        self.tag(Tag::Uint);
        self.u64(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Never> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(mut self, v: f64) -> Result<(), Never> {
        self.tag(Tag::Float);
        // Bits, so −0 and 0, or two NaNs, differ as the JSON image would.
        self.u64(v.to_bits());
        Ok(())
    }

    fn serialize_char(mut self, v: char) -> Result<(), Never> {
        self.tag(Tag::Char);
        self.u64(v as u64);
        Ok(())
    }

    fn serialize_str(mut self, v: &str) -> Result<(), Never> {
        self.tag(Tag::Str);
        self.str(v);
        Ok(())
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<(), Never> {
        self.tag(Tag::Bytes);
        self.u64(v.len() as u64);
        self.0.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(mut self) -> Result<(), Never> {
        self.tag(Tag::None);
        Ok(())
    }

    fn serialize_some<T: Serialize + ?Sized>(mut self, v: &T) -> Result<(), Never> {
        self.tag(Tag::Some);
        v.serialize(self)
    }

    fn serialize_unit(mut self) -> Result<(), Never> {
        self.tag(Tag::Unit);
        Ok(())
    }

    fn serialize_unit_struct(mut self, name: &'static str) -> Result<(), Never> {
        self.tag(Tag::UnitStruct);
        self.str(name);
        Ok(())
    }

    fn serialize_unit_variant(mut self, name: &'static str, index: u32, variant: &'static str) -> Result<(), Never> {
        self.tag(Tag::UnitVariant);
        self.str(name);
        self.u64(u64::from(index));
        self.str(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(mut self, name: &'static str, v: &T) -> Result<(), Never> {
        self.tag(Tag::NewtypeStruct);
        self.str(name);
        v.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(mut self, name: &'static str, index: u32, variant: &'static str, v: &T) -> Result<(), Never> {
        self.tag(Tag::NewtypeVariant);
        self.str(name);
        self.u64(u64::from(index));
        self.str(variant);
        v.serialize(self)
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self, Never> {
        self.tag(Tag::Seq);
        self.len(len);
        Ok(self)
    }

    fn serialize_tuple(mut self, len: usize) -> Result<Self, Never> {
        self.tag(Tag::Tuple);
        self.len(Some(len));
        Ok(self)
    }

    fn serialize_tuple_struct(mut self, name: &'static str, len: usize) -> Result<Self, Never> {
        self.tag(Tag::TupleStruct);
        self.str(name);
        self.len(Some(len));
        Ok(self)
    }

    fn serialize_tuple_variant(mut self, name: &'static str, index: u32, variant: &'static str, len: usize) -> Result<Self, Never> {
        self.tag(Tag::TupleVariant);
        self.str(name);
        self.u64(u64::from(index));
        self.str(variant);
        self.len(Some(len));
        Ok(self)
    }

    fn serialize_map(mut self, len: Option<usize>) -> Result<Self, Never> {
        self.tag(Tag::Map);
        self.len(len);
        Ok(self)
    }

    fn serialize_struct(mut self, name: &'static str, len: usize) -> Result<Self, Never> {
        self.tag(Tag::Struct);
        self.str(name);
        self.len(Some(len));
        Ok(self)
    }

    fn serialize_struct_variant(mut self, name: &'static str, index: u32, variant: &'static str, len: usize) -> Result<Self, Never> {
        self.tag(Tag::StructVariant);
        self.str(name);
        self.u64(u64::from(index));
        self.str(variant);
        self.len(Some(len));
        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        // As the JSON image: a type that serializes differently for people (uuid, time) takes
        // the readable form here too, so the two agree on what changed.
        true
    }

    fn collect_str<T: std::fmt::Display + ?Sized>(mut self, v: &T) -> Result<(), Never> {
        use std::fmt::Write as _;
        self.tag(Tag::Str);
        let at = self.0.len();
        self.u64(0);
        let mut w = DisplayBuf(self.0);
        let _ = write!(w, "{v}");
        let len = (self.0.len() - at - 8) as u64;
        self.0[at..at + 8].copy_from_slice(&len.to_le_bytes());
        Ok(())
    }
}

/// `Display` output written straight into the buffer (no String).
struct DisplayBuf<'a>(&'a mut Vec<u8>);

impl std::fmt::Write for DisplayBuf<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl ser::SerializeSeq for Fingerprint<'_> {
    type Ok = ();
    type Error = Never;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Never> {
        v.serialize(Fingerprint(&mut *self.0))
    }

    fn end(mut self) -> Result<(), Never> {
        self.tag(Tag::End);
        Ok(())
    }
}

impl ser::SerializeTuple for Fingerprint<'_> {
    type Ok = ();
    type Error = Never;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Never> {
        v.serialize(Fingerprint(&mut *self.0))
    }

    fn end(mut self) -> Result<(), Never> {
        self.tag(Tag::End);
        Ok(())
    }
}

impl ser::SerializeTupleStruct for Fingerprint<'_> {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Never> {
        v.serialize(Fingerprint(&mut *self.0))
    }

    fn end(mut self) -> Result<(), Never> {
        self.tag(Tag::End);
        Ok(())
    }
}

impl ser::SerializeTupleVariant for Fingerprint<'_> {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Never> {
        v.serialize(Fingerprint(&mut *self.0))
    }

    fn end(mut self) -> Result<(), Never> {
        self.tag(Tag::End);
        Ok(())
    }
}

impl ser::SerializeMap for Fingerprint<'_> {
    type Ok = ();
    type Error = Never;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Never> {
        self.tag(Tag::Key);
        k.serialize(Fingerprint(&mut *self.0))
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Never> {
        self.tag(Tag::Value);
        v.serialize(Fingerprint(&mut *self.0))
    }

    fn end(mut self) -> Result<(), Never> {
        self.tag(Tag::End);
        Ok(())
    }
}

impl ser::SerializeStruct for Fingerprint<'_> {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, v: &T) -> Result<(), Never> {
        self.tag(Tag::Field);
        self.str(key);
        v.serialize(Fingerprint(&mut *self.0))
    }

    fn skip_field(&mut self, _: &'static str) -> Result<(), Never> {
        Ok(())
    }

    fn end(mut self) -> Result<(), Never> {
        self.tag(Tag::End);
        Ok(())
    }
}

impl ser::SerializeStructVariant for Fingerprint<'_> {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, v: &T) -> Result<(), Never> {
        self.tag(Tag::Field);
        self.str(key);
        v.serialize(Fingerprint(&mut *self.0))
    }

    fn skip_field(&mut self, _: &'static str) -> Result<(), Never> {
        Ok(())
    }

    fn end(mut self) -> Result<(), Never> {
        self.tag(Tag::End);
        Ok(())
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests {
    use super::of;
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct Fixture {
        name: String,
        no: Option<u32>,
        pos: [f64; 3],
        tags: Vec<String>,
    }

    fn one() -> Fixture {
        Fixture { name: "Spot".into(), no: Some(101), pos: [0.0, 1.5, -2.0], tags: vec!["a".into()] }
    }

    #[test]
    fn the_same_value_hashes_the_same_and_a_change_moves_it() {
        assert_eq!(of(&one()), of(&one()));
        let mut f = one();
        f.pos[1] = 1.5000001;
        assert_ne!(of(&one()), of(&f));
        let mut f = one();
        f.no = None;
        assert_ne!(of(&one()), of(&f));
        let mut f = one();
        f.name = "spot".into();
        assert_ne!(of(&one()), of(&f));
    }

    #[test]
    fn shapes_do_not_collide() {
        assert_ne!(of(&vec![1u32, 2]), of(&vec![12u32]));
        assert_ne!(of(&vec!["ab".to_string()]), of(&vec!["a".to_string(), "b".to_string()]));
        assert_ne!(of(&Some(0u32)), of(&None::<u32>));
        assert_ne!(of(&0.0f64), of(&-0.0f64));
        assert_ne!(of(&1u32), of(&1i32));
        let a: BTreeMap<u32, &str> = [(1, "x")].into();
        let b: BTreeMap<u32, &str> = [(2, "x")].into();
        assert_ne!(of(&a), of(&b));
    }

    #[test]
    fn a_readable_type_hashes_its_readable_form() {
        let u = uuid::Uuid::from_u128(7);
        // Twice the same: the Display path is deterministic.
        assert_eq!(of(&u), of(&u));
        assert_ne!(of(&u), of(&uuid::Uuid::from_u128(8)));
    }
}
