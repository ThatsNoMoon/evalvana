#![cfg_attr(windows, windows_subsystem = "windows")]

use evalvana_api as api;

use std::io::BufRead;

fn main() -> std::io::Result<()> {
	let mut state = 0u8;
	let stdin = std::io::stdin();
	for line in stdin.lock().lines() {
		let line = line?;

		let call: api::EvalStringCall = serde_json::from_str(&line).unwrap();

		let text = call.params.code.into_owned();

		let result = match state {
			0 => api::EvalResult::Success(api::EvalMessage { text }),
			1 => api::EvalResult::Warning(api::EvalMessage { text }),
			2 => api::EvalResult::Error(api::EvalMessage { text }),
			_ => unreachable!(),
		};

		state = (state + 1) % 3;

		let response = api::EvalResponse {
			rpc: api::RpcMessage::new(Some(call.rpc.id)),
			data: api::RpcResponseResult::Success(vec![result]),
		};

		println!("{}", serde_json::to_string(&response).unwrap());
	}

	Ok(())
}
