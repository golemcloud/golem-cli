// Copyright 2024 Golem Cloud
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

use golem_wasm_rpc_stubgen::stub::StubDefinition;
use golem_wasm_rpc_stubgen::wit::{copy_wit_files, generate_stub_wit};
use golem_wasm_rpc_stubgen::WasmRpcOverride;
use std::path::Path;
use tempfile::tempdir;
use wit_parser::{FunctionKind, Resolve, TypeDefKind, TypeOwner};

///! Tests in this module are verifying the STUB WIT created by the stub generator

#[test]
fn all_wit_types() {
    // TODO: extract some of to the main `wit` module
    let source_wit_root = Path::new("test-data/all-wit-types");
    let target_root = tempdir().unwrap();

    let def = StubDefinition::new(
        source_wit_root,
        target_root.path(),
        &None,
        "1.0.0",
        &WasmRpcOverride::default(),
        false,
    )
    .unwrap();
    generate_stub_wit(&def).unwrap();
    copy_wit_files(&def).unwrap();
    let resolve = def.verify_target_wits().unwrap();

    assert_has_package_name(&resolve, "test:main-stub");
    assert_has_world(&resolve, "wasm-rpc-stub-api");
    assert_has_interface(&resolve, "stub-api");

    assert_has_stub_function(&resolve, "stub-api", "iface1", "no-op", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-bool", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-bool", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-bool", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s8", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s16", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s32", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s64", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u8", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u16", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u32", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u64", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-f32", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-f64", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-char", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-string", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-orders", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-orders", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "apply-metadata", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-option-bool", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-option-bool", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-coordinates", true);
    assert_has_stub_function(
        &resolve,
        "stub-api",
        "iface1",
        "get-coordinates-alias",
        true,
    );
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-coordinates", false);
    assert_has_stub_function(
        &resolve,
        "stub-api",
        "iface1",
        "set-coordinates-alias",
        false,
    );
    assert_has_stub_function(&resolve, "stub-api", "iface1", "tuple-to-point", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "pt-log-error", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "validate-pt", true);
    assert_has_stub_function(
        &resolve,
        "stub-api",
        "iface1",
        "print-checkout-result",
        true,
    );
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-checkout-result", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-color", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-color", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "validate-permissions", true);
}

