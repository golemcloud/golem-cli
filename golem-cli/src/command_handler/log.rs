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

use crate::command_handler::CommandHandler;
use crate::model::text::fmt::TextView;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Log {
    fn base(&self) -> &CommandHandler;
    fn base_mut(&mut self) -> &mut CommandHandler;

    fn log_view<View: TextView + Serialize + DeserializeOwned>(&self, view: &View) {
        // TODO: handle formats
        view.log();
    }
}

impl Log for CommandHandler {
    fn base(&self) -> &CommandHandler {
        self
    }

    fn base_mut(&mut self) -> &mut CommandHandler {
        self
    }
}
