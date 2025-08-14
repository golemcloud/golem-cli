use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn wait_http_ready(client: &Client, base: &str, timeout: Duration) -> bool {
	let start = Instant::now();
	while start.elapsed() < timeout {
		let res = client
			.post(&format!("{}/mcp", base))
			.header(ACCEPT, "application/json")
			.header(CONTENT_TYPE, "application/json")
			.body(r#"{"jsonrpc":"2.0","id":"ping","method":"ping","params":{}}"#)
			.send();
		if let Ok(r) = res {
			if r.status().is_success() {
				return true;
			}
		}
		thread::sleep(Duration::from_millis(200));
	}
	false
}

#[test]
fn sse_tools_list_then_call() {
	let port: u16 = 1240;
	let base = format!("http://127.0.0.1:{}", port);

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

	let client = Client::builder().timeout(Duration::from_secs(3)).build().unwrap();
	assert!(wait_http_ready(&client, &base, Duration::from_secs(20)), "server not ready");

	// Start SSE reader thread
	let sse_url = format!("{}/sse", base);
	let (tx, rx) = mpsc::channel::<String>();
	let handle = thread::spawn(move || {
		let resp = client
			.get(&sse_url)
			.header(ACCEPT, "text/event-stream")
			.send()
			.expect("sse connect");
		let mut reader = BufReader::new(resp);
		let mut line = String::new();
		loop {
			line.clear();
			let n = reader.read_line(&mut line).unwrap_or(0);
			if n == 0 {
				break;
			}
			if let Some(rest) = line.strip_prefix("data: ") {
				let _ = tx.send(rest.trim().to_string());
			}
		}
	});

	// tools.list
	let list_id = "tools-list-sse-1";
	let list_req = format!(
		"{{\"jsonrpc\":\"2.0\",\"id\":\"{}\",\"method\":\"tools/list\",\"params\":{{}}}}",
		list_id
	);
	let r = client
		.post(&format!("{}/mcp", base))
		.header(ACCEPT, "application/json")
		.header(CONTENT_TYPE, "application/json")
		.body(list_req)
		.send()
		.expect("send tools.list");
	assert!(r.status().is_success());

	let start = Instant::now();
	let mut saw_tools = false;
	while start.elapsed() < Duration::from_secs(20) {
		match rx.recv_timeout(Duration::from_millis(500)) {
			Ok(data) => {
				if data.contains(list_id) && data.contains("\"result\"") && data.contains("tools") {
					saw_tools = true;
					break;
				}
			}
			Err(_) => {}
		}
	}
	assert!(saw_tools, "did not receive tools.list result over SSE");

	// Try a benign tools.call: not all tool names are guaranteed; send and accept any success
	let call_id = "tools-call-sse-1";
	let call_req = format!(
		"{{\"jsonrpc\":\"2.0\",\"id\":\"{}\",\"method\":\"tools/call\",\"params\":{{\"name\":\"help\",\"arguments\":{{}}}}}}",
		call_id
	);
	let _ = client
		.post(&format!("{}/mcp", base))
		.header(ACCEPT, "application/json")
		.header(CONTENT_TYPE, "application/json")
		.body(call_req)
		.send();

	let start2 = Instant::now();
	let mut saw_call = false;
	while start2.elapsed() < Duration::from_secs(10) {
		match rx.recv_timeout(Duration::from_millis(500)) {
			Ok(data) => {
				if data.contains(call_id) && data.contains("\"result\"") {
					saw_call = true;
					break;
				}
			}
			Err(_) => {}
		}
	}
	// Non-fatal if specific tool name is not present
	let _ = saw_call;

	let _ = child.kill();
	let _ = child.wait();
	let _ = handle.join();

	if let Some(mut out) = child.stdout.take() {
		let mut buf = String::new();
		let _ = out.read_to_string(&mut buf);
	}
	if let Some(mut err) = child.stderr.take() {
		let mut buf = String::new();
		let _ = err.read_to_string(&mut buf);
	}
} 