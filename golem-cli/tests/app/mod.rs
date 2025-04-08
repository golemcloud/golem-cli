// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::test_r_get_dep_tracing;
use crate::Tracing;
use assert2::{assert, check};
use colored::Colorize;
use golem_cli::fs;
use golem_templates::model::GuestLanguage;
use indoc::indoc;
use itertools::Itertools;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use strum::IntoEnumIterator;
use tempfile::TempDir;
use test_r::test;
use tracing::info;
use golem_cli::model::invoke_result_view::InvokeResultView;

mod cmd {
    pub static APP: &str = "app";
    pub static BUILD: &str = "build";
    pub static COMPLETION: &str = "completion";
    pub static COMPONENT: &str = "component";
    pub static DEPLOY: &str = "deploy";
    pub static NEW: &str = "new";
    pub static WORKER: &str = "worker";
    pub static INVOKE: &str = "invoke";
}

mod flag {
    pub static FORCE_BUILD: &str = "--force-build";
}

mod pattern {
    pub static ERROR: &str = "error";
    pub static HELP_APPLICATION_COMPONENTS: &str = "Application components:";
    pub static HELP_APPLICATION_CUSTOM_COMMANDS: &str = "Application custom commands:";
    pub static HELP_COMMANDS: &str = "Commands:";
    pub static HELP_USAGE: &str = "Usage:";
}

#[test]
fn app_help_in_empty_folder(_tracing: &Tracing) {
    let ctx = TestContext::new();
    let outputs = ctx.cli([cmd::APP]);
    assert!(!outputs.success());
    check!(outputs.stderr_contains(pattern::HELP_USAGE));
    check!(outputs.stderr_contains(pattern::HELP_COMMANDS));
    check!(!outputs.stderr_contains(pattern::ERROR));
    check!(!outputs.stderr_contains(pattern::HELP_APPLICATION_COMPONENTS));
    check!(!outputs.stderr_contains(pattern::HELP_APPLICATION_CUSTOM_COMMANDS));
}

#[test]
fn app_new_with_many_components_and_then_help_in_app_folder(_tracing: &Tracing) {
    let app_name = "test-app-name";

    let mut ctx = TestContext::new();
    let outputs = ctx.cli([
        cmd::APP,
        cmd::NEW,
        app_name,
        "c",
        "go",
        "typescript",
        "rust",
    ]);
    assert!(outputs.success());

    ctx.cd(app_name);

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "c", "app:c"]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "go", "app:go"]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "typescript", "app:typescript"]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "rust", "app:rust"]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::APP]);
    assert!(!outputs.success());
    check!(outputs.stderr_contains(pattern::HELP_USAGE));
    check!(outputs.stderr_contains(pattern::HELP_COMMANDS));
    check!(!outputs.stderr_contains(pattern::ERROR));
    check!(outputs.stderr_contains(pattern::HELP_APPLICATION_COMPONENTS));
    check!(outputs.stderr_contains("app:c"));
    check!(outputs.stderr_contains("app:go"));
    check!(outputs.stderr_contains("app:rust"));
    check!(outputs.stderr_contains("app:typescript"));
    check!(outputs.stderr_contains(pattern::HELP_APPLICATION_CUSTOM_COMMANDS));
    check!(outputs.stderr_contains("cargo-clean"));
    check!(outputs.stderr_contains("npm-install"));
}

#[test]
fn app_build_with_rust_component(_tracing: &Tracing) {
    let app_name = "test-app-name";

    let mut ctx = TestContext::new();
    let outputs = ctx.cli([cmd::APP, cmd::NEW, app_name, "rust"]);
    assert!(outputs.success());

    ctx.cd(app_name);

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "rust", "app:rust"]);
    assert!(outputs.success());

    // First build
    let outputs = ctx.cli([cmd::APP, cmd::BUILD]);
    assert!(outputs.success());
    check!(outputs.stdout_contains("Executing external command 'cargo component build'"));
    check!(outputs.stderr_contains("Compiling app_rust v0.0.1"));

    check_component_metadata(
        &ctx.working_dir
            .join("golem-temp/components/app_rust_debug.wasm"),
        "app:rust".to_string(),
        None,
    );

    // Rebuild - 1
    let outputs = ctx.cli([cmd::APP, cmd::BUILD]);
    assert!(outputs.success());
    check!(!outputs.stdout_contains("Executing external command 'cargo component build'"));
    check!(!outputs.stderr_contains("Compiling app_rust v0.0.1"));

    // Rebuild - 2
    let outputs = ctx.cli([cmd::APP, cmd::BUILD]);
    assert!(outputs.success());
    check!(!outputs.stdout_contains("Executing external command 'cargo component build'"));
    check!(!outputs.stderr_contains("Compiling app_rust v0.0.1"));

    // Rebuild - 3 - force, but cargo is smart to skip actual compile
    let outputs = ctx.cli([cmd::APP, cmd::BUILD, flag::FORCE_BUILD]);
    assert!(outputs.success());
    check!(outputs.stdout_contains("Executing external command 'cargo component build'"));
    check!(outputs.stderr_contains("Finished `dev` profile"));

    // Rebuild - 4
    let outputs = ctx.cli([cmd::APP, cmd::BUILD]);
    assert!(outputs.success());
    check!(!outputs.stdout_contains("Executing external command 'cargo component build'"));
    check!(!outputs.stderr_contains("Compiling app_rust v0.0.1"));

    // Clean
    let outputs = ctx.cli([cmd::APP, cmd::BUILD]);
    assert!(outputs.success());

    // Rebuild - 5
    let outputs = ctx.cli([cmd::APP, cmd::BUILD]);
    assert!(outputs.success());
    check!(!outputs.stdout_contains("Executing external command 'cargo component build'"));
    check!(!outputs.stderr_contains("Compiling app_rust v0.0.1"));
}

