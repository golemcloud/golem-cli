use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::{env, path::PathBuf, thread};

#[tokio::test]
async fn mcp_serve_starts_and_accepts_sse() {
	let bin = locate_cli_binary().unwrap_or_else(|| {
		println!("skipping: golem-cli binary not found");
		PathBuf::new()
	});
	if bin.as_os_str().is_empty() {
		return;
	}

	let port = 18232u16;

	let mut child = Command::new(&bin)
		.arg("--serve")
		.arg("--serve-port")
		.arg(port.to_string())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.expect("failed to spawn golem-cli");

	let start = Instant::now();
	let client = reqwest::Client::builder().timeout(Duration::from_millis(800)).build().unwrap();
	let url = format!("http://127.0.0.1:{}/sse", port);
	let mut ok = false;
	while start.elapsed() < Duration::from_secs(5) {
		if let Ok(resp) = client.get(&url).header("Accept", "text/event-stream").send().await {
			if resp.status().is_success() {
				ok = true;
				break;
			}
		}
		thread::sleep(Duration::from_millis(100));
	}

	let _ = child.kill();

	assert!(ok, "SSE endpoint did not respond with success at {}", url);
}

fn locate_cli_binary() -> Option<PathBuf> {
	if let Ok(p) = env::var("CARGO_BIN_EXE_golem-cli") {
		let path = PathBuf::from(p);
		if path.exists() {
			return Some(path);
		}
	}
	let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	p.pop();
	p.push("target");
	p.push("debug");
	p.push("golem-cli");
	if p.exists() {
		return Some(p);
	}
	None
} 