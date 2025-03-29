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

//! Tests for the 'add stub as a dependency' mechanism.

use test_r::test;

use assert2::assert;
use fs_extra::dir::CopyOptions;
use golem_wasm_rpc_stubgen::commands::generate::generate_client_wit_dir;
use golem_wasm_rpc_stubgen::model::app::ComponentName;
use golem_wasm_rpc_stubgen::stub::{
    RustDependencyOverride, StubConfig, StubDefinition, StubSourceTransform,
};
use golem_wasm_rpc_stubgen::wit_generate::{
    add_client_as_dependency_to_wit_dir, AddClientAsDepConfig, UpdateCargoToml,
};
use golem_wasm_rpc_stubgen::wit_resolve::ResolvedWitDir;
use golem_wit::{WASI_POLL_WIT, WASI_WALL_CLOCKS_WIT, WASM_RPC_WIT};
use semver::Version;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use wit_encoder::{packages_from_parsed, Package, PackageName};
use wit_parser::Resolve;

test_r::enable!();

fn all_wit_types_no_collision(source_transform: StubSourceTransform) {
    let (_source_dir, stub_dir) = init_stub("all-wit-types", source_transform);
    let dest_dir = init_caller("caller-no-dep");

    let stub_wit_root = stub_dir.path().join("wit");
    let dest_wit_root = dest_dir.path().join("wit");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_wit_root.clone(),
        dest_wit_root: dest_wit_root.clone(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(&dest_wit_root);

    assert_has_wasm_rpc_wit_deps(&dest_wit_root);

    assert_has_same_wit_package(
        &PackageName::new("test", "main-client", None),
        &dest_wit_root,
        &stub_wit_root,
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "main"),
            None,
        ),
        &dest_wit_root,
        &stub_wit_root,
    );
}

#[test]
fn all_wit_types_no_collisions_extract() {
    all_wit_types_no_collision(StubSourceTransform::ExtractExportsPackage)
}

#[test]
fn all_wit_types_no_collisions_strip() {
    all_wit_types_no_collision(StubSourceTransform::StripSourcePackage)
}

fn all_wit_types_re_add_with_changes(source_transform: StubSourceTransform) {
    let (source_dir, stub_dir) = init_stub("all-wit-types", source_transform);
    let (alternative_source_dir, alternative_stub_dir) =
        init_stub("all-wit-types-alternative", source_transform);
    let dest_dir = init_caller("caller-no-dep");

    let stub_wit_root = stub_dir.path().join("wit");
    let alternative_stub_wit_root = alternative_stub_dir.path().join("wit");
    let dest_wit_root = dest_dir.path().join("wit");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_wit_root.clone(),
        dest_wit_root: dest_wit_root.clone(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(&dest_wit_root);
    assert_has_wasm_rpc_wit_deps(&dest_wit_root);
    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "main"),
            None,
        ),
        source_dir.path(),
        &stub_wit_root,
    );
    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "main"),
            None,
        ),
        source_dir.path(),
        &dest_wit_root,
    );
    assert_has_same_wit_package(
        &PackageName::new("test", "main-client", None),
        &stub_wit_root,
        &dest_wit_root,
    );

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: alternative_stub_wit_root.clone(),
        dest_wit_root: dest_wit_root.clone(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(&dest_wit_root);
    assert_has_wasm_rpc_wit_deps(&dest_wit_root);
    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "main"),
            None,
        ),
        alternative_source_dir.path(),
        &alternative_stub_wit_root,
    );
    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "main"),
            None,
        ),
        alternative_source_dir.path(),
        &dest_wit_root,
    );
    assert_has_same_wit_package(
        &PackageName::new("test", "main-client", None),
        &alternative_stub_wit_root,
        &dest_wit_root,
    );
}

