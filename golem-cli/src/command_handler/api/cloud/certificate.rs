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

use crate::command::api::cloud::certificate::ApiCloudCertificateSubcommand;
use crate::context::Context;
use std::sync::Arc;

pub struct ApiCloudCertificateCommandHandler {
    ctx: Arc<Context>,
}

impl ApiCloudCertificateCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handle_command(
        &self,
        command: ApiCloudCertificateSubcommand,
    ) -> anyhow::Result<()> {
        match command {}
    }
}