#[test]
fn all_wit_types_inlined() {
    let source_wit_root = Path::new("test-data/all-wit-types");
    let target_root = tempdir().unwrap();

    let def = StubDefinition::new(
        source_wit_root,
        target_root.path(),
        &None,
        "1.0.0",
        &WasmRpcOverride::default(),
        true,
    )
    .unwrap();
    generate_stub_wit(&def).unwrap();
    copy_wit_files(&def).unwrap();
    let resolve = def.verify_target_wits().unwrap();

    assert_has_package_name(&resolve, "test:main-stub");
    assert_has_world(&resolve, "wasm-rpc-stub-api");
    assert_has_interface(&resolve, "stub-api");

    assert_has_stub_function(&resolve, "stub-api", "iface1", "no-op", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-bool", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-bool", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-bool", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s8", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s16", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s32", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-s64", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u8", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u16", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u32", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-u64", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-f32", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-f64", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-char", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "identity-string", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-orders", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-orders", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "apply-metadata", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-option-bool", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-option-bool", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-coordinates", true);
    assert_has_stub_function(
        &resolve,
        "stub-api",
        "iface1",
        "get-coordinates-alias",
        true,
    );
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-coordinates", false);
    assert_has_stub_function(
        &resolve,
        "stub-api",
        "iface1",
        "set-coordinates-alias",
        false,
    );
    assert_has_stub_function(&resolve, "stub-api", "iface1", "tuple-to-point", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "pt-log-error", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "validate-pt", true);
    assert_has_stub_function(
        &resolve,
        "stub-api",
        "iface1",
        "print-checkout-result",
        true,
    );
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-checkout-result", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "get-color", true);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "set-color", false);
    assert_has_stub_function(&resolve, "stub-api", "iface1", "validate-permissions", true);

    assert_defines_enum(&resolve, "stub-api", "color");
    assert_defines_flags(&resolve, "stub-api", "permissions");
    assert_defines_record(&resolve, "stub-api", "metadata");
    assert_defines_record(&resolve, "stub-api", "point");
    assert_defines_record(&resolve, "stub-api", "product-item");
    assert_defines_record(&resolve, "stub-api", "order");
    assert_defines_record(&resolve, "stub-api", "order-confirmation");
    assert_defines_tuple_alias(&resolve, "stub-api", "point-tuple");
    assert_defines_variant(&resolve, "stub-api", "checkout-result");
}

fn assert_has_package_name(resolve: &Resolve, package_name: &str) {
    assert!(resolve
        .packages
        .iter()
        .any(|(_pkg_id, pkg)| pkg.name.to_string() == package_name.to_string()))
}

fn assert_has_world(resolve: &Resolve, world_name: &str) {
    assert!(resolve
        .worlds
        .iter()
        .any(|(_world_id, world)| &world.name == world_name))
}

fn assert_has_interface(resolve: &Resolve, interface_name: &str) {
    assert!(resolve
        .interfaces
        .iter()
        .any(|(_iface_id, iface)| iface.name == Some(interface_name.to_string())))
}

fn assert_has_stub_function(
    resolve: &Resolve,
    interface_name: &str,
    resource_name: &str,
    function_name: &str,
    has_result: bool,
) {
    let (_, iface) = resolve
        .interfaces
        .iter()
        .find(|(_iface_id, iface)| iface.name == Some(interface_name.to_string()))
        .unwrap();
    let (_, resource_id) = iface
        .types
        .iter()
        .find(|(name, _typ_id)| name.as_str() == resource_name)
        .unwrap();
    let resource_typ = resolve.types.get(resource_id.clone()).unwrap();
    assert_eq!(resource_typ.kind, TypeDefKind::Resource);

    let async_function_name = format!("[method]{resource_name}.{function_name}");
    let blocking_function_name = format!("[method]{resource_name}.blocking-{function_name}");

    let (_, _async_function) = iface
        .functions
        .iter()
        .find(|(_, fun)| {
            fun.kind == FunctionKind::Method(resource_id.clone())
                && fun.name == async_function_name.to_string()
        })
        .expect(&format!(
            "Could not find method {async_function_name} in interface {interface_name}"
        ));
    let (_, _blocking_function) = iface
        .functions
        .iter()
        .find(|(_, fun)| {
            fun.kind == FunctionKind::Method(resource_id.clone())
                && fun.name == blocking_function_name.to_string()
        })
        .expect(&format!(
            "Could not find method {blocking_function_name} in interface {interface_name}"
        ));

    if has_result {
        // for functions with a result value the async version returns a generated resource type
        let future_result_name = format!("future-{function_name}-result");
        let (_, result_resource_id) = iface
            .types
            .iter()
            .find(|(name, _typ_id)| name.as_str() == future_result_name)
            .unwrap();
        let result_resource_typ = resolve.types.get(result_resource_id.clone()).unwrap();
        assert_eq!(result_resource_typ.kind, TypeDefKind::Resource);
    }
}

fn assert_defines_enum(resolve: &Resolve, interface_name: &str, enum_name: &str) {
    assert!(resolve
        .types
        .iter()
        .any(|(_, typ)| typ.name == Some(enum_name.to_string())
            && matches!(typ.kind, TypeDefKind::Enum(_))
            && is_owned_by_interface(resolve, &typ.owner, interface_name)))
}

fn assert_defines_flags(resolve: &Resolve, interface_name: &str, flags_name: &str) {
    assert!(resolve
        .types
        .iter()
        .any(|(_, typ)| typ.name == Some(flags_name.to_string())
            && matches!(typ.kind, TypeDefKind::Flags(_))
            && is_owned_by_interface(resolve, &typ.owner, interface_name)))
}

fn assert_defines_record(resolve: &Resolve, interface_name: &str, record_name: &str) {
    assert!(resolve
        .types
        .iter()
        .any(|(_, typ)| typ.name == Some(record_name.to_string())
            && matches!(typ.kind, TypeDefKind::Record(_))
            && is_owned_by_interface(resolve, &typ.owner, interface_name)))
}

fn assert_defines_tuple_alias(resolve: &Resolve, interface_name: &str, alias_name: &str) {
    assert!(resolve
        .types
        .iter()
        .any(|(_, typ)| typ.name == Some(alias_name.to_string())
            && matches!(typ.kind, TypeDefKind::Tuple(_))
            && is_owned_by_interface(resolve, &typ.owner, interface_name)))
}

fn assert_defines_variant(resolve: &Resolve, interface_name: &str, variant_name: &str) {
    assert!(resolve
        .types
        .iter()
        .any(|(_, typ)| typ.name == Some(variant_name.to_string())
            && matches!(typ.kind, TypeDefKind::Variant(_))
            && is_owned_by_interface(resolve, &typ.owner, interface_name)))
}

fn is_owned_by_interface(resolve: &Resolve, owner: &TypeOwner, interface_name: &str) -> bool {
    match owner {
        TypeOwner::World(_) => false,
        TypeOwner::Interface(iface_id) => {
            resolve.interfaces.get(iface_id.clone()).unwrap().name
                == Some(interface_name.to_string())
        }
        TypeOwner::None => false,
    }
}
