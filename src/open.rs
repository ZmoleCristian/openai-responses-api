use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ResponsesError;

pub trait Kinds {
    const KINDS: &'static [&'static str];

    fn untyped(value: Value) -> Result<Value, ResponsesError> {
        Err(ResponsesError::Untyped(value))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    untagged,
    try_from = "Value",
    bound(deserialize = "K: Kinds + DeserializeOwned")
)]
pub enum Open<K> {
    Known(K),
    Unknown(Value),
}

impl<K: Kinds + DeserializeOwned> TryFrom<Value> for Open<K> {
    type Error = ResponsesError;

    fn try_from(value: Value) -> Result<Open<K>, ResponsesError> {
        let value = match value.get("type") {
            Some(Value::String(..)) => value,
            Some(..) | None => K::untyped(value)?,
        };
        let known = match value.get("type") {
            Some(Value::String(kind)) => K::KINDS.contains(&kind.as_str()),
            Some(..) | None => return Err(ResponsesError::Untyped(value)),
        };
        if known {
            return Ok(Open::Known(serde_json::from_value(value)?));
        }
        Ok(Open::Unknown(value))
    }
}
