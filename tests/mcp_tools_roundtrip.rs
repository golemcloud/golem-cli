use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::{io::Read, thread};

// NOTE: HTTP JSON-RPC endpoint tests are deferred; SSE is the required transport.
// This test is kept as a placeholder and is ignored until SSE-client wiring lands.
#[ignore]
#[test]
fn http_tools_list_roundtrip() {
	// Spawn the CLI server in serve mode with the mcp-serve feature via cargo run
	let port: u16 = 1236;
	let mut child = Command::new("cargo")
		.args([
			"run",
			"--features",
			"mcp-serve",
			"--",
			"--serve",
			"--serve-port",
			&port.to_string(),
		])
		.env("RUST_LOG", "info")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("failed to spawn server");

	// Wait for HTTP endpoint to become ready
	let base = format!("http://127.0.0.1:{}/mcp", port);
	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(2))
		.build()
		.expect("client");

	let start = Instant::now();
	let ready = loop {
		if start.elapsed() > Duration::from_secs(20) {
			break false;
		}
		let res = client
			.post(&base)
			.header("Accept", "application/json")
			.header("Content-Type", "application/json")
			.body(r#"{"jsonrpc":"2.0","id":"ping","method":"ping","params":{}}"#)
			.send();
		match res {
			Ok(r) if r.status().is_success() => break true,
			_ => thread::sleep(Duration::from_millis(200)),
		}
	};
	assert!(ready, "server was not ready in time");

	// tools.list
	let body = r#"{"jsonrpc":"2.0","id":"tools-list","method":"tools/list","params":{}}"#;
	let resp = client
		.post(&base)
		.header("Accept", "application/json")
		.header("Content-Type", "application/json")
		.body(body)
		.send()
		.expect("tools.list send");
	assert!(resp.status().is_success(), "status = {}", resp.status());
	let text = resp.text().unwrap_or_default();
	assert!(text.contains("\"result\""), "missing result: {}", text);
	assert!(text.contains("tools"), "missing tools in result: {}", text);

	// Cleanup process
	let _ = child.kill();
	let _ = child.wait();

	// Drain output (best effort)
	if let Some(mut out) = child.stdout.take() {
		let mut buf = String::new();
		let _ = out.read_to_string(&mut buf);
	}
	if let Some(mut err) = child.stderr.take() {
		let mut buf = String::new();
		let _ = err.read_to_string(&mut buf);
	}
} 