#[test]
fn all_wit_types_re_add_with_changes_extract() {
    all_wit_types_re_add_with_changes(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn all_wit_types_re_add_with_changes_strip() {
    all_wit_types_re_add_with_changes(StubSourceTransform::StripSourcePackage);
}

fn many_ways_to_export_no_collision(source_transform: StubSourceTransform) {
    let (source_dir, stub_dir) = init_stub("many-ways-to-export", source_transform);
    let dest_dir = init_caller("caller-no-dep");

    let stub_wit_root = stub_dir.path().join("wit");
    let dest_wit_root = dest_dir.path().join("wit");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_wit_root.clone(),
        dest_wit_root: dest_wit_root.clone(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(&dest_wit_root);

    assert_has_wasm_rpc_wit_deps(&dest_wit_root);

    assert_has_same_wit_package(
        &PackageName::new("test", "exports-client", None),
        &dest_wit_root,
        &stub_wit_root,
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "exports"),
            None,
        ),
        source_dir.path(),
        &dest_wit_root,
    );

    assert_has_same_wit_package(
        &PackageName::new("test", "sub", None),
        &dest_wit_root,
        Path::new("test-data/wit/many-ways-to-export/deps/sub/sub.wit"),
    );
}

#[test]
fn many_ways_to_export_with_collision_export() {
    many_ways_to_export_no_collision(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn many_ways_to_export_with_collision_strip() {
    many_ways_to_export_no_collision(StubSourceTransform::StripSourcePackage);
}

fn direct_circular(source_transform: StubSourceTransform) {
    let (_source_a_dir, stub_a_dir) = init_stub("direct-circular-a", source_transform);
    let (_source_b_dir, stub_b_dir) = init_stub("direct-circular-b", source_transform);

    let dest_a = init_caller("direct-circular-a");
    let dest_b = init_caller("direct-circular-b");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_b.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_b_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());
    assert_valid_wit_root(dest_b.path());

    assert_has_wasm_rpc_wit_deps(dest_a.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "b-client", None),
        dest_a.path(),
        &stub_b_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "b"),
            None,
        ),
        dest_a.path(),
        _source_b_dir.path(),
    );

    assert_has_wasm_rpc_wit_deps(dest_b.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "a-client", None),
        dest_b.path(),
        &stub_a_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "a"),
            None,
        ),
        dest_b.path(),
        _source_a_dir.path(),
    );
}

#[test]
fn direct_circular_extract() {
    direct_circular(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn direct_circular_strip() {
    direct_circular(StubSourceTransform::StripSourcePackage);
}

fn direct_circular_readd(source_transform: StubSourceTransform) {
    let (_source_a_dir, stub_a_dir) = init_stub("direct-circular-a", source_transform);
    let (_source_b_dir, stub_b_dir) = init_stub("direct-circular-b", source_transform);

    let dest_a = init_caller("direct-circular-a");
    let dest_b = init_caller("direct-circular-b");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_b.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_b_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());
    assert_valid_wit_root(dest_b.path());

    // At this point we simulate doing stub generation and add-stub-dependency _again_ on the a.wit and b.wit which are already have the corresponding
    // stubs imported

    regenerate_stub(stub_a_dir.path(), dest_a.path(), source_transform);
    regenerate_stub(stub_b_dir.path(), dest_b.path(), source_transform);

    println!("Second round of add_stub_dependency calls");
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_b.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_b_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());
    assert_valid_wit_root(dest_b.path());

    assert_has_wasm_rpc_wit_deps(dest_a.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "b-client", None),
        dest_a.path(),
        &stub_b_dir.path().join("wit"),
    );

    // TODO: diff on circular import
    /*assert_has_same_wit_package(
        &PackageName::new("test", "b", None),
        dest_a.path(),
        dest_b.path(),
    );*/

    assert_has_wasm_rpc_wit_deps(dest_b.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "a-client", None),
        dest_b.path(),
        &stub_a_dir.path().join("wit"),
    );

    // TODO: diff on circular import
    /*
    assert_has_same_wit_package(
        &PackageName::new("test", "a", None),
        dest_b.path(),
        dest_a.path(),
    );
    */
}

