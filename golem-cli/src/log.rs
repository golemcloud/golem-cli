// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::fs::{OverwriteSafeAction, OverwriteSafeActionPlan, PathExtra};
use camino::{Utf8Path, Utf8PathBuf};
use colored::{ColoredString, Colorize};
use console::strip_ansi_codes;
use rmcp::model::{ProgressNotificationParam, ProgressToken};
use rmcp::{Peer, RoleServer};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock, RwLock};
use terminal_size::terminal_size;
use textwrap::WordSplitter;
use tracing::debug;

static LOG_STATE: LazyLock<RwLock<LogState>> = LazyLock::new(RwLock::default);
static TERMINAL_WIDTH: OnceLock<Option<usize>> = OnceLock::new();
static WRAP_PADDING: usize = 2;
// static PROGRESS_COUNTER: LazyLock<RwLock<u32>> = LazyLock::new(|| RwLock::new(0));
static TOOL_OUTPUT: LazyLock<RwLock<Vec<String>>> = LazyLock::new(RwLock::default);


fn terminal_width() -> Option<usize> {
    *TERMINAL_WIDTH.get_or_init(|| terminal_size().map(|(width, _)| width.0 as usize))
}

#[derive(Debug, Clone)]
pub enum Output {
    Stdout,
    Stderr,
    None,
    TracingDebug,
    Mcp(Mcp),
}

impl Output {
    fn is_mcp(&self) -> bool {
        matches!(self, Output::Mcp(_))
    }
}

#[derive(Debug, Clone)]
pub struct Mcp {
    pub client: Peer<RoleServer>,
    pub tool_name: String,
    // pub progress_token: ProgressToken
}

struct LogState {
    indents: Vec<Option<String>>,
    calculated_indent: String,
    max_width: Option<usize>,
    output: Output,
}

impl LogState {
    pub fn new() -> Self {
        Self {
            indents: Vec::new(),
            calculated_indent: String::new(),
            max_width: terminal_width().map(|w| w - WRAP_PADDING),
            output: Output::Stdout,
        }
    }

    pub fn inc_indent(&mut self, custom_prefix: Option<&str>) {
        self.indents.push(custom_prefix.map(|p| p.to_string()));
        self.regen_indent_prefix();
    }

    pub fn dec_indent(&mut self) {
        self.indents.pop();
        self.regen_indent_prefix()
    }

    fn regen_indent_prefix(&mut self) {
        self.calculated_indent = String::with_capacity(self.indents.len() * 2);
        for indent in &self.indents {
            self.calculated_indent
                .push_str(indent.as_ref().map(|s| s.as_str()).unwrap_or("  "))
        }
        self.max_width = terminal_width().map(|w| w - WRAP_PADDING - self.calculated_indent.len());
    }

    fn set_output(&mut self, output: Output) {
        self.output = output;
    }
}

impl Default for LogState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogIndent;

impl LogIndent {
    pub fn new() -> Self {
        LOG_STATE.write().unwrap().inc_indent(None);
        Self
    }

    pub fn prefix<S: AsRef<str>>(prefix: S) -> Self {
        LOG_STATE.write().unwrap().inc_indent(Some(prefix.as_ref()));
        Self
    }
}

