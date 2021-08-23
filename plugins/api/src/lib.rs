// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use std::{borrow::Cow, fmt, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMessage<Id> {
	#[serde(deserialize_with = "deserialize_jsonrpc")]
	#[serde(serialize_with = "serialize_jsonrpc")]
	jsonrpc: (),
	pub id: Id,
}

impl<Id> RpcMessage<Id> {
	pub fn new(id: Id) -> Self {
		Self { jsonrpc: (), id }
	}
}

struct RpcVersionVisitor;
impl<'de> de::Visitor<'de> for RpcVersionVisitor {
	type Value = ();
	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "2.0")
	}
	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		match v {
			"2.0" => Ok(()),
			other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
		}
	}
}
fn deserialize_jsonrpc<'de, D>(d: D) -> Result<(), D::Error>
where
	D: Deserializer<'de>,
{
	d.deserialize_str(RpcVersionVisitor)
}

fn serialize_jsonrpc<S: Serializer>(_: &(), ser: S) -> Result<S::Ok, S::Error> {
	ser.serialize_str("2.0")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMethodCall<'id, 'm, Args> {
	#[serde(flatten)]
	pub rpc: RpcMessage<Cow<'id, str>>,
	pub method: Cow<'m, str>,
	pub params: Args,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<'id, 'e, T> {
	#[serde(flatten)]
	pub rpc: RpcMessage<Option<Cow<'id, str>>>,
	#[serde(flatten)]
	pub data: RpcResponseResult<'e, T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcResponseResult<'e, T> {
	#[serde(rename = "result")]
	Success(T),
	#[serde(rename = "error")]
	Failure(RpcError<'e>),
}

impl<'e, T> From<RpcResponseResult<'e, T>> for Result<T, RpcError<'e>> {
	fn from(result: RpcResponseResult<'e, T>) -> Self {
		match result {
			RpcResponseResult::Success(t) => Ok(t),
			RpcResponseResult::Failure(e) => Err(e),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError<'m> {
	pub code: i32,
	pub message: Cow<'m, str>,
}

impl<'m> fmt::Display for RpcError<'m> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "jsonrpc error: {} - {}", self.code, self.message)
	}
}

impl<'m> std::error::Error for RpcError<'m> {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvalStringArgs<'s> {
	pub code: Cow<'s, str>,
}

pub type EvalStringCall<'id, 'n, 's> =
	RpcMethodCall<'id, 'n, EvalStringArgs<'s>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvalFileArgs<'p> {
	pub path: Cow<'p, Path>,
}

pub type EvalFileCall<'id, 'n, 'p> = RpcMethodCall<'id, 'n, EvalFileArgs<'p>>;

pub type EvalResponse<'id, 'e> = RpcResponse<'id, 'e, Vec<EvalResult>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "level", content = "text")]
pub enum EvalResult {
	Success(EvalMessage),
	Warning(EvalMessage),
	Error(EvalMessage),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvalMessage {
	pub text: String,
}
