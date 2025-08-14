use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::{env, fs, path::PathBuf, thread};
use serde_json::json;

// Ignored for now: streamable HTTP is optional; SSE is the supported transport.
// Enable and un-ignore when streamable HTTP endpoint is ready.
#[ignore]
#[tokio::test]
async fn http_tools_list_and_resources_roundtrip() {
	let bin = locate_cli_binary().unwrap_or_else(|| {
		println!("skipping: golem-cli binary not found");
		PathBuf::new()
	});
	if bin.as_os_str().is_empty() {
		return;
	}

	let temp = tempfile::tempdir().expect("tempdir");
	fs::write(temp.path().join("golem.yaml"), "name: test\nversion: 0.0.0\n").unwrap();

	let port = 18233u16;

	let mut child = Command::new(&bin)
		.arg("--serve")
		.arg("--serve-port")
		.arg(port.to_string())
		.current_dir(temp.path())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.expect("failed to spawn golem-cli");

	let start = Instant::now();
	let client = reqwest::Client::builder().timeout(Duration::from_millis(1500)).build().unwrap();
	let url = format!("http://127.0.0.1:{}/mcp", port);

	// Wait for server readiness by probing tools/list
	let mut ready = false;
	while start.elapsed() < Duration::from_secs(10) {
		let list_tools = json!({"id": 1, "jsonrpc": "2.0", "method": "tools/list", "params": {}});
		if let Ok(resp) = client.post(&url).header("Accept","application/json").json(&list_tools).send().await {
			if resp.status().is_success() { ready = true; break; }
		}
		thread::sleep(Duration::from_millis(200));
	}
	assert!(ready, "server was not ready at {}", url);

	// tools/list
	let list_tools = json!({"id": 2, "jsonrpc": "2.0", "method": "tools/list", "params": {}});
	let resp = client.post(&url).header("Accept","application/json").json(&list_tools).send().await.unwrap();
	assert!(resp.status().is_success());
	let v: serde_json::Value = resp.json().await.unwrap();
	let tools = v.pointer("/result/tools").and_then(|x| x.as_array()).cloned().unwrap_or_default();
	assert!(!tools.is_empty(), "expected at least one tool: {:?}", v);

	// resources/list
	let list_res = json!({"id": 3, "jsonrpc": "2.0", "method": "resources/list", "params": {}});
	let resp = client.post(&url).header("Accept","application/json").json(&list_res).send().await.unwrap();
	assert!(resp.status().is_success());
	let v: serde_json::Value = resp.json().await.unwrap();
	let resources = v.pointer("/result/resources").and_then(|x| x.as_array()).cloned().unwrap_or_default();
	assert!(!resources.is_empty(), "expected at least one resource in {:?}", v);
	let uri = resources[0].get("uri").and_then(|u| u.as_str()).unwrap_or("").to_string();
	assert!(uri.starts_with("file://"));

	// resources/read
	let read_res = json!({"id": 4, "jsonrpc": "2.0", "method": "resources/read", "params": {"uri": uri}});
	let resp = client.post(&url).header("Accept","application/json").json(&read_res).send().await.unwrap();
	assert!(resp.status().is_success());
	let v: serde_json::Value = resp.json().await.unwrap();
	let contents = v.pointer("/result/contents").and_then(|x| x.as_array()).cloned().unwrap_or_default();
	assert!(!contents.is_empty(), "expected contents in {:?}", v);

	let _ = child.kill();
}

fn locate_cli_binary() -> Option<PathBuf> {
	if let Ok(p) = env::var("CARGO_BIN_EXE_golem-cli") {
		let path = PathBuf::from(p);
		if path.exists() { return Some(path); }
	}
	let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	p.pop();
	p.push("target");
	p.push("debug");
	p.push("golem-cli");
	if p.exists() { return Some(p); }
	None
} 