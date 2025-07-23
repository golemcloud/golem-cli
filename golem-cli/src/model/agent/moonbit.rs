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

use crate::model::agent::wit::AgentWrapperGeneratorContext;
use anyhow::{anyhow, Context};
use camino::Utf8Path;
use moonbit_component_generator::MoonBitComponent;
use std::path::Path;

pub fn generate_moonbit_wrapper(
    ctx: AgentWrapperGeneratorContext,
    target: &Path,
) -> anyhow::Result<()> {
    let wit = ""; // TODO
    let mut component = MoonBitComponent::empty_from_wit(wit, Some("agent-wrapper"))?;

    component
        .define_bindgen_packages()
        .context("Defining bindgen packages")?;

    let mut stub_mbt = String::new();
    // TODO

    component
        .write_world_stub(&stub_mbt)
        .context("Writing world stub")?;

    // TODO: implement the interface stubs

    component
        .build(
            None,
            Utf8Path::from_path(target).ok_or_else(|| {
                anyhow!("Invalid target path for the agent wrapper component: {path:?}")
            })?,
        )
        .context("Building component")?;

    Ok(())
}
