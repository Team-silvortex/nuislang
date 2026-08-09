use super::*;

fn host_context_version_thunk() -> Option<&'static [u8]> {
    #[cfg(target_arch = "aarch64")]
    return Some(&[0x00, 0x08, 0x40, 0xb9, 0xc0, 0x03, 0x5f, 0xd6]);
    #[cfg(target_arch = "x86_64")]
    return Some(&[0x8b, 0x47, 0x08, 0xc3]);
    #[allow(unreachable_code)]
    None
}

fn request<'a>(bytes: &'a [u8], expected_hash: &'a str) -> ExecutableEntryRequest<'a> {
    ExecutableEntryRequest {
        execution_identity_hash: "0x1111111111111111",
        section_id: "sec.native-entry",
        section_kind: NUIS_NATIVE_ENTRY_SECTION_KIND,
        expected_code_hash: expected_hash,
        entry_symbol: "main",
        entry_offset: 0,
        entry_size_bytes: bytes.len(),
        abi_contract: NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1,
        target_machine_arch: native_host_machine_arch().expect("supported test host"),
        runtime_dispatch_import_identity_hash: None,
        code_bytes: bytes,
    }
}

fn permit(
    request: &ExecutableEntryRequest<'_>,
    context: &NativeLifecycleEntryContextV1,
) -> NativeEntryInvocationPermit {
    NativeEntryInvocationPermit {
        protocol: NATIVE_ENTRY_INVOCATION_PERMIT_PROTOCOL,
        execution_identity_hash: request.execution_identity_hash.to_owned(),
        section_id: request.section_id.to_owned(),
        entry_symbol: request.entry_symbol.to_owned(),
        target_machine_arch: request.target_machine_arch.to_owned(),
        runtime_dispatch_import_identity_hash: request
            .runtime_dispatch_import_identity_hash
            .map(str::to_owned),
        context_identity_hash: context.identity_hash(),
        dispatch_table_identity: context.dispatch_table_identity(),
        dispatch_capability_mask: context.dispatch_capability_mask(),
    }
}

#[test]
fn native_host_adapter_maps_rx_and_invokes_one_shot_entry() {
    let Some(bytes) = host_context_version_thunk() else {
        return;
    };
    let expected_hash = fnv1a64_hex(bytes);
    let preparation = NativeHostExecutableMemoryAdapter.prepare(&request(bytes, &expected_hash));
    assert!(preparation.ready, "{:?}", preparation.blockers);
    assert_eq!(preparation.status, "ready");
    assert_eq!(preparation.protection_status, "sealed-read-execute");
    assert_eq!(preparation.entry_bounds_status, "verified");
    assert_eq!(preparation.machine_arch_status, "verified-host-match");

    let context = NativeLifecycleEntryContextV1::test_fixture();
    // SAFETY: the fixture accepts and ignores the immutable context pointer.
    let authorized =
        preparation.authorize(permit(&request(bytes, &expected_hash), &context), context);
    let result = unsafe {
        authorized
            .expect("matching permit authorizes entry")
            .invoke()
    };
    assert!(result.invoked);
    assert_eq!(result.status, "invoked");
    assert_eq!(result.return_value, Some(1));
}

#[test]
fn section_hash_drift_blocks_before_mapping_or_invocation() {
    let Some(bytes) = host_context_version_thunk() else {
        return;
    };
    let expected_hash = fnv1a64_hex(bytes);
    let mut request = request(bytes, &expected_hash);
    request.expected_code_hash = "0xaaaaaaaaaaaaaaaa";
    let context = NativeLifecycleEntryContextV1::test_fixture();
    let permit = permit(&request, &context);
    let preparation = NativeHostExecutableMemoryAdapter.prepare(&request);
    assert!(!preparation.ready);
    assert_eq!(preparation.mapping_size_bytes, 0);
    assert!(preparation
        .blockers
        .contains(&"executable-memory:section-bytes-unverified".to_owned()));
    let result = preparation
        .authorize(permit, context)
        .err()
        .expect("blocked preparation cannot be authorized");
    assert!(!result.invoked);
    assert_eq!(result.return_value, None);
}

#[test]
fn invalid_entry_range_and_abi_fail_closed() {
    let Some(bytes) = host_context_version_thunk() else {
        return;
    };
    let expected_hash = fnv1a64_hex(bytes);
    let mut request = request(bytes, &expected_hash);
    request.entry_offset = bytes.len();
    request.entry_size_bytes = 1;
    request.abi_contract = "foreign-entry-abi";
    request.section_kind = "compiled-artifact";
    let preparation = NativeHostExecutableMemoryAdapter.prepare(&request);
    assert!(!preparation.ready);
    assert!(preparation
        .blockers
        .contains(&"executable-memory:entry-range-invalid".to_owned()));
    assert!(preparation
        .blockers
        .contains(&"executable-memory:entry-abi-unsupported".to_owned()));
    assert!(preparation
        .blockers
        .contains(&"executable-memory:section-kind-unsupported".to_owned()));
}

#[test]
fn target_machine_arch_drift_blocks_before_mapping() {
    let Some(bytes) = host_context_version_thunk() else {
        return;
    };
    let expected_hash = fnv1a64_hex(bytes);
    let mut request = request(bytes, &expected_hash);
    request.target_machine_arch = match native_host_machine_arch() {
        Some(NUIS_MACHINE_ARCH_AARCH64) => NUIS_MACHINE_ARCH_X86_64,
        Some(NUIS_MACHINE_ARCH_X86_64) => NUIS_MACHINE_ARCH_AARCH64,
        _ => return,
    };

    let preparation = NativeHostExecutableMemoryAdapter.prepare(&request);
    assert!(!preparation.ready);
    assert_eq!(preparation.mapping_size_bytes, 0);
    assert_eq!(preparation.machine_arch_status, "host-mismatch");
    assert!(preparation
        .blockers
        .contains(&"executable-memory:host-machine-arch-mismatch".to_owned()));
}

#[test]
fn mismatched_permit_cannot_authorize_prepared_entry() {
    let Some(bytes) = host_context_version_thunk() else {
        return;
    };
    let expected_hash = fnv1a64_hex(bytes);
    let request = request(bytes, &expected_hash);
    let preparation = NativeHostExecutableMemoryAdapter.prepare(&request);
    let context = NativeLifecycleEntryContextV1::test_fixture();
    let mut permit = permit(&request, &context);
    permit.execution_identity_hash = "0x2222222222222222".to_owned();
    permit.dispatch_table_identity ^= 1;

    let result = preparation
        .authorize(permit, context)
        .err()
        .expect("identity-drifted permit must fail");
    assert!(!result.invoked);
    assert!(result
        .blockers
        .contains(&"native-entry-authorization:permit-identity-mismatch".to_owned()));
    assert!(result
        .blockers
        .contains(&"native-entry-authorization:dispatch-identity-mismatch".to_owned()));
}