#[test]
fn direct_circular_readd_extract() {
    direct_circular_readd(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn direct_circular_readd_strip() {
    direct_circular_readd(StubSourceTransform::StripSourcePackage);
}

fn direct_circular_same_world_name(source_transform: StubSourceTransform) {
    let (source_a_dir, stub_a_dir) =
        init_stub("direct-circular-a-same-world-name", source_transform);
    let (source_b_dir, stub_b_dir) =
        init_stub("direct-circular-b-same-world-name", source_transform);

    let dest_a = init_caller("direct-circular-a-same-world-name");
    let dest_b = init_caller("direct-circular-b-same-world-name");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_b.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_b_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());
    assert_valid_wit_root(dest_b.path());

    assert_has_wasm_rpc_wit_deps(dest_a.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "b-client", None),
        dest_a.path(),
        &stub_b_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "b"),
            None,
        ),
        dest_a.path(),
        source_b_dir.path(),
    );

    assert_has_wasm_rpc_wit_deps(dest_b.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "a-client", None),
        dest_b.path(),
        &stub_a_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "a"),
            None,
        ),
        dest_b.path(),
        source_a_dir.path(),
    );
}

#[test]
fn direct_circular_same_world_name_extract() {
    direct_circular_same_world_name(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn direct_circular_same_world_name_strip() {
    direct_circular_same_world_name(StubSourceTransform::StripSourcePackage);
}

fn indirect_circular(source_transform: StubSourceTransform) {
    let (source_a_dir, stub_a_dir) = init_stub("indirect-circular-a", source_transform);
    let (_source_b_dir, stub_b_dir) = init_stub("indirect-circular-b", source_transform);
    let (_source_c_dir, stub_c_dir) = init_stub("indirect-circular-c", source_transform);

    let dest_a = init_caller("indirect-circular-a");
    let dest_b = init_caller("indirect-circular-b");
    let dest_c = init_caller("indirect-circular-c");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_c.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_b_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_c_dir.path().join("wit"),
        dest_wit_root: dest_b.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());
    assert_valid_wit_root(dest_b.path());
    assert_valid_wit_root(dest_c.path());

    assert_has_wasm_rpc_wit_deps(dest_a.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "b-client", None),
        dest_a.path(),
        &stub_b_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "b"),
            None,
        ),
        dest_a.path(),
        &stub_b_dir.path().join("wit"),
    );

    assert_has_wasm_rpc_wit_deps(dest_b.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "c-client", None),
        dest_b.path(),
        &stub_c_dir.path().join("wit"),
    );

    assert_has_wasm_rpc_wit_deps(dest_c.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "a-client", None),
        dest_c.path(),
        &stub_a_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "a"),
            None,
        ),
        dest_c.path(),
        source_a_dir.path(),
    );
}

#[test]
fn indirect_circular_extract() {
    indirect_circular(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn indirect_circular_strip() {
    indirect_circular(StubSourceTransform::StripSourcePackage);
}

fn indirect_circular_readd(source_transform: StubSourceTransform) {
    let (_source_a_dir, stub_a_dir) = init_stub("indirect-circular-a", source_transform);
    let (_source_b_dir, stub_b_dir) = init_stub("indirect-circular-b", source_transform);
    let (_source_c_dir, stub_c_dir) = init_stub("indirect-circular-c", source_transform);

    let dest_a = init_caller("indirect-circular-a");
    let dest_b = init_caller("indirect-circular-b");
    let dest_c = init_caller("indirect-circular-c");

    println!("dest_a: {:?}", dest_a.path());
    println!("dest_b: {:?}", dest_b.path());
    println!("dest_c: {:?}", dest_c.path());

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_c.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_b_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_c_dir.path().join("wit"),
        dest_wit_root: dest_b.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());
    assert_valid_wit_root(dest_b.path());
    assert_valid_wit_root(dest_c.path());

    // At this point we simulate doing stub generation and add-stub-dependency _again_ on the a.wit and b.wit which are already have the corresponding
    // stubs imported

    regenerate_stub(stub_a_dir.path(), dest_a.path(), source_transform);
    regenerate_stub(stub_b_dir.path(), dest_b.path(), source_transform);
    regenerate_stub(stub_c_dir.path(), dest_c.path(), source_transform);

    println!("Second round of add_stub_dependency calls");
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_c.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_b_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();
    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_c_dir.path().join("wit"),
        dest_wit_root: dest_b.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());
    assert_valid_wit_root(dest_b.path());
    assert_valid_wit_root(dest_c.path());

    assert_has_wasm_rpc_wit_deps(dest_a.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "b-client", None),
        dest_a.path(),
        &stub_b_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "b"),
            None,
        ),
        dest_a.path(),
        dest_b.path(),
    );

    assert_has_wasm_rpc_wit_deps(dest_b.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "c-client", None),
        dest_b.path(),
        &stub_c_dir.path().join("wit"),
    );

    assert_has_no_package_by_name(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "c"),
            None,
        ),
        dest_b.path(),
    );
    assert_has_package_by_name(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "c"),
            None,
        ),
        dest_c.path(),
    );

    assert_has_wasm_rpc_wit_deps(dest_c.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "a-client", None),
        dest_c.path(),
        &stub_a_dir.path().join("wit"),
    );

    assert_has_same_wit_package(
        &PackageName::new(
            "test",
            transformed_source_package_name(source_transform, "a"),
            None,
        ),
        dest_c.path(),
        dest_a.path(),
    );
    assert_has_no_package_by_name(&PackageName::new("test", "a", None), dest_c.path());
    assert_has_package_by_name(&PackageName::new("test", "a", None), dest_a.path());
}

