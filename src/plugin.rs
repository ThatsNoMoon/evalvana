// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use std::{
	borrow::Cow, cell::Cell, fmt, hash::Hasher, path::PathBuf, process::Stdio,
	sync::Arc,
};

use anyhow::Result;
use evalvana_api::{
	EvalResponse, EvalStringArgs, EvalStringCall, RpcMessage, RpcMethodCall,
};
use iced_futures::{subscription::Recipe, BoxStream};
use serde::{de, Deserialize, Deserializer, Serialize};
use tokio::{
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
	process::{Child, ChildStdout, Command},
};
use tokio_stream::{wrappers::LinesStream, StreamExt};

#[derive(Debug, Clone, Deserialize)]
pub struct Plugin {
	#[serde(deserialize_with = "deserialize_plugin_name")]
	pub name: Arc<str>,
	pub program: PathBuf,
	pub args: Vec<String>,
	#[serde(skip)]
	env_seq: u32,
}

struct PluginNameVisitor;
impl<'de> de::Visitor<'de> for PluginNameVisitor {
	type Value = Arc<str>;
	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a string")
	}
	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Ok(v.into())
	}
}
fn deserialize_plugin_name<'de, D>(d: D) -> Result<Arc<str>, D::Error>
where
	D: Deserializer<'de>,
{
	d.deserialize_str(PluginNameVisitor)
}

impl Plugin {
	pub fn open(&mut self) -> Result<(Environment, EnvironmentOutput)> {
		let mut child = Command::new(&self.program)
			.args(&self.program)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?;

		let output = EnvironmentOutput::new(
			child
				.stdout
				.take()
				.expect("Plugin child process had no stdout"),
		);

		let env = Environment {
			plugin_name: self.name.clone(),
			id: format!("{}/{}", self.name, self.env_seq).into(),
			process: child,
			call_seq: 0,
		};

		self.env_seq += 1;

		Ok((env, output))
	}
}

#[derive(Debug)]
pub struct Environment {
	pub plugin_name: Arc<str>,
	pub id: Arc<str>,
	process: Child,
	call_seq: u32,
}

impl Environment {
	async fn send_method_call<Args: Serialize>(
		&mut self,
		call: &RpcMethodCall<'_, '_, Args>,
	) -> Result<()> {
		let input = self
			.process
			.stdin
			.as_mut()
			.expect("Plugin child process had no stdin");

		let mut bytes = serde_json::to_vec(call)?;

		// Just a sanity check, as a newline in the middle of a single message
		// could completely ruin every plugin
		debug_assert!(!bytes.iter().any(|&x| x == b'\n'));

		bytes.push(b'\n');

		input.write_all(&bytes).await?;

		input.flush().await?;

		Ok(())
	}

	pub async fn eval_string(&mut self, code: &str) -> Result<()> {
		let args = EvalStringArgs {
			code: Cow::Borrowed(code),
		};

		let id = format!("{}/{}", self.id, self.call_seq);

		let call = EvalStringCall {
			rpc: RpcMessage::new(Cow::Borrowed(&id)),
			method: Cow::Borrowed("eval-string"),
			params: args,
		};

		self.send_method_call(&call).await?;

		Ok(())
	}

	pub async fn kill(&mut self) -> Result<()> {
		self.process.kill().await.map_err(Into::into)
	}
}

pub struct EnvironmentOutput {
	inner: Cell<Option<ChildStdout>>,
	hash: u128,
}

impl fmt::Debug for EnvironmentOutput {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("EnvironmentOutput")
			.field("hash", &self.hash)
			.field("inner", &"[ChildStdout]")
			.finish()
	}
}

impl EnvironmentOutput {
	fn new(inner: ChildStdout) -> Self {
		// goal is to just make a collision practically impossible, since this
		// value is used by `iced` and seems to be assumed to be unique.
		let mut bytes = [0; 16];
		getrandom::getrandom(&mut bytes)
			.expect("Failed to generate random hash");
		Self {
			inner: Cell::new(Some(inner)),
			hash: u128::from_ne_bytes(bytes),
		}
	}

	pub fn take(&self) -> EnvironmentOutput {
		Self {
			inner: Cell::new(self.inner.take()),
			hash: self.hash,
		}
	}
}

impl<H: Hasher, E> Recipe<H, E> for EnvironmentOutput {
	type Output = Result<EvalResponse<'static, 'static>>;

	fn hash(&self, state: &mut H) {
		state.write_u128(self.hash);
	}

	fn stream(self: Box<Self>, _: BoxStream<E>) -> BoxStream<Self::Output> {
		let output = self
			.inner
			.take()
			.expect("Tried to use empty EnvironmentOutput");

		Box::pin(
			LinesStream::new(BufReader::new(output).lines())
				.map(|line| Ok(serde_json::from_str(line?.as_str())?)),
		)
	}
}