#[test]
fn app_new_language_hints(_tracing: &Tracing) {
    let ctx = TestContext::new();
    let outputs = ctx.cli([cmd::APP, cmd::NEW, "dummy-app-name"]);
    assert!(!outputs.success());
    check!(outputs.stderr_contains("Available languages:"));

    let languages_without_templates = GuestLanguage::iter()
        .filter(|language| !outputs.stderr_contains(format!("- {}", language)))
        .collect::<Vec<_>>();

    assert!(
        languages_without_templates.is_empty(),
        "{:?}",
        languages_without_templates
    );
}

#[test]
fn completion(_tracing: &Tracing) {
    let ctx = TestContext::new();

    let outputs = ctx.cli([cmd::COMPLETION, "bash"]);
    assert!(outputs.success(), "bash");

    let outputs = ctx.cli([cmd::COMPLETION, "elvish"]);
    assert!(outputs.success(), "elvish");

    let outputs = ctx.cli([cmd::COMPLETION, "fish"]);
    assert!(outputs.success(), "fish");

    let outputs = ctx.cli([cmd::COMPLETION, "powershell"]);
    assert!(outputs.success(), "powershell");

    let outputs = ctx.cli([cmd::COMPLETION, "zsh"]);
    assert!(outputs.success(), "zsh");
}

#[test]
fn basic_dependencies_build(_tracing: &Tracing) {
    let mut ctx = TestContext::new();
    let app_name = "test-app-name";

    let outputs = ctx.cli([cmd::APP, cmd::NEW, app_name, "rust", "ts"]);
    assert!(outputs.success());

    ctx.cd(app_name);

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "rust", "app:rust"]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "ts", "app:ts"]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::APP, "ts-npm-install"]);
    assert!(outputs.success());

    fs::append_str(
        ctx.cwd_path_join(
            Path::new("components-rust")
                .join("app-rust")
                .join("golem.yaml"),
        ),
        indoc! {"
            dependencies:
              app:rust:
              - target: app:rust
                type: wasm-rpc
              - target: app:ts
                type: wasm-rpc
        "},
    )
    .unwrap();

    fs::append_str(
        ctx.cwd_path_join(Path::new("components-ts").join("app-ts").join("golem.yaml")),
        indoc! {"
            dependencies:
              app:ts:
              - target: app:rust
                type: wasm-rpc
              - target: app:ts
                type: wasm-rpc
        "},
    )
    .unwrap();

    let outputs = ctx.cli([cmd::APP]);
    assert!(!outputs.success());
    check!(outputs.stderr_count_lines_containing("- app:rust (wasm-rpc)") == 2);
    check!(outputs.stderr_count_lines_containing("- app:ts (wasm-rpc)") == 2);

    let outputs = ctx.cli([cmd::APP, cmd::BUILD]);
    assert!(outputs.success());
}

#[test]
fn basic_ifs_deploy(_tracing: &Tracing) {
    let mut ctx = TestContext::new();
    let app_name = "test-app-name";

    let outputs = ctx.cli([cmd::APP, cmd::NEW, app_name, "rust"]);
    assert!(outputs.success());

    ctx.cd(app_name);

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "rust", "app:rust"]);
    assert!(outputs.success());

    fs::write_str(
        ctx.cwd_path_join(
            Path::new("components-rust")
                .join("app-rust")
                .join("golem.yaml"),
        ),
        indoc! {"
            components:
              app:rust:
                template: rust
                profiles:
                  debug:
                    files:
                    - sourcePath: Cargo.toml
                      targetPath: /Cargo.toml
                      permissions: read-only
                    - sourcePath: src/lib.rs
                      targetPath: /src/lib.rs
                      permissions: read-write

        "},
    )
    .unwrap();

    ctx.start_server();

    let outputs = ctx.cli([cmd::APP, cmd::DEPLOY]);
    assert!(outputs.success());
    check!(outputs.stdout_contains("ro /Cargo.toml"));
    check!(outputs.stdout_contains("rw /src/lib.rs"));

    fs::write_str(
        ctx.cwd_path_join(
            Path::new("components-rust")
                .join("app-rust")
                .join("golem.yaml"),
        ),
        indoc! {"
            components:
              app:rust:
                template: rust
                profiles:
                  debug:
                    files:
                    - sourcePath: Cargo.toml
                      targetPath: /Cargo2.toml
                      permissions: read-only
                    - sourcePath: src/lib.rs
                      targetPath: /src/lib.rs
                      permissions: read-only

        "},
    )
    .unwrap();

    let outputs = ctx.cli([cmd::APP, cmd::DEPLOY]);
    assert!(outputs.success());
    check!(!outputs.stdout_contains("ro /Cargo.toml"));
    check!(outputs.stdout_contains("ro /Cargo2.toml"));
    check!(!outputs.stdout_contains("rw /src/lib.rs"));
    check!(outputs.stdout_contains("ro /src/lib.rs"));
}

#[test]
fn custom_app_subcommand_with_builtin_name() {
    let mut ctx = TestContext::new();
    let app_name = "test-app-name";

    let outputs = ctx.cli([cmd::APP, cmd::NEW, app_name, "rust"]);
    assert!(outputs.success());

    ctx.cd(app_name);

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "rust", "app:rust"]);
    assert!(outputs.success());

    fs::append_str(
        ctx.cwd_path_join("golem.yaml"),
        indoc! {"
            customCommands:
              new:
                - command: cargo tree
        "},
    )
    .unwrap();

    let outputs = ctx.cli([cmd::APP]);
    assert!(!outputs.success());
    check!(outputs.stderr_contains(":new"));

    let outputs = ctx.cli([cmd::APP, ":new"]);
    assert!(outputs.success());
    check!(outputs.stdout_contains("Executing external command 'cargo tree'"));
}

