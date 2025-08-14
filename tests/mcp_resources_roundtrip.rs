use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::{io::Read, thread};

// NOTE: HTTP JSON-RPC endpoint tests are deferred; SSE is the required transport.
// This test is kept as a placeholder and is ignored until SSE-client wiring lands.
#[ignore]
#[test]
fn http_resources_list_and_read_roundtrip() {
	// Create a temporary golem.yaml in CWD
	let tmp = tempfile::tempdir().expect("tmpdir");
	let yaml_path = tmp.path().join("golem.yaml");
	let mut f = File::create(&yaml_path).expect("create yaml");
	writeln!(f, "name: test-app\nversion: 0.1.0").unwrap();

	// Spawn the CLI server from the tmp as cwd
	let port: u16 = 1237;
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

	// resources.list
	let list_body = r#"{"jsonrpc":"2.0","id":"res-list","method":"resources/list","params":{}}"#;
	let list_resp = client
		.post(&base)
		.header("Accept", "application/json")
		.header("Content-Type", "application/json")
		.body(list_body)
		.send()
		.expect("resources.list send");
	assert!(list_resp.status().is_success());
	let list_text = list_resp.text().unwrap_or_default();
	assert!(list_text.contains("golem.yaml"), "list missing golem.yaml: {}", list_text);

	// Extract a file:// URI if present
	let uri_start = list_text.find("file://");
	assert!(uri_start.is_some(), "no file:// URI in list: {}", list_text);

	// resources.read (read first URI found)
	let read_body = format!(
		"{{\"jsonrpc\":\"2.0\",\"id\":\"res-read\",\"method\":\"resources/read\",\"params\":{{\"uri\":\"file://{}\"}}}}",
		yaml_path.to_string_lossy().replace('\\', "\\\\")
	);
	let read_resp = client
		.post(&base)
		.header("Accept", "application/json")
		.header("Content-Type", "application/json")
		.body(read_body)
		.send()
		.expect("resources.read send");
	assert!(read_resp.status().is_success());
	let read_text = read_resp.text().unwrap_or_default();
	assert!(read_text.contains("test-app"), "read content missing: {}", read_text);

	// Cleanup process
	let _ = child.kill();
	let _ = child.wait();
	if let Some(mut out) = child.stdout.take() {
		let mut buf = String::new();
		let _ = out.read_to_string(&mut buf);
	}
	if let Some(mut err) = child.stderr.take() {
		let mut buf = String::new();
		let _ = err.read_to_string(&mut buf);
	}
} 