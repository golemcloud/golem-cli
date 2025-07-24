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
use moonbit_component_generator::{MoonBitComponent, MoonBitPackage};
use std::path::Path;
use wit_parser::PackageName;

pub fn generate_moonbit_wrapper(
    ctx: AgentWrapperGeneratorContext,
    target: &Path,
) -> anyhow::Result<()> {
    let wit = &ctx.single_file_wrapper_wit_source;
    let mut component = MoonBitComponent::empty_from_wit(wit, Some("agent-wrapper"))?;
    component.disable_cleanup(); // TODO: just for testing

    component
        .define_bindgen_packages()
        .context("Defining bindgen packages")?;

    let moonbit_root_package = component.moonbit_root_package()?;
    let pkg_namespace = component.root_pkg_namespace()?;
    let pkg_name = component.root_pkg_name()?;

    // Adding the builder and extractor packages
    const BUILDER_CHILD_ITEMS_BUILDER_MBT: &str =
        include_str!("../../../agent_wrapper/builder/child_items_builder.mbt");
    const BUILDER_ITEM_BUILDER_MBT: &str =
        include_str!("../../../agent_wrapper/builder/item_builder.mbt");
    const BUILDER_TESTS_MBT: &str = include_str!("../../../agent_wrapper/builder/tests.mbt");
    const BUILDER_TOP_MBT: &str = include_str!("../../../agent_wrapper/builder/top.mbt");

    component.write_file(
        Utf8Path::new("builder/child_items_builder.mbt"),
        BUILDER_CHILD_ITEMS_BUILDER_MBT,
    )?;
    component.write_file(
        Utf8Path::new("builder/item_builder.mbt"),
        BUILDER_ITEM_BUILDER_MBT,
    )?;
    component.write_file(Utf8Path::new("builder/tests.mbt"), BUILDER_TESTS_MBT)?;
    component.write_file(Utf8Path::new("builder/top.mbt"), BUILDER_TOP_MBT)?;

    let builder_package = MoonBitPackage {
        name: format!("{moonbit_root_package}/builder"),
        mbt_files: vec![
            Utf8Path::new("builder").join("child_items_builder.mbt"),
            Utf8Path::new("builder").join("item_builder.mbt"),
            Utf8Path::new("builder").join("tests.mbt"),
            Utf8Path::new("builder").join("top.mbt"),
        ],
        warning_control: vec![],
        output: Utf8Path::new("target")
            .join("wasm")
            .join("release")
            .join("build")
            .join("builder")
            .join("builder.core"),
        dependencies: vec![(
            Utf8Path::new("target")
                .join("wasm")
                .join("release")
                .join("build")
                .join("interface")
                .join("golem")
                .join("rpc")
                .join("types")
                .join("types.mi"),
            "types".to_string(),
        )],
        package_sources: vec![(
            format!("{moonbit_root_package}/builder"),
            Utf8Path::new("builder").to_path_buf(),
        )],
    };
    component.define_package(builder_package);

    const EXTRACTOR_TESTS_MBT: &str = include_str!("../../../agent_wrapper/extractor/tests.mbt");
    const EXTRACTOR_TOP_MBT: &str = include_str!("../../../agent_wrapper/extractor/top.mbt");

    component.write_file(Utf8Path::new("extractor/tests.mbt"), EXTRACTOR_TESTS_MBT)?;
    component.write_file(Utf8Path::new("extractor/top.mbt"), EXTRACTOR_TOP_MBT)?;

    let extractor_package = MoonBitPackage {
        name: format!("{moonbit_root_package}/extractor"),
        mbt_files: vec![
            Utf8Path::new("extractor").join("tests.mbt"),
            Utf8Path::new("extractor").join("top.mbt"),
        ],
        warning_control: vec![],
        output: Utf8Path::new("target")
            .join("wasm")
            .join("release")
            .join("build")
            .join("extractor")
            .join("extractor.core"),
        dependencies: vec![
            (
                Utf8Path::new("target")
                    .join("wasm")
                    .join("release")
                    .join("build")
                    .join("interface")
                    .join("golem")
                    .join("rpc")
                    .join("types")
                    .join("types.mi"),
                "types".to_string(),
            ),
            (
                Utf8Path::new("target")
                    .join("wasm")
                    .join("release")
                    .join("build")
                    .join("interface")
                    .join("golem")
                    .join("agent")
                    .join("common")
                    .join("common.mi"),
                "common".to_string(),
            ),
            (
                Utf8Path::new("target")
                    .join("wasm")
                    .join("release")
                    .join("build")
                    .join("builder")
                    .join("builder.mi"),
                "builder".to_string(),
            ),
        ],
        package_sources: vec![(
            format!("{moonbit_root_package}/extractor"),
            Utf8Path::new("extractor").to_path_buf(),
        )],
    };
    component.define_package(extractor_package);

    // NOTE: setting up additional dependencies. this could be automatically done by a better implementation of define_bindgen_packages

    let depends_on_golem_agent_common = |component: &mut MoonBitComponent, name: &str| {
        component.add_dependency(
            &format!("{moonbit_root_package}/{name}"),
            &Utf8Path::new("target")
                .join("wasm")
                .join("release")
                .join("build")
                .join("interface")
                .join("golem")
                .join("agent")
                .join("common")
                .join("common.mi"),
            "common",
        )
    };

    let depends_on_golem_agent_guest = |component: &mut MoonBitComponent, name: &str| {
        component.add_dependency(
            &format!("{moonbit_root_package}/{name}"),
            &Utf8Path::new("target")
                .join("wasm")
                .join("release")
                .join("build")
                .join("interface")
                .join("golem")
                .join("agent")
                .join("guest")
                .join("guest.mi"),
            "guest",
        )
    };

    let depends_on_wasm_rpc_types = |component: &mut MoonBitComponent, name: &str| {
        component.add_dependency(
            &format!("{moonbit_root_package}/{name}"),
            &Utf8Path::new("target")
                .join("wasm")
                .join("release")
                .join("build")
                .join("interface")
                .join("golem")
                .join("rpc")
                .join("types")
                .join("types.mi"),
            "types",
        )
    };

    depends_on_golem_agent_common(&mut component, "interface/golem/agent/guest")?;
    depends_on_golem_agent_common(
        &mut component,
        &format!("gen/interface/{pkg_namespace}/{pkg_name}/agent"),
    )?;
    depends_on_golem_agent_common(&mut component, "gen/interface/golem/agent/guest")?;
    depends_on_golem_agent_guest(&mut component, "gen/interface/golem/agent/guest")?;

    depends_on_wasm_rpc_types(&mut component, "interface/golem/agent/common")?;
    depends_on_wasm_rpc_types(&mut component, "interface/golem/agent/guest")?;

    depends_on_golem_agent_common(&mut component, "gen")?;
    depends_on_wasm_rpc_types(&mut component, "gen")?;

    component.add_dependency(
        &format!("{moonbit_root_package}/interface/golem/rpc/types"),
        &Utf8Path::new("target")
            .join("wasm")
            .join("release")
            .join("build")
            .join("interface")
            .join("wasi")
            .join("io")
            .join("poll")
            .join("poll.mi"),
        "poll",
    )?;
    component.add_dependency(
        &format!("{moonbit_root_package}/interface/golem/rpc/types"),
        &Utf8Path::new("target")
            .join("wasm")
            .join("release")
            .join("build")
            .join("interface")
            .join("wasi")
            .join("clocks")
            .join("wallClock")
            .join("wallClock.mi"),
        "wallClock",
    )?;

    let world_stub_mbt = String::new();
    component
        .write_world_stub(&world_stub_mbt)
        .context("Writing world stub")?;

    const AGENT_GUEST_MBT: &str = include_str!("../../../agent_wrapper/guest.mbt");

    component.write_interface_stub(
        &PackageName {
            namespace: "golem".to_string(),
            name: "agent".to_string(),
            version: None,
        },
        "guest",
        AGENT_GUEST_MBT,
    )?;

    let mut agent_stub = String::new();
    // TODO
    component.write_interface_stub(
        &PackageName {
            namespace: pkg_namespace,
            name: pkg_name,
            version: None,
        },
        "agent",
        &agent_stub,
    )?;

    component
        .build(
            None,
            Utf8Path::from_path(target).ok_or_else(|| {
                anyhow!("Invalid target path for the agent wrapper component: {target:?}")
            })?,
        )
        .context("Building component")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::model::agent::moonbit::generate_moonbit_wrapper;
    use crate::model::agent::test;
    use crate::model::agent::wit::generate_agent_wrapper_wit;
    use crate::model::app::AppComponentName;
    use tempfile::NamedTempFile;
    use test_r::test;

    #[cfg(test)]
    struct Trace;

    #[cfg(test)]
    #[test_r::test_dep]
    fn initialize_trace() -> Trace {
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Debug)
            .write_style(pretty_env_logger::env_logger::WriteStyle::Always)
            .init();
        Trace
    }

    #[test]
    fn multi_agent_example(_trace: &Trace) {
        let component_name: AppComponentName = "example:multi1".into();
        let agent_types = test::multi_agent_wrapper_2_types();
        let ctx = generate_agent_wrapper_wit(&component_name, &agent_types).unwrap();

        let target = NamedTempFile::new().unwrap();
        generate_moonbit_wrapper(ctx, target.path()).unwrap();
    }
}