#[test]
fn indirect_circular_readd_extract() {
    indirect_circular_readd(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn indirect_circular_readd_strip() {
    indirect_circular_readd(StubSourceTransform::StripSourcePackage);
}

fn self_circular(source_transform: StubSourceTransform) {
    let (_source_a_dir, stub_a_dir) = init_stub("self-circular", source_transform);

    let dest_a = init_caller("self-circular");

    add_client_as_dependency_to_wit_dir(AddClientAsDepConfig {
        client_wit_root: stub_a_dir.path().join("wit"),
        dest_wit_root: dest_a.path().to_path_buf(),
        update_cargo_toml: UpdateCargoToml::NoUpdate,
    })
    .unwrap();

    assert_valid_wit_root(dest_a.path());

    assert_has_wasm_rpc_wit_deps(dest_a.path());

    assert_has_same_wit_package(
        &PackageName::new("test", "a-client", None),
        dest_a.path(),
        &stub_a_dir.path().join("wit"),
    );
}

#[test]
fn self_circular_extract() {
    self_circular(StubSourceTransform::ExtractExportsPackage);
}

#[test]
fn self_circular_strip() {
    self_circular(StubSourceTransform::StripSourcePackage);
}

fn init_stub(name: &str, source_transform: StubSourceTransform) -> (TempDir, TempDir) {
    let source = TempDir::new().unwrap();
    let canonical_source = source.path().canonicalize().unwrap();

    fs_extra::dir::copy(
        Path::new("test-data/wit").join(name),
        &canonical_source,
        &CopyOptions::new().content_only(true),
    )
    .unwrap();

    let target = TempDir::new().unwrap();
    let canonical_target = target.path().canonicalize().unwrap();

    let def = StubDefinition::new(StubConfig {
        source_wit_root: canonical_source,
        client_root: canonical_target,
        selected_world: None,
        stub_crate_version: "1.0.0".to_string(),
        golem_rust_override: RustDependencyOverride::default(),
        source_transform,
        seal_cargo_workspace: false,
        component_name: ComponentName::from("test:component"),
        is_ephemeral: false,
    })
    .unwrap();
    let _ = generate_client_wit_dir(&def).unwrap();
    (source, target)
}

fn regenerate_stub(stub_dir: &Path, source_wit_root: &Path, source_transform: StubSourceTransform) {
    let def = StubDefinition::new(StubConfig {
        source_wit_root: source_wit_root.to_path_buf(),
        client_root: stub_dir.to_path_buf(),
        selected_world: None,
        stub_crate_version: "1.0.0".to_string(),
        golem_rust_override: RustDependencyOverride::default(),
        source_transform,
        seal_cargo_workspace: false,
        component_name: ComponentName::from("test:component"),
        is_ephemeral: false,
    })
    .unwrap();
    let _ = generate_client_wit_dir(&def).unwrap();
}

fn init_caller(name: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let source = Path::new("test-data/wit").join(name);

    fs_extra::dir::copy(
        source,
        temp_dir.path(),
        &CopyOptions::new().content_only(true).overwrite(true),
    )
    .unwrap();

    temp_dir
}

fn assert_valid_wit_root(wit_root: &Path) {
    ResolvedWitDir::new(wit_root).unwrap();
}

trait WitSource {
    fn resolve(&self) -> anyhow::Result<Resolve>;

    fn encoded_packages(&self) -> anyhow::Result<Vec<Package>> {
        Ok(packages_from_parsed(&self.resolve()?))
    }

    fn encoded_package(&self, package_name: &PackageName) -> anyhow::Result<Package> {
        self.encoded_packages()?
            .into_iter()
            .find(|package| package.name() == package_name)
            .ok_or_else(|| anyhow::anyhow!("package {} not found", package_name))
    }

    fn encoded_package_wit(&self, package_name: &PackageName) -> anyhow::Result<String> {
        self.encoded_package(package_name)
            .map(|package| package.to_string())
    }
}

impl WitSource for &Path {
    fn resolve(&self) -> anyhow::Result<Resolve> {
        let mut resolve = Resolve::new();
        let _ = resolve.push_path(self)?;
        Ok(resolve)
    }
}

impl WitSource for &PathBuf {
    fn resolve(&self) -> anyhow::Result<Resolve> {
        let mut resolve = Resolve::new();
        let _ = resolve.push_path(self)?;
        Ok(resolve)
    }
}

impl WitSource for &[(&str, &str)] {
    fn resolve(&self) -> anyhow::Result<Resolve> {
        let mut resolve = Resolve::new();
        for (name, source) in *self {
            let _ = resolve.push_str(name, source)?;
        }
        Ok(resolve)
    }
}

/// Asserts that both wit sources contains the same effective (encoded) wit package.
fn assert_has_same_wit_package(
    package_name: &PackageName,
    actual_wit_source: impl WitSource,
    expected_wit_source: impl WitSource,
) {
    let actual_wit = actual_wit_source.encoded_package_wit(package_name).unwrap();
    let expected_wit = expected_wit_source
        .encoded_package_wit(package_name)
        .unwrap();
    assert_eq!(actual_wit, expected_wit)
}

fn assert_has_no_package_by_name(package_name: &PackageName, wit_source: impl WitSource) {
    assert!(wit_source.encoded_package(package_name).is_err())
}

fn assert_has_package_by_name(package_name: &PackageName, wit_source: impl WitSource) {
    assert!(wit_source.encoded_package(package_name).is_ok())
}

fn assert_has_wasm_rpc_wit_deps(wit_dir: &Path) {
    let deps = vec![
        ("poll", WASI_POLL_WIT),
        ("clocks", WASI_WALL_CLOCKS_WIT),
        ("wasm-rpc", WASM_RPC_WIT),
    ];

    assert_has_same_wit_package(
        &PackageName::new("wasi", "io", Some(Version::new(0, 2, 0))),
        wit_dir,
        deps.as_slice(),
    );
    assert_has_same_wit_package(
        &PackageName::new("wasi", "clocks", Some(Version::new(0, 2, 0))),
        wit_dir,
        deps.as_slice(),
    );
    assert_has_same_wit_package(
        &PackageName::new("golem", "rpc", Some(Version::new(0, 2, 0))),
        wit_dir,
        deps.as_slice(),
    );
}

fn transformed_source_package_name(source_transform: StubSourceTransform, name: &str) -> String {
    match source_transform {
        StubSourceTransform::None => {
            unimplemented!()
        }
        StubSourceTransform::ExtractExportsPackage => {
            format!("{}-exports", name)
        }
        StubSourceTransform::StripSourcePackage => name.to_string(),
    }
}
