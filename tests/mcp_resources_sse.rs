use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
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
fn sse_resources_list_then_read() {
	// Create temp directory with a golem.yaml for discovery
	let tmp = tempfile::tempdir().expect("tmpdir");
	let yaml_path = tmp.path().join("golem.yaml");
	let mut f = File::create(&yaml_path).expect("create yaml");
	writeln!(f, "name: test-app\nversion: 0.1.0").unwrap();

	let port: u16 = 1241;
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
		.current_dir(tmp.path())
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

	// resources.list
	let list_id = "resources-list-sse-1";
	let list_req = format!(
		"{{\"jsonrpc\":\"2.0\",\"id\":\"{}\",\"method\":\"resources/list\",\"params\":{{}}}}",
		list_id
	);
	let r = Client::new()
		.post(&format!("{}/mcp", base))
		.header(ACCEPT, "application/json")
		.header(CONTENT_TYPE, "application/json")
		.body(list_req)
		.send()
		.expect("send resources.list");
	assert!(r.status().is_success());

	let start = Instant::now();
	let mut saw_list = false;
	while start.elapsed() < Duration::from_secs(20) {
		match rx.recv_timeout(Duration::from_millis(500)) {
			Ok(data) => {
				if data.contains(list_id) && data.contains("golem.yaml") {
					saw_list = true;
					break;
				}
			}
			Err(_) => {}
		}
	}
	assert!(saw_list, "did not receive resources.list result over SSE");

	// resources.read for our file
	let read_id = "resources-read-sse-1";
	let uri = format!("file://{}", yaml_path.to_string_lossy());
	let read_req = format!(
		"{{\"jsonrpc\":\"2.0\",\"id\":\"{}\",\"method\":\"resources/read\",\"params\":{{\"uri\":\"{}\"}}}}",
		read_id, uri.replace('\\', "\\\\")
	);
	let r2 = Client::new()
		.post(&format!("{}/mcp", base))
		.header(ACCEPT, "application/json")
		.header(CONTENT_TYPE, "application/json")
		.body(read_req)
		.send()
		.expect("send resources.read");
	assert!(r2.status().is_success());

	let start2 = Instant::now();
	let mut saw_read = false;
	while start2.elapsed() < Duration::from_secs(20) {
		match rx.recv_timeout(Duration::from_millis(500)) {
			Ok(data) => {
				if data.contains(read_id) && data.contains("test-app") {
					saw_read = true;
					break;
				}
			}
			Err(_) => {}
		}
	}
	assert!(saw_read, "did not receive resources.read result over SSE");

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