#[test]
fn wasm_library_dependency_type() -> anyhow::Result<()> {
    let mut ctx = TestContext::new();
    let app_name = "test-app-name";

    let outputs = ctx.cli([cmd::APP, cmd::NEW, app_name, "rust"]);
    assert!(outputs.success());

    ctx.cd(app_name);

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "rust", "app:main"]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::COMPONENT, cmd::NEW, "rust", "app:lib"]);
    assert!(outputs.success());

    // Changing the `app:lib` component type to be a library
    fs::write_str(
        ctx.cwd_path_join(
            Path::new("components-rust")
                .join("app-lib")
                .join("golem.yaml"),
        ),
        indoc! {"
            components:
              app:lib:
                template: rust
                profiles:
                  debug:
                    componentType: library
                  release:
                    componentType: library
        "},
    )?;

    // Adding as a wasm dependency

    fs::append_str(
        ctx.cwd_path_join(
            Path::new("components-rust")
                .join("app-main")
                .join("golem.yaml"),
        ),
        indoc! {"
            dependencies:
              app:main:
                - type: wasm
                  target: app:lib
        "},
    )?;

    // Rewriting the main WIT

    fs::write_str(
        ctx.cwd_path_join(
            Path::new("components-rust").join("app-main").join("wit").join("app-main.wit")
        ),
        indoc! {"
            package app:main;

            interface app-main-api {
                run: func() -> u64;
            }

            world app-main {
                export app-main-api;
                import app:lib-exports/app-lib-api;
            }
        "},
    )?;

    // Rewriting the main Rust source code

    fs::write_str(
        ctx.cwd_path_join(
            Path::new("components-rust").join("app-main").join("src").join("lib.rs")
        ),
        indoc! {"
                #[allow(static_mut_refs)]
                mod bindings;

                use bindings::app::lib_exports::app_lib_api;
                use crate::bindings::exports::app::main_exports::app_main_api::*;

                struct Component;

                impl Guest for Component {
                    fn run() -> u64 {
                        app_lib_api::add(1);
                        app_lib_api::add(2);
                        app_lib_api::add(3);
                        app_lib_api::get()
                    }
                }

                bindings::export!(Component with_types_in bindings);
         "}
    )?;

    ctx.start_server();

    let outputs = ctx.cli([cmd::APP, cmd::DEPLOY]);
    assert!(outputs.success());

    let outputs = ctx.cli([cmd::WORKER, cmd::INVOKE, "app:main/test1", "run", "--format", "json"]);
    assert!(outputs.success());

    let result: InvokeResultView = serde_json::from_str(&outputs.stdout[0])?;
    assert_eq!(result.result_wave, Some(vec!["6".to_string()]));

    Ok(())
}

pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl Output {
    fn success(&self) -> bool {
        self.status.success()
    }

    fn stdout_contains<S: AsRef<str>>(&self, text: S) -> bool {
        self.stdout.iter().any(|line| line.contains(text.as_ref()))
    }

    fn stderr_contains<S: AsRef<str>>(&self, text: S) -> bool {
        self.stderr.iter().any(|line| line.contains(text.as_ref()))
    }

    fn stderr_count_lines_containing<S: AsRef<str>>(&self, text: S) -> usize {
        self.stderr
            .iter()
            .filter(|line| line.contains(text.as_ref()))
            .count()
    }
}

impl From<std::process::Output> for Output {
    fn from(output: std::process::Output) -> Self {
        fn to_lines(bytes: Vec<u8>) -> Vec<String> {
            String::from_utf8(bytes)
                .unwrap()
                .lines()
                .map(|s| s.to_string())
                .collect()
        }

        Self {
            status: output.status,
            stdout: to_lines(output.stdout),
            stderr: to_lines(output.stderr),
        }
    }
}

#[derive(Debug)]
struct TestContext {
    golem_path: PathBuf,
    golem_cli_path: PathBuf,
    _test_dir: TempDir,
    config_dir: TempDir,
    data_dir: TempDir,
    working_dir: PathBuf,
    server_process: Option<Child>,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.stop_server();
    }
}

impl TestContext {
    fn new() -> Self {
        let test_dir = TempDir::new().unwrap();
        let working_dir = test_dir.path().to_path_buf();

        let ctx = Self {
            golem_path: PathBuf::from("../target/debug/golem")
                .canonicalize()
                .unwrap(),
            golem_cli_path: PathBuf::from("../target/debug/golem-cli")
                .canonicalize()
                .unwrap(),
            _test_dir: test_dir,
            config_dir: TempDir::new().unwrap(),
            data_dir: TempDir::new().unwrap(),
            working_dir,
            server_process: None,
        };

        info!(ctx = ?ctx ,"Created test context");

        ctx
    }

    fn cli<I, S>(&self, args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = args.into_iter().collect::<Vec<_>>();
        let working_dir = &self.working_dir.canonicalize().unwrap();

        println!(
            "{} {}",
            "> working directory:".bold(),
            working_dir.display()
        );
        println!(
            "{} {}",
            "> golem-cli".bold(),
            args.iter()
                .map(|s| s.as_ref().to_string_lossy())
                .join(" ")
                .blue()
        );

        let output: Output = Command::new(&self.golem_cli_path)
            .args(args)
            .current_dir(working_dir)
            .output()
            .unwrap()
            .into();

        let status_prefix = {
            let status_prefix = "> status:".bold();
            if output.success() {
                status_prefix.green()
            } else {
                status_prefix.red()
            }
        };
        println!("{} {}", status_prefix, output.status);
        let stdout_prefix = "> stdout:".green().bold();
        for line in &output.stdout {
            println!("{} {}", stdout_prefix, line);
        }
        let stderr_prefix = "> stderr:".red().bold();
        for line in &output.stderr {
            println!("{} {}", stderr_prefix, line);
        }

        output
    }

    fn start_server(&mut self) {
        assert!(self.server_process.is_none(), "server is already running");

        println!("{}", "> starting golem server".bold());
        println!(
            "{} {}",
            "> server config directory:".bold(),
            self.config_dir.path().display()
        );
        println!(
            "{} {}",
            "> server data directory:".bold(),
            self.data_dir.path().display()
        );

        self.server_process = Some(
            Command::new(&self.golem_path)
                .args([
                    "server",
                    "run",
                    "--config-dir",
                    self.config_dir.path().to_str().unwrap(),
                    "--data-dir",
                    self.data_dir.path().to_str().unwrap(),
                ])
                .current_dir(&self.working_dir)
                .spawn()
                .unwrap(),
        )
    }

    fn stop_server(&mut self) {
        let server_process = self.server_process.take();
        if let Some(mut server_process) = server_process {
            println!("{}", "> stopping golem server".bold());
            server_process.kill().unwrap();
        }
    }

    fn cd<P: AsRef<Path>>(&mut self, path: P) {
        self.working_dir = self.working_dir.join(path.as_ref());
    }

    fn cwd_path_join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.working_dir.join(path)
    }
}

fn check_component_metadata(
    wasm: &Path,
    expected_package_name: String,
    expected_version: Option<String>,
) {
    let wasm = std::fs::read(wasm).unwrap();
    let payload = wasm_metadata::Payload::from_binary(&wasm).unwrap();
    let metadata = payload.metadata();

    assert_eq!(metadata.name, Some(expected_package_name));
    assert_eq!(
        metadata.version.as_ref().map(|v| v.to_string()),
        expected_version
    );
}
