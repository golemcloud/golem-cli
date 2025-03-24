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

use shadow_rs::{BuildPattern, SdResult, ShadowBuilder, ShadowError};
use std::fs::File;
use std::io::Write;
use std::process::Command;

fn main() {
    ShadowBuilder::builder()
        .hook(hook)
        .build_pattern(BuildPattern::Lazy)
        .build()
        .unwrap();
}

fn hook(file: &File) -> SdResult<()> {
    append_write_git_describe_tags(file)?;
    Ok(())
}

fn append_write_git_describe_tags(mut file: &File) -> SdResult<()> {
    let output = Command::new("git").args(["describe", "--tags"]).output()?;
    if !output.status.success() {
        println!("cargo::error=git describe --tags failed:");
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            println!("cargo::error=stdout: {}", line);
        }
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            println!("cargo::error=stderr: {}", line);
        }

        return Err(ShadowError::from("git describe --tags failed"));
    }
    let version = String::from_utf8(output.stdout)?;
    println!("cargo::warning=git describe --tags: {}", version);

    let git_describe_tags = format!(
        r#"#[allow(clippy::all, clippy::pedantic, clippy::restriction, clippy::nursery)]
pub const GIT_DESCRIBE_TAGS: &str = "{}";"#,
        version
    );
    writeln!(file, "{git_describe_tags}")?;

    Ok(())
}
