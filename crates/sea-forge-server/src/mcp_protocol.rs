//! A single, duplicate-aware interpretation for MCP authorization and dispatch.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};
use std::fmt;

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueValue;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("unambiguous JSON")
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Bool(value)))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(value.into())))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(value.into())))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Number::from_f64(value)
                    .map(|number| UniqueValue(Value::Number(number)))
                    .ok_or_else(|| E::custom("invalid number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(value.into())))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(value)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut array = Vec::new();
                while let Some(value) = sequence.next_element::<UniqueValue>()? {
                    array.push(value.0);
                }
                Ok(UniqueValue(Value::Array(array)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut object = Map::new();
                while let Some((key, value)) = map.next_entry::<String, UniqueValue>()? {
                    if object.insert(key, value.0).is_some() {
                        return Err(de::Error::custom("duplicate JSON key"));
                    }
                }
                Ok(UniqueValue(Value::Object(object)))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}

#[derive(Debug)]
pub(crate) struct RpcRequest {
    pub value: Value,
    pub id: Option<Value>,
    pub method: String,
}

impl RpcRequest {
    fn parse(value: Value) -> Result<Self, &'static str> {
        let object = value.as_object().ok_or("request must be an object")?;
        if object
            .keys()
            .any(|key| !matches!(key.as_str(), "jsonrpc" | "id" | "method" | "params"))
            || object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        {
            return Err("invalid JSON-RPC envelope");
        }
        let method = object
            .get("method")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or("method must be a string")?
            .to_owned();
        let id = object.get("id").cloned();
        if id
            .as_ref()
            .is_some_and(|id| !matches!(id, Value::String(_) | Value::Number(_) | Value::Null))
        {
            return Err("invalid request id");
        }
        if object
            .get("params")
            .is_some_and(|params| !params.is_object())
        {
            return Err("params must be an object");
        }
        match method.as_str() {
            "tools/call" => {
                if value
                    .pointer("/params/name")
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
                    || value
                        .pointer("/params/arguments")
                        .is_some_and(|args| !args.is_object())
                    || id.is_none()
                {
                    return Err("invalid tool call");
                }
            }
            "initialize" if id.is_none() => return Err("initialize requires an id"),
            "notifications/initialized" if id.is_some() => {
                return Err("initialized must be a notification")
            }
            _ => {}
        }
        Ok(Self { value, id, method })
    }
}

pub(crate) fn decode_json(bytes: &[u8]) -> Result<Value, String> {
    serde_json::from_slice::<UniqueValue>(bytes)
        .map(|value| value.0)
        .map_err(|error| error.to_string())
}

pub(crate) fn parse_frame(bytes: &[u8]) -> Result<(bool, Vec<RpcRequest>), String> {
    let value = decode_json(bytes)?;
    let (batch, values) = match value {
        Value::Array(values) if !values.is_empty() && values.len() <= 256 => (true, values),
        Value::Array(_) => return Err("invalid batch size".into()),
        value => (false, vec![value]),
    };
    // Validate the entire batch before any member can reach a resource.
    let requests = values
        .into_iter()
        .map(RpcRequest::parse)
        .collect::<Result<Vec<_>, _>>()
        .map_err(str::to_owned)?;
    Ok((batch, requests))
}

#[cfg(test)]
mod tests {
    use super::parse_frame;
    #[test]
    fn duplicate_equivalent_keys_and_conflicting_envelopes_are_refused() {
        for bytes in [
            br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","meth\u006fd":"tools/call"}"#.as_slice(),
            br#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"safe","arguments":{"x":1,"x":2}}}"#,
            br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","result":{}}"#,
            br#"[{"jsonrpc":"2.0","id":1,"method":"tools/list"},null]"#,
            b"[]", b"{}{}", b"\xff",
        ] { assert!(parse_frame(bytes).is_err()); }
        let (batch, requests) = parse_frame(br#"[{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"metadata.read","arguments":{"nested":["\u03bb",1]}}},{"jsonrpc":"2.0","id":2,"method":"tools/list"}]"#).unwrap();
        assert!(batch);
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].value["params"]["arguments"]["nested"][0], "λ");
    }
}
