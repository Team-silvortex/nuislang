use super::manifest::parse_optional_string_array;
use super::*;
use crate::registry::NustarPackageManifest;

#[derive(Debug)]
struct PolicyCase {
    name: &'static str,
    domain: &'static str,
    support_surface: Vec<&'static str>,
    abi_capabilities: Vec<&'static str>,
    expect_ok: bool,
    expect_error_contains: &'static str,
}

fn make_manifest(domain: &str) -> NustarPackageManifest {
    let abi_profiles = vec![format!("{domain}.abi.v1")];
    let abi_targets = if domain == "cpu" {
        vec![format!(
                "{}:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin",
                abi_profiles[0]
            )]
    } else {
        Vec::new()
    };
    NustarPackageManifest {
        manifest_schema: "nustar-manifest-v1".to_owned(),
        package_id: format!("test.{domain}"),
        domain_family: domain.to_owned(),
        frontend: format!("nustar-{domain}"),
        entry_crate: format!("crates/yir-domain-{domain}"),
        ast_entry: format!("{domain}.ast.bootstrap.v1"),
        nir_entry: format!("{domain}.nir.bootstrap.v1"),
        yir_lowering_entry: format!("{domain}.yir.lowering.v1"),
        part_verify_entry: format!("{domain}.verify.partial.v1"),
        ast_surface: vec![format!("{domain}.mod-ast.v1")],
        nir_surface: vec![format!("nir.{domain}.surface.v1")],
        yir_lowering: vec![format!("yir.{domain}.lowering.v1")],
        part_verify: vec![format!("verify.{domain}.contract.v1")],
        binary_extension: "nustar".to_owned(),
        package_layout: "single-envelope".to_owned(),
        machine_abi_policy: "exact-match".to_owned(),
        abi_profiles,
        abi_capabilities: Vec::new(),
        abi_targets,
        implementation_kinds: vec!["native-stub".to_owned()],
        loader_entry: "nustar.bootstrap.v1".to_owned(),
        loader_abi: "nustar-loader-v1".to_owned(),
        host_ffi_surface: Vec::new(),
        host_ffi_abis: Vec::new(),
        host_ffi_bridge: "none".to_owned(),
        bridge_lane_policy: None,
        bridge_surface: None,
        bridge_emission_kind: None,
        bridge_entry: None,
        bridge_kind: None,
        bridge_scheduler_binding: None,
        backend_stub_kind: None,
        backend_submission_mode: None,
        backend_wake_policy: None,
        backend_transport_model: None,
        backend_request_shape: None,
        backend_response_shape: None,
        backend_dispatch_shape: None,
        backend_memory_binding: None,
        backend_resource_binding: None,
        backend_completion_model: None,
        phase_bind: None,
        phase_submit: None,
        phase_wait: None,
        phase_finalize: None,
        host_bridge_host_ffi_surface: None,
        host_bridge_handle_family: None,
        host_bridge_phase_order: None,
        host_bridge_phase_bind_inputs: None,
        host_bridge_phase_bind_outputs: None,
        host_bridge_phase_submit_inputs: None,
        host_bridge_phase_submit_outputs: None,
        host_bridge_phase_wait_inputs: None,
        host_bridge_phase_wait_outputs: None,
        host_bridge_phase_finalize_inputs: None,
        host_bridge_phase_finalize_outputs: None,
        host_bridge_phase_bind_wake: None,
        host_bridge_phase_submit_wake: None,
        host_bridge_phase_wait_wake: None,
        host_bridge_phase_finalize_wake: None,
        host_bridge_plan_begin: None,
        host_bridge_plan_end: None,
        support_surface: Vec::new(),
        support_profile_slots: Vec::new(),
        capability_tags: Vec::new(),
        default_lanes: Vec::new(),
        clock_domain_id: format!("{domain}.clock.local.v1"),
        clock_kind: "local-monotonic".to_owned(),
        clock_epoch_kind: "domain-epoch".to_owned(),
        clock_resolution: "tick:1ns".to_owned(),
        clock_bridge_default: "self".to_owned(),
        profiles: vec!["aot".to_owned()],
        resource_families: vec![domain.to_owned()],
        unit_types: vec!["Main".to_owned()],
        lowering_targets: vec!["native".to_owned()],
        ops: vec![format!("{domain}.const")],
    }
}

#[test]
fn string_array_parser_preserves_commas_inside_quoted_ffi_signatures() {
    let values = parse_optional_string_array(
            r#"abi_capabilities = ["c:ffi_symbol:host_network_open_tcp_stream=i64(i64,i64)", "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)"]"#,
            "abi_capabilities",
        )
        .expect("array should parse");

    assert_eq!(
        values,
        vec![
            "c:ffi_symbol:host_network_open_tcp_stream=i64(i64,i64)",
            "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)"
        ]
    );
}