impl Default for LogIndent {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LogIndent {
    fn drop(&mut self) {
        let mut state = LOG_STATE.write().unwrap();
        state.dec_indent();
    }
}

pub struct LogOutput {
    prev_output: Output,
}

impl LogOutput {
    pub fn new(output: Output) -> Self {
        let prev_output = &LOG_STATE.read().unwrap().output.clone();
        LOG_STATE.write().unwrap().set_output(output);
        Self {
            prev_output: prev_output.clone(),
        }
    }
}

impl Drop for LogOutput {
    fn drop(&mut self) {
        LOG_STATE
            .write()
            .unwrap()
            .set_output(self.prev_output.clone());
    }
}

pub fn set_log_output(output: Output) {
    debug!(output=?output, "set log output");
    LOG_STATE.write().unwrap().set_output(output);
}

pub fn get_log_output() -> Output {
    LOG_STATE.read().unwrap().output.clone()
}

pub fn log_action<T: AsRef<str>>(action: &str, subject: T) {
    logln_internal(&format!(
        "{} {}",
        action.log_color_action(),
        subject.as_ref()
    ));
}

pub fn log_warn_action<T: AsRef<str>>(action: &str, subject: T) {
    logln_internal(&format!("{} {}", action.log_color_warn(), subject.as_ref(),));
}

pub fn log_error_action<T: AsRef<str>>(action: &str, subject: T) {
    logln_internal(&format!(
        "{} {}",
        action.log_color_error(),
        subject.as_ref(),
    ));
}

pub fn logln<T: AsRef<str>>(message: T) {
    logln_internal(message.as_ref());
}

pub fn logln_internal(message: &str) {
    // Acquire read lock once
    let state = LOG_STATE.read().unwrap();
    let width = state.max_width;
    let indent = state.calculated_indent.clone();
    let output = state.output.clone();

    drop(state); // Release read lock before logging

    // Wrap lines if needed
    let lines: Vec<Cow<'_, str>> = if !output.is_mcp() {
        process_lines(message, width)
    } else {
        vec![Cow::from(process_lines(message, width).concat())]
    };

    for line in lines {
        match &output {
            Output::Stdout => println!("{}{}", indent, line),
            Output::Stderr => eprintln!("{}{}", indent, line),
            Output::None => {}
            Output::TracingDebug => debug!("{}{}", indent, line),
            Output::Mcp(_mcp) => {
                store_mcp_tool_output(&indent, line)
                // notify_log_to_mcp_client(&indent, line, mcp);
            }
        }
    }
}

fn process_lines(message: &str, width: Option<usize>) -> Vec<Cow<'_, str>> {
    match width {
        Some(w) if w > 0 && message.len() > w && !message.contains('\n') => {
            textwrap::wrap(
                message,
                textwrap::Options::new(w)
                    // deliberately 5 spaces, to makes this indent different from normal ones
                    .subsequent_indent("     ")
                    .word_splitter(WordSplitter::NoHyphenation),
            )
        }
        _ => vec![Cow::from(message)],
    }
}

// fn notify_log_to_mcp_client(
//     indent: &str,
//     line: Cow<'_, str>,
//     mcp: &Mcp,
// ) {
//     let data: String = format!("{}{}", indent, line).clone();
//     let progress = *PROGRESS_COUNTER.read().unwrap() as u32;
//     tokio::spawn({
//         let client = mcp.client.clone();
//         let progress_token = mcp.progress_token.clone();
//         let progress = progress;
//         async move {
//             let plain_data = strip_ansi_codes(&data).into_owned();

//             let _ = client.notify_progress(ProgressNotificationParam {
//                     progress_token,
//                     progress,
//                     message: plain_data.into(),
//                     total: None,
//                 })
//                 .await;
//             *PROGRESS_COUNTER.write().unwrap() = progress + 1;
//         }
//     });
// }

pub fn store_mcp_tool_output(
    indent: &str,
    line: Cow<'_, str>,
) {
    let data: String = format!("{}{}", indent, line).clone();
    (*TOOL_OUTPUT.write().unwrap()).push(data)
}

pub fn get_mcp_tool_output() -> Vec<String> {
    let output = TOOL_OUTPUT.read().unwrap().to_vec().join("\n");
    (*TOOL_OUTPUT.write().unwrap()).clear();
    
    vec![output]
}

pub fn log_skipping_up_to_date<T: AsRef<str>>(subject: T) {
    log_warn_action(
        "Skipping",
        format!(
            "{}, {}",
            subject.as_ref(),
            "UP-TO-DATE".log_color_ok_highlight()
        ),
    );
}

