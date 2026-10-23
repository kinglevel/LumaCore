//! Bytes that hold JSON, carried as JSON (decisions §31 "Session stream"): a snapshot's bytes
//! (`SubtreeSnapshot.data`, `LocalSnapshot`) go into the stream as the JSON they are — not as a
//! list of numbers three or four times the size, and not escaped into a string. Bytes that
//! aren't JSON travel as `{"b64": "…"}`. Use with `#[serde(with = "crate::embedded_json")]`.

use base64::Engine;
use serde::de::Error as _;
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;

#[derive(Serialize, Deserialize)]
struct Encoded {
    b64: String,
}

pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    match std::str::from_utf8(bytes).ok().and_then(|t| serde_json::from_str::<&RawValue>(t).ok()) {
        Some(raw) => raw.serialize(s),
        None if bytes.is_empty() => Encoded { b64: String::new() }.serialize(s),
        None => Encoded { b64: base64::engine::general_purpose::STANDARD.encode(bytes) }
            .serialize(s)
            .map_err(|e| S::Error::custom(e.to_string())),
    }
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let raw: Box<RawValue> = Deserialize::deserialize(d)?;
    if let Ok(Encoded { b64 }) = serde_json::from_str::<Encoded>(raw.get()) {
        return base64::engine::general_purpose::STANDARD.decode(b64).map_err(D::Error::custom);
    }
    Ok(raw.get().as_bytes().to_vec())
}

/// A `String` that holds JSON (a whole show), carried as that JSON — not escaped into a JSON
/// string. Text that isn't JSON travels as a string.
pub mod text {
    use super::*;

    pub fn serialize<S: Serializer>(text: &str, s: S) -> Result<S::Ok, S::Error> {
        match serde_json::from_str::<&RawValue>(text) {
            Ok(raw) => raw.serialize(s),
            Err(_) => s.serialize_str(text),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        let raw: Box<RawValue> = Deserialize::deserialize(d)?;
        Ok(match serde_json::from_str::<String>(raw.get()) {
            Ok(plain) => plain,
            Err(_) => raw.get().to_string(),
        })
    }
}

/// [`text`] for an `Option<String>` (`null` = none).
pub mod text_opt {
    use super::*;

    pub fn serialize<S: Serializer>(text: &Option<String>, s: S) -> Result<S::Ok, S::Error> {
        match text {
            Some(t) => text::serialize(t, s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
        let raw: Option<Box<RawValue>> = Deserialize::deserialize(d)?;
        Ok(raw.map(|raw| match serde_json::from_str::<String>(raw.get()) {
            Ok(plain) => plain,
            Err(_) => raw.get().to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Holder {
        #[serde(with = "super")]
        data: Vec<u8>,
    }

    #[test]
    fn json_bytes_travel_as_json_and_others_as_base64() {
        let json = Holder { data: br#"{"a":[1,2,{"b":"x"}]}"#.to_vec() };
        let text = serde_json::to_string(&json).unwrap();
        assert_eq!(text, r#"{"data":{"a":[1,2,{"b":"x"}]}}"#);
        assert_eq!(serde_json::from_str::<Holder>(&text).unwrap(), json);
        let binary = Holder { data: vec![0, 159, 146, 150] };
        let text = serde_json::to_string(&binary).unwrap();
        assert!(text.contains("\"b64\""), "{text}");
        assert_eq!(serde_json::from_str::<Holder>(&text).unwrap(), binary);
        let empty = Holder { data: Vec::new() };
        assert_eq!(serde_json::from_str::<Holder>(&serde_json::to_string(&empty).unwrap()).unwrap(), empty);
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Show {
        #[serde(with = "super::text")]
        show: String,
        #[serde(with = "super::text_opt", default)]
        base: Option<String>,
    }

    #[test]
    fn json_text_travels_as_json() {
        let show = Show { show: r#"{"a":"q\"x"}"#.into(), base: Some("not json".into()) };
        let text = serde_json::to_string(&show).unwrap();
        assert_eq!(text, r#"{"show":{"a":"q\"x"},"base":"not json"}"#);
        assert_eq!(serde_json::from_str::<Show>(&text).unwrap(), show);
        let none = Show { show: "{}".into(), base: None };
        assert_eq!(serde_json::from_str::<Show>(&serde_json::to_string(&none).unwrap()).unwrap(), none);
        assert_eq!(serde_json::from_str::<Show>(r#"{"show":{}}"#).unwrap(), none);
    }
}