#[test]
fn capability_policy_table() {
    let cases = vec![
        PolicyCase {
            name: "reject cross-domain op for shader",
            domain: "shader",
            support_surface: vec!["shader.profile.packet.v1"],
            abi_capabilities: vec!["shader.abi.v1:surface:shader.profile.*|op:cpu.*"],
            expect_ok: false,
            expect_error_contains: "cross-domain op capability pattern",
        },
        PolicyCase {
            name: "reject invalid data surface prefix",
            domain: "data",
            support_surface: vec!["data.profile.bind-core.v1"],
            abi_capabilities: vec!["data.abi.v1:surface:shader.profile.*|op:data.*"],
            expect_ok: false,
            expect_error_contains: "invalid surface capability pattern",
        },
        PolicyCase {
            name: "reject missing surface capability for kernel",
            domain: "kernel",
            support_surface: vec!["kernel.profile.bind-core.v1"],
            abi_capabilities: vec!["kernel.abi.v1:op:kernel.*"],
            expect_ok: false,
            expect_error_contains: "must declare at least one `surface:` capability",
        },
        PolicyCase {
            name: "reject surface capability in cpu domain",
            domain: "cpu",
            support_surface: vec![],
            abi_capabilities: vec!["cpu.abi.v1:surface:cpu.profile.*|op:cpu.*"],
            expect_ok: false,
            expect_error_contains: "invalid surface capability pattern",
        },
        PolicyCase {
            name: "accept valid shader capability policy",
            domain: "shader",
            support_surface: vec!["shader.profile.packet.v1", "shader.inline.wgsl.v1"],
            abi_capabilities: vec![
                "shader.abi.v1:surface:shader.profile.*|surface:shader.inline.wgsl.v1|op:shader.*",
            ],
            expect_ok: true,
            expect_error_contains: "",
        },
        PolicyCase {
            name: "accept valid cpu capability policy",
            domain: "cpu",
            support_surface: vec![],
            abi_capabilities: vec!["cpu.abi.v1:op:cpu.*"],
            expect_ok: true,
            expect_error_contains: "",
        },
    ];

    for case in cases {
        let mut manifest = make_manifest(case.domain);
        manifest.support_surface = case
            .support_surface
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        manifest.abi_capabilities = case
            .abi_capabilities
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        let result = validate_manifest_for_packaging(&manifest);
        if case.expect_ok {
            assert!(
                result.is_ok(),
                "{}: unexpected error: {result:?}",
                case.name
            );
        } else {
            let error = result.unwrap_err();
            assert!(
                error.contains(case.expect_error_contains),
                "{}: unexpected error: {}",
                case.name,
                error
            );
        }
    }
}

#[test]
fn implementation_contracts_include_abi_target_metadata() {
    let mut manifest = make_manifest("cpu");
    manifest.implementation_kinds = vec!["native-dylib".to_owned(), "llvm-bc".to_owned()];
    let binary = default_binary(manifest, Vec::new());
    let contracts = implementation_contracts(&binary);
    for contract in contracts {
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item.starts_with("abi_target=")),
            "missing abi_target metadata in {} contract",
            contract.kind
        );
    }
}

#[test]
fn network_loader_contracts_include_control_host_symbol_metadata() {
    let mut manifest = make_manifest("network");
    manifest.implementation_kinds = vec![
        "native-stub".to_owned(),
        "native-dylib".to_owned(),
        "llvm-bc".to_owned(),
    ];
    let binary = default_binary(manifest, Vec::new());
    let contracts = implementation_contracts(&binary);
    for contract in contracts {
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.connect:host_network_connect_probe"),
            "missing network.connect host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.accept:host_network_accept_probe"),
            "missing network.accept host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract.required_metadata.iter().any(|item| item
                == "host_symbol=network.open_tcp_listener:host_network_open_tcp_listener"),
            "missing network.open_tcp_listener host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.bind_udp:host_network_bind_udp_datagram"),
            "missing network.bind_udp host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.accept_owned:host_network_accept_owned"),
            "missing network.accept_owned host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.close:host_network_close"),
            "missing network.close host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.send_owned:host_network_send_owned"),
            "missing network.send_owned host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.recv_owned:host_network_recv_owned"),
            "missing network.recv_owned host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
                contract.required_metadata.iter().any(|item| item
                    == "host_symbol=network.recv_http_status_owned:host_network_recv_http_status_owned"),
                "missing network.recv_http_status_owned host symbol metadata in {} contract",
                contract.kind
            );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.send:host_network_send_probe"),
            "missing network.send host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract
                .required_metadata
                .iter()
                .any(|item| item == "host_symbol=network.recv:host_network_recv_probe"),
            "missing network.recv host symbol metadata in {} contract",
            contract.kind
        );
        assert!(
            contract.notes.contains("host_network_connect_probe"),
            "missing runtime symbol note in {} contract",
            contract.kind
        );
    }
}

#[test]
fn reject_missing_capability_mapping_for_one_profile() {
    let mut manifest = make_manifest("data");
    manifest.abi_profiles = vec!["data.abi.v1".to_owned(), "data.abi.alt.v1".to_owned()];
    manifest.support_surface = vec!["data.profile.bind-core.v1".to_owned()];
    manifest.abi_capabilities = vec!["data.abi.v1:surface:data.profile.*|op:data.*".to_owned()];

    let error = validate_manifest_for_packaging(&manifest).unwrap_err();
    assert!(
        error.contains("has no abi_capabilities mapping"),
        "unexpected error: {error}"
    );
}