pub fn log_action_plan(action: &OverwriteSafeAction, plan: OverwriteSafeActionPlan) {
    match plan {
        OverwriteSafeActionPlan::Create => match action {
            OverwriteSafeAction::CopyFile { source, target } => {
                log_action(
                    "Copying",
                    format!(
                        "{} to {}",
                        source.log_color_highlight(),
                        target.log_color_highlight()
                    ),
                );
            }
            OverwriteSafeAction::CopyFileTransformed { source, target, .. } => {
                log_action(
                    "Copying",
                    format!(
                        "{} to {} transformed",
                        source.log_color_highlight(),
                        target.log_color_highlight()
                    ),
                );
            }
            OverwriteSafeAction::WriteFile { target, .. } => {
                log_action("Creating", format!("{}", target.log_color_highlight()));
            }
        },
        OverwriteSafeActionPlan::Overwrite => match action {
            OverwriteSafeAction::CopyFile { source, target } => {
                log_warn_action(
                    "Overwriting",
                    format!(
                        "{} with {}",
                        target.log_color_highlight(),
                        source.log_color_highlight()
                    ),
                );
            }
            OverwriteSafeAction::CopyFileTransformed { source, target, .. } => {
                log_warn_action(
                    "Overwriting",
                    format!(
                        "{} with {} transformed",
                        target.log_color_highlight(),
                        source.log_color_highlight()
                    ),
                );
            }
            OverwriteSafeAction::WriteFile { content: _, target } => {
                log_warn_action("Overwriting", format!("{}", target.log_color_highlight()));
            }
        },
        OverwriteSafeActionPlan::SkipSameContent => match action {
            OverwriteSafeAction::CopyFile { source, target } => {
                log_warn_action(
                    "Skipping",
                    format!(
                        "copying {} to {}, content already up-to-date",
                        source.log_color_highlight(),
                        target.log_color_highlight(),
                    ),
                );
            }
            OverwriteSafeAction::CopyFileTransformed { source, target, .. } => {
                log_warn_action(
                    "Skipping",
                    format!(
                        "copying {} to {} transformed, content already up-to-date",
                        source.log_color_highlight(),
                        target.log_color_highlight()
                    ),
                );
            }
            OverwriteSafeAction::WriteFile { content: _, target } => {
                log_warn_action(
                    "Skipping",
                    format!(
                        "generating {}, content already up-to-date",
                        target.log_color_highlight()
                    ),
                );
            }
        },
    }
}

pub trait LogColorize {
    fn as_str(&self) -> impl Colorize;

    fn log_color_action(&self) -> ColoredString {
        self.as_str().green()
    }

    fn log_color_warn(&self) -> ColoredString {
        self.as_str().yellow().bold()
    }

    fn log_color_error(&self) -> ColoredString {
        self.as_str().red().bold()
    }

    fn log_color_highlight(&self) -> ColoredString {
        self.as_str().bold()
    }

    fn log_color_help_group(&self) -> ColoredString {
        self.as_str().bold().underline()
    }

    fn log_color_error_highlight(&self) -> ColoredString {
        self.as_str().bold().red().underline()
    }

    fn log_color_ok_highlight(&self) -> ColoredString {
        self.as_str().bold().green()
    }
}

impl LogColorize for &str {
    fn as_str(&self) -> impl Colorize {
        *self
    }
}

impl LogColorize for String {
    fn as_str(&self) -> impl Colorize {
        self.as_str()
    }
}

impl LogColorize for &Path {
    fn as_str(&self) -> impl Colorize {
        ColoredString::from(self.display().to_string())
    }
}

impl LogColorize for &Utf8Path {
    fn as_str(&self) -> impl Colorize {
        ColoredString::from(self.to_string())
    }
}

impl LogColorize for PathBuf {
    fn as_str(&self) -> impl Colorize {
        ColoredString::from(self.display().to_string())
    }
}

impl LogColorize for Utf8PathBuf {
    fn as_str(&self) -> impl Colorize {
        ColoredString::from(self.to_string())
    }
}

impl<P: AsRef<Path>> LogColorize for PathExtra<P> {
    fn as_str(&self) -> impl Colorize {
        ColoredString::from(self.display().to_string())
    }
}
