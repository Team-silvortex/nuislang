use std::path::Path;

use super::{
    HostFfiMemoryCapability, HostFfiMemoryDestructor, HostFfiMemoryKind, HostFfiMemorySlot,
    HostFfiRegistryView,
};
use yir_core::ffi::{ffi_memory_capability_hash, ffi_symbol_signature_hash};

fn cffi_manifest() -> super::NustarPackageManifest {
    super::load_manifest_for_domain(Path::new("nustar-packages"), "cffi")
        .expect("official CFFI manifest should load")
}

fn rehash(capability: &mut HostFfiMemoryCapability) {
    capability.capability_hash = ffi_memory_capability_hash(
        &capability.abi,
        &capability.symbol,
        &capability.signature_hash,
        &capability.descriptor(),
    );
}

#[test]
fn official_cffi_registry_exposes_hash_bound_borrowed_utf8_contracts() {
    let manifest = cffi_manifest();
    let view = HostFfiRegistryView::try_from_manifest(&manifest)
        .expect("official CFFI memory capabilities should validate");
    let signature_hash = ffi_symbol_signature_hash("libc", "puts", "i32(String)");

    let capabilities = view.memory_capabilities("libc", "puts", &signature_hash);

    assert_eq!(manifest.host_ffi_memory_capabilities.len(), 5);
    assert_eq!(capabilities.len(), 1);
    assert_eq!(capabilities[0].kind, HostFfiMemoryKind::BorrowedUtf8);
    assert_eq!(capabilities[0].slot, HostFfiMemorySlot::Arg(0));
    assert_eq!(capabilities[0].length, "nul_terminated");
    assert_eq!(capabilities[0].mutability, "read_only");
    assert_eq!(capabilities[0].lifetime, "call");
    assert_eq!(capabilities[0].destructor, HostFfiMemoryDestructor::None);
}

#[test]
fn accepts_owned_return_buffer_with_exact_registered_destructor() {
    let mut manifest = cffi_manifest();
    let acquire_hash = ffi_symbol_signature_hash("c", "test_buffer_acquire", "ref_Buffer(i64)");
    let release_hash = ffi_symbol_signature_hash("c", "test_buffer_release", "i64(ref_Buffer)");
    manifest.abi_capabilities.push(
        "c:ffi_symbol:test_buffer_acquire=ref_Buffer(i64)|ffi_symbol:test_buffer_release=i64(ref_Buffer)"
            .to_owned(),
    );
    let capability = HostFfiMemoryCapability::owned_return_buffer(
        "c",
        "test_buffer_acquire",
        &acquire_hash,
        "test_buffer_release",
        &release_hash,
    );
    manifest
        .host_ffi_memory_capabilities
        .push(capability.render());

    let view = HostFfiRegistryView::try_from_manifest(&manifest)
        .expect("owned return-buffer contract should validate");

    assert_eq!(
        view.memory_capabilities("c", "test_buffer_acquire", &acquire_hash),
        &[capability]
    );
}

#[test]
fn rejects_borrowed_utf8_policy_field_drift_before_lowering() {
    for (field, expected) in [
        ("lifetime", "lifetime, length, mutability"),
        ("length", "lifetime, length, mutability"),
        ("mutability", "lifetime, length, mutability"),
        ("destructor", "lifetime, length, mutability"),
    ] {
        let mut manifest = cffi_manifest();
        let signature_hash = ffi_symbol_signature_hash("libc", "puts", "i32(String)");
        let mut capability =
            HostFfiMemoryCapability::borrowed_utf8("libc", "puts", &signature_hash, 0);
        match field {
            "lifetime" => capability.lifetime = "retained".to_owned(),
            "length" => capability.length = "unchecked".to_owned(),
            "mutability" => capability.mutability = "mutable".to_owned(),
            "destructor" => {
                capability.destructor = HostFfiMemoryDestructor::Registered {
                    symbol: "free".to_owned(),
                    signature_hash: "fnv1a64:0000000000000000".to_owned(),
                };
            }
            _ => unreachable!(),
        }
        rehash(&mut capability);
        manifest.host_ffi_memory_capabilities[0] = capability.render();

        let error = HostFfiRegistryView::try_from_manifest(&manifest)
            .expect_err("borrowed UTF-8 policy drift must be rejected");

        assert!(error.contains(expected), "field={field}, error={error}");
    }
}

#[test]
fn rejects_memory_capability_hash_and_signature_drift() {
    let mut hash_drift = cffi_manifest();
    hash_drift.host_ffi_memory_capabilities[0] = hash_drift.host_ffi_memory_capabilities[0]
        .replacen(
            "@fnv1a64:588094eacdd1e033=",
            "@fnv1a64:088094eacdd1e033=",
            1,
        );
    let hash_error = HostFfiRegistryView::try_from_manifest(&hash_drift)
        .expect_err("capability hash drift must be rejected");
    assert!(hash_error.contains("capability hash mismatch"));

    let mut signature_drift = cffi_manifest();
    let mut capability =
        HostFfiMemoryCapability::borrowed_utf8("libc", "puts", "fnv1a64:0000000000000000", 0);
    rehash(&mut capability);
    signature_drift.host_ffi_memory_capabilities[0] = capability.render();
    let signature_error = HostFfiRegistryView::try_from_manifest(&signature_drift)
        .expect_err("signature drift must be rejected");
    assert!(signature_error.contains("matching exact `ffi_symbol:` registration"));
}

#[test]
fn rejects_owned_return_buffer_destructor_drift() {
    let mut manifest = cffi_manifest();
    let acquire_hash = ffi_symbol_signature_hash("c", "test_buffer_acquire", "ref_Buffer(i64)");
    manifest
        .abi_capabilities
        .push("c:ffi_symbol:test_buffer_acquire=ref_Buffer(i64)".to_owned());
    let mut capability = HostFfiMemoryCapability::owned_return_buffer(
        "c",
        "test_buffer_acquire",
        &acquire_hash,
        "test_buffer_release",
        "fnv1a64:0000000000000000",
    );
    rehash(&mut capability);
    manifest
        .host_ffi_memory_capabilities
        .push(capability.render());

    let error = HostFfiRegistryView::try_from_manifest(&manifest)
        .expect_err("unregistered owned-buffer destructor must be rejected");

    assert!(error.contains("destructor `test_buffer_release`"));
    assert!(error.contains("not exact-signature registered"));
}
