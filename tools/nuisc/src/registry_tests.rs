use super::*;
use crate::project::{
    ProjectAbiRequirement, ProjectAbiResolution, ProjectCompilationPlan,
    ProjectExchangeOrganization, ProjectOrganization, ProjectOutputIntent, ProjectSyntheticInput,
};
use crate::registry_abi_target::{
    host_arch, host_calling_abi, host_clang_target, host_object_format, host_os,
};
use crate::registry_load::{resolve_registry_root, INDEX_FILE};
use crate::registry_manifest_parse::parse_optional_string_array;
use crate::registry_support_usage::{
    collect_resource_usage_hints, covered_profile_slots, detect_matched_support_usage,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn test_project_plan(domain: &str, abi: &str) -> ProjectCompilationPlan {
    ProjectCompilationPlan {
        project_name: "registry-check-demo".to_owned(),
        entry: "main.ns".to_owned(),
        organization: ProjectOrganization {
            entry: "main.ns".to_owned(),
            domains: vec![domain.to_owned()],
            modules: Vec::new(),
            links: Vec::new(),
        },
        exchanges: ProjectExchangeOrganization { routes: Vec::new() },
        abi_resolution: ProjectAbiResolution {
            requirements: vec![ProjectAbiRequirement {
                domain: domain.to_owned(),
                abi: abi.to_owned(),
            }],
            explicit: true,
        },
        dependencies: Vec::new(),
        synthetic_input: ProjectSyntheticInput {
            kind: "test".to_owned(),
            path: PathBuf::from("main.ns"),
        },
        output_intents: Vec::<ProjectOutputIntent>::new(),
        effective_input_path: PathBuf::from("main.ns"),
    }
}
use crate::pipeline;

const DATA_BINDING_SOURCE: &str = r#"
use data FabricPlane;

mod cpu Main {
  fn capture_data_profile_summary() -> i64 {
    let bind_core: Unit = data_profile_bind_core("FabricPlane");
    let window_offset: i64 = data_profile_window_offset("FabricPlane");
    let uplink_len: i64 = data_profile_uplink_len("FabricPlane");
    let downlink_len: i64 = data_profile_downlink_len("FabricPlane");
    let _ = bind_core;
    return window_offset + uplink_len + downlink_len;
  }

  fn main() {
    print(capture_data_profile_summary());
  }
}
"#;

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

fn binding_plan_from_source(source: &str) -> NustarBindingPlan {
    let artifacts = pipeline::compile_source(source).expect("source should compile");
    let declared_used_units = artifacts
        .ast
        .uses
        .iter()
        .map(|item| (item.domain.clone(), item.unit.clone()))
        .collect::<Vec<_>>();
    let declared_externs = artifacts
        .ast
        .externs
        .iter()
        .map(|item| (item.abi.clone(), item.name.clone()))
        .chain(
            artifacts
                .ast
                .extern_interfaces
                .iter()
                .flat_map(|interface| {
                    interface.methods.iter().map(move |method| {
                        (
                            method.abi.clone(),
                            format!("{}__{}", interface.name, method.name),
                        )
                    })
                }),
        )
        .collect::<Vec<_>>();

    let registry_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("nustar-packages");
    plan_bindings(
        &registry_root,
        &artifacts.nir,
        &artifacts.yir,
        &artifacts.ast.domain,
        &artifacts.ast.unit,
        &declared_used_units,
        &declared_externs,
    )
    .expect("binding plan should resolve")
}

fn cpu_manifest_with_host_target() -> NustarPackageManifest {
    NustarPackageManifest {
        manifest_schema: "nustar-manifest-v1".to_owned(),
        package_id: "official.cpu".to_owned(),
        domain_family: "cpu".to_owned(),
        frontend: "nustar-cpu".to_owned(),
        entry_crate: "crates/yir-domain-cpu".to_owned(),
        ast_entry: "cpu.ast.bootstrap.v1".to_owned(),
        nir_entry: "cpu.nir.bootstrap.v1".to_owned(),
        yir_lowering_entry: "cpu.yir.lowering.v1".to_owned(),
        part_verify_entry: "cpu.verify.partial.v1".to_owned(),
        ast_surface: vec!["cpu.mod-ast.v1".to_owned()],
        nir_surface: vec!["nir.cpu.surface.v1".to_owned()],
        yir_lowering: vec!["yir.cpu.lowering.v1".to_owned()],
        part_verify: vec!["verify.cpu.contract.v1".to_owned()],
        binary_extension: "nustar".to_owned(),
        package_layout: "single-envelope".to_owned(),
        machine_abi_policy: "exact-match".to_owned(),
        abi_profiles: vec!["cpu.host.v1".to_owned()],
        abi_capabilities: vec!["cpu.host.v1:op:cpu.*".to_owned()],
        abi_targets: vec![
            "cpu.host.v1:arch=host|os=host|object=host|calling=host|clang=host".to_owned(),
        ],
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
        clock_domain_id: "cpu.clock.host.v1".to_owned(),
        clock_kind: "host-monotonic".to_owned(),
        clock_epoch_kind: "host-epoch".to_owned(),
        clock_resolution: "cpu.tick_i64".to_owned(),
        clock_bridge_default: "global->monotonic:bridge".to_owned(),
        profiles: vec!["aot".to_owned()],
        resource_families: vec!["cpu".to_owned()],
        unit_types: vec!["Main".to_owned()],
        lowering_targets: vec!["llvm".to_owned()],
        ops: vec!["cpu.const".to_owned()],
    }
}

#[test]
fn host_ffi_registry_view_collects_signature_and_hash_registrations() {
    let mut manifest = cpu_manifest_with_host_target();
    manifest.abi_capabilities = vec![
            "c:ffi:i64(*)|ffi:i32(*)|ffi_symbol:host_i32_curve=i32(i32)|ffi_symbol_hash:host_hashed_curve=fnv1a64:38ca92f356fcb551".to_owned(),
        ];

    let view = HostFfiRegistryView::from_manifest(&manifest);

    assert!(view.has_abi("c"));
    assert_eq!(
        view.signature_families("c"),
        &["i32(*)".to_owned(), "i64(*)".to_owned()]
    );
    assert_eq!(
        view.symbol_registrations("c", "host_i32_curve"),
        &[HostFfiSymbolRegistration::Signature("i32(i32)".to_owned())]
    );
    assert_eq!(
        view.symbol_registrations("c", "host_hashed_curve"),
        &[HostFfiSymbolRegistration::Hash(
            "fnv1a64:38ca92f356fcb551".to_owned()
        )]
    );
    assert!(view.symbol_registrations("c", "missing").is_empty());
}

fn render_manifest_text(manifest: &NustarPackageManifest) -> String {
    fn render_array(values: &[String]) -> String {
        format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{value}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn render_optional_string(value: Option<&str>) -> String {
        match value {
            Some(value) => format!("\"{value}\""),
            None => "null".to_owned(),
        }
    }

    fn render_optional_array(value: Option<&[String]>) -> String {
        match value {
            Some(values) => render_array(values),
            None => "null".to_owned(),
        }
    }

    fn render_optional_bool(value: Option<bool>) -> String {
        match value {
            Some(true) => "true".to_owned(),
            Some(false) => "false".to_owned(),
            None => "null".to_owned(),
        }
    }

    format!(
        concat!(
            "manifest_schema = \"{}\"\n",
            "package_id = \"{}\"\n",
            "domain_family = \"{}\"\n",
            "frontend = \"{}\"\n",
            "entry_crate = \"{}\"\n",
            "ast_entry = \"{}\"\n",
            "nir_entry = \"{}\"\n",
            "yir_lowering_entry = \"{}\"\n",
            "part_verify_entry = \"{}\"\n",
            "ast_surface = {}\n",
            "nir_surface = {}\n",
            "yir_lowering = {}\n",
            "part_verify = {}\n",
            "binary_extension = \"{}\"\n",
            "package_layout = \"{}\"\n",
            "machine_abi_policy = \"{}\"\n",
            "abi_profiles = {}\n",
            "abi_capabilities = {}\n",
            "abi_targets = {}\n",
            "implementation_kinds = {}\n",
            "loader_entry = \"{}\"\n",
            "loader_abi = \"{}\"\n",
            "host_ffi_surface = {}\n",
            "host_ffi_abis = {}\n",
            "host_ffi_bridge = \"{}\"\n",
            "bridge_lane_policy = {}\n",
            "bridge_surface = {}\n",
            "bridge_emission_kind = {}\n",
            "bridge_entry = {}\n",
            "bridge_kind = {}\n",
            "bridge_scheduler_binding = {}\n",
            "backend_stub_kind = {}\n",
            "backend_submission_mode = {}\n",
            "backend_wake_policy = {}\n",
            "backend_transport_model = {}\n",
            "backend_request_shape = {}\n",
            "backend_response_shape = {}\n",
            "backend_dispatch_shape = {}\n",
            "backend_memory_binding = {}\n",
            "backend_resource_binding = {}\n",
            "backend_completion_model = {}\n",
            "phase_bind = {}\n",
            "phase_submit = {}\n",
            "phase_wait = {}\n",
            "phase_finalize = {}\n",
            "host_bridge_host_ffi_surface = {}\n",
            "host_bridge_handle_family = {}\n",
            "host_bridge_phase_order = {}\n",
            "host_bridge_phase_bind_inputs = {}\n",
            "host_bridge_phase_bind_outputs = {}\n",
            "host_bridge_phase_submit_inputs = {}\n",
            "host_bridge_phase_submit_outputs = {}\n",
            "host_bridge_phase_wait_inputs = {}\n",
            "host_bridge_phase_wait_outputs = {}\n",
            "host_bridge_phase_finalize_inputs = {}\n",
            "host_bridge_phase_finalize_outputs = {}\n",
            "host_bridge_phase_bind_wake = {}\n",
            "host_bridge_phase_submit_wake = {}\n",
            "host_bridge_phase_wait_wake = {}\n",
            "host_bridge_phase_finalize_wake = {}\n",
            "host_bridge_plan_begin = {}\n",
            "host_bridge_plan_end = {}\n",
            "support_surface = {}\n",
            "support_profile_slots = {}\n",
            "capability_tags = {}\n",
            "default_lanes = {}\n",
            "clock_domain_id = \"{}\"\n",
            "clock_kind = \"{}\"\n",
            "clock_epoch_kind = \"{}\"\n",
            "clock_resolution = \"{}\"\n",
            "clock_bridge_default = \"{}\"\n",
            "profiles = {}\n",
            "resource_families = {}\n",
            "unit_types = {}\n",
            "lowering_targets = {}\n",
            "ops = {}\n"
        ),
        manifest.manifest_schema,
        manifest.package_id,
        manifest.domain_family,
        manifest.frontend,
        manifest.entry_crate,
        manifest.ast_entry,
        manifest.nir_entry,
        manifest.yir_lowering_entry,
        manifest.part_verify_entry,
        render_array(&manifest.ast_surface),
        render_array(&manifest.nir_surface),
        render_array(&manifest.yir_lowering),
        render_array(&manifest.part_verify),
        manifest.binary_extension,
        manifest.package_layout,
        manifest.machine_abi_policy,
        render_array(&manifest.abi_profiles),
        render_array(&manifest.abi_capabilities),
        render_array(&manifest.abi_targets),
        render_array(&manifest.implementation_kinds),
        manifest.loader_entry,
        manifest.loader_abi,
        render_array(&manifest.host_ffi_surface),
        render_array(&manifest.host_ffi_abis),
        manifest.host_ffi_bridge,
        render_optional_string(manifest.bridge_lane_policy.as_deref()),
        render_optional_string(manifest.bridge_surface.as_deref()),
        render_optional_string(manifest.bridge_emission_kind.as_deref()),
        render_optional_string(manifest.bridge_entry.as_deref()),
        render_optional_string(manifest.bridge_kind.as_deref()),
        render_optional_string(manifest.bridge_scheduler_binding.as_deref()),
        render_optional_string(manifest.backend_stub_kind.as_deref()),
        render_optional_string(manifest.backend_submission_mode.as_deref()),
        render_optional_string(manifest.backend_wake_policy.as_deref()),
        render_optional_string(manifest.backend_transport_model.as_deref()),
        render_optional_string(manifest.backend_request_shape.as_deref()),
        render_optional_string(manifest.backend_response_shape.as_deref()),
        render_optional_string(manifest.backend_dispatch_shape.as_deref()),
        render_optional_string(manifest.backend_memory_binding.as_deref()),
        render_optional_string(manifest.backend_resource_binding.as_deref()),
        render_optional_string(manifest.backend_completion_model.as_deref()),
        render_optional_string(manifest.phase_bind.as_deref()),
        render_optional_string(manifest.phase_submit.as_deref()),
        render_optional_string(manifest.phase_wait.as_deref()),
        render_optional_string(manifest.phase_finalize.as_deref()),
        render_optional_array(manifest.host_bridge_host_ffi_surface.as_deref()),
        render_optional_array(manifest.host_bridge_handle_family.as_deref()),
        render_optional_array(manifest.host_bridge_phase_order.as_deref()),
        render_optional_array(manifest.host_bridge_phase_bind_inputs.as_deref()),
        render_optional_array(manifest.host_bridge_phase_bind_outputs.as_deref()),
        render_optional_array(manifest.host_bridge_phase_submit_inputs.as_deref()),
        render_optional_array(manifest.host_bridge_phase_submit_outputs.as_deref()),
        render_optional_array(manifest.host_bridge_phase_wait_inputs.as_deref()),
        render_optional_array(manifest.host_bridge_phase_wait_outputs.as_deref()),
        render_optional_array(manifest.host_bridge_phase_finalize_inputs.as_deref()),
        render_optional_array(manifest.host_bridge_phase_finalize_outputs.as_deref()),
        render_optional_string(manifest.host_bridge_phase_bind_wake.as_deref()),
        render_optional_string(manifest.host_bridge_phase_submit_wake.as_deref()),
        render_optional_string(manifest.host_bridge_phase_wait_wake.as_deref()),
        render_optional_string(manifest.host_bridge_phase_finalize_wake.as_deref()),
        render_optional_bool(manifest.host_bridge_plan_begin),
        render_optional_bool(manifest.host_bridge_plan_end),
        render_array(&manifest.support_surface),
        render_array(&manifest.support_profile_slots),
        render_array(&manifest.capability_tags),
        render_array(&manifest.default_lanes),
        manifest.clock_domain_id,
        manifest.clock_kind,
        manifest.clock_epoch_kind,
        manifest.clock_resolution,
        manifest.clock_bridge_default,
        render_array(&manifest.profiles),
        render_array(&manifest.resource_families),
        render_array(&manifest.unit_types),
        render_array(&manifest.lowering_targets),
        render_array(&manifest.ops),
    )
}

fn temp_registry_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuisc-{label}-{nanos}"));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn relative_checked_in_registry_root_resolves_to_workspace_path() {
    let root = resolve_registry_root(Path::new("nustar-packages"));

    assert!(root.is_absolute());
    assert!(root.join("cpu.toml").exists());
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
    assert_eq!(manifest.package_id, "official.cpu");
}

fn write_registry_fixture(
    root: &Path,
    entries: &[NustarPackageIndexEntry],
    manifests: &[NustarPackageManifest],
) {
    let mut index_text = String::new();
    for entry in entries {
        index_text.push_str("[[package]]\n");
        index_text.push_str(&format!("package_id = \"{}\"\n", entry.package_id));
        index_text.push_str(&format!("manifest = \"{}\"\n", entry.manifest));
        index_text.push_str(&format!("domain_family = \"{}\"\n\n", entry.domain_family));
    }
    fs::write(root.join(INDEX_FILE), index_text).unwrap();
    for (entry, manifest) in entries.iter().zip(manifests.iter()) {
        fs::write(root.join(&entry.manifest), render_manifest_text(manifest)).unwrap();
    }
}

#[test]
fn registered_abi_target_expands_host_adaptive_contract() {
    let manifest = cpu_manifest_with_host_target();
    let target = registered_abi_target(&manifest, "cpu.host.v1").unwrap();
    assert_eq!(target.machine_arch, host_arch());
    assert_eq!(target.machine_os, host_os());
    assert_eq!(target.object_format, host_object_format());
    assert_eq!(target.calling_abi, host_calling_abi());
    assert_eq!(target.clang_target, host_clang_target());
    assert!(target.host_adaptive);
}

#[test]
fn registered_abi_target_preserves_backend_family() {
    let mut manifest = cpu_manifest_with_host_target();
    manifest.abi_profiles = vec!["cpu.backend.v1".to_owned()];
    manifest.abi_capabilities = vec!["cpu.backend.v1:op:cpu.*".to_owned()];
    manifest.abi_targets = vec![
            "cpu.backend.v1:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin|backend=metal|vendor=apple|device=apple-silicon-gpu".to_owned(),
        ];
    let target = registered_abi_target(&manifest, "cpu.backend.v1").unwrap();
    assert_eq!(target.backend_family.as_deref(), Some("metal"));
    assert_eq!(target.vendor.as_deref(), Some("apple"));
    assert_eq!(target.device_class.as_deref(), Some("apple-silicon-gpu"));
    assert!(!target.host_adaptive);
}

#[test]
fn network_manifest_skeleton_is_registered() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    assert_eq!(manifest.package_id, "official.network");
    assert_eq!(manifest.clock_domain_id, "network.clock.io.v1");
    assert_eq!(manifest.clock_kind, "io-monotonic");
    assert!(manifest
        .support_surface
        .contains(&"network.profile.bind-core.v1".to_owned()));
    assert!(manifest
        .support_surface
        .contains(&"network.profile.connect.v1".to_owned()));
    assert!(manifest
        .support_surface
        .contains(&"network.profile.stream-window.v1".to_owned()));
    assert!(manifest
        .support_surface
        .contains(&"network.profile.transport.v1".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"bind_core".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"endpoint_kind".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"transport_family".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"retry_budget".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"stream_window".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"protocol_kind".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"network.send=tx".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"network.recv=rx".to_owned()));
}

#[test]
fn shader_manifest_registers_texture_sampling_contracts() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();
    assert_eq!(manifest.package_id, "official.shader");
    assert!(manifest
        .support_surface
        .contains(&"shader.profile.texture.v1".to_owned()));
    assert!(manifest
        .support_surface
        .contains(&"shader.profile.sampler.v1".to_owned()));
    assert!(manifest
        .support_surface
        .contains(&"shader.profile.bind-set.v1".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"texture_format".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"sampler_kind".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"bind_set".to_owned()));
    assert!(manifest
        .capability_tags
        .contains(&"texture-sampling".to_owned()));
    assert!(manifest
        .capability_tags
        .contains(&"bind-group-layout".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"shader.texture2d=resource".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"shader.sample_uv=render".to_owned()));
}

#[test]
fn kernel_manifest_registers_tensor_axis_contracts() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
    assert_eq!(manifest.package_id, "official.kernel");
    assert!(manifest
        .support_surface
        .contains(&"kernel.profile.tensor-shape.v1".to_owned()));
    assert!(manifest
        .support_surface
        .contains(&"kernel.profile.tensor-reduce.v1".to_owned()));
    assert!(manifest
        .support_surface
        .contains(&"kernel.profile.tensor-selection.v1".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"tensor_rows".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"reduce_axis".to_owned()));
    assert!(manifest
        .support_profile_slots
        .contains(&"result_buffer".to_owned()));
    assert!(manifest
        .capability_tags
        .contains(&"axis-reduction".to_owned()));
    assert!(manifest
        .capability_tags
        .contains(&"tensor-selection".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"kernel.reduce_sum_axis=reduce".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"kernel.topk_axis=select".to_owned()));
}

#[test]
fn cpu_manifest_contract_is_registered() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
    assert_eq!(manifest.package_id, "official.cpu");
    assert_eq!(manifest.loader_abi, "nustar-loader-v1");
    assert_eq!(manifest.loader_entry, "nustar.bootstrap.v1");
    assert_eq!(manifest.machine_abi_policy, "exact-match");
    assert_eq!(manifest.clock_domain_id, "cpu.clock.host.v1");
    assert_eq!(manifest.clock_kind, "host-monotonic");
    assert_eq!(manifest.clock_bridge_default, "global->monotonic:bridge");
    assert!(manifest
        .host_ffi_surface
        .contains(&"cpu.host-ffi.nurs.v1".to_owned()));
    assert!(manifest
        .host_ffi_surface
        .contains(&"cpu.host-ffi.c-bridge.v1".to_owned()));
    assert!(manifest.host_ffi_abis.contains(&"nurs".to_owned()));
    assert!(manifest.host_ffi_abis.contains(&"c".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"cpu.window=main".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"cpu.alloc_node=mem".to_owned()));
    assert!(manifest
        .default_lanes
        .contains(&"cpu.instantiate_unit=main".to_owned()));
    assert!(manifest
        .abi_profiles
        .contains(&"cpu.arm64.apple_aapcs64".to_owned()));
}

#[test]
fn scheduler_summary_uses_manifest_clock_and_domain_samples() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    let summary = scheduler_summary(&manifest);
    assert_eq!(summary.clock.domain_id, "network.clock.io.v1");
    assert_eq!(summary.clock.kind, "io-monotonic");
    assert_eq!(
            summary.sample_navigation.as_deref(),
            Some(
                "result_ladder -> transport_split_ladder -> transport_summary_ladder -> summary_classes"
            )
        );
    assert!(summary
        .result_samples
        .as_deref()
        .unwrap_or_default()
        .contains("network_result_profile_demo"));
    assert!(summary
        .transport_samples
        .as_deref()
        .unwrap_or_default()
        .contains("network_transport_result_policy_split_demo"));
}

#[test]
fn capability_summary_tracks_support_and_clock_contract() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    let summary = capability_summary(&manifest);
    assert!(summary
        .support_surface
        .contains(&"network.profile.transport.v1".to_owned()));
    assert!(summary
        .support_profile_slots
        .contains(&"protocol_kind".to_owned()));
    assert!(summary.capability_tags.contains(&"io-reactor".to_owned()));
    assert!(summary
        .capability_tags
        .contains(&"protocol-framing".to_owned()));
    assert!(summary
        .default_lanes
        .contains(&"network.send=tx".to_owned()));
    assert_eq!(summary.clock.domain_id, "network.clock.io.v1");
    assert_eq!(summary.clock.bridge_default, "global->io:bridge");
}

#[test]
fn execution_summary_derives_minimum_execution_skeleton() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
    let summary = execution_summary(&manifest);
    assert_eq!(summary.skeleton_version, "nustar-execution-skeleton-v1");
    assert_eq!(summary.function_kind, "function-node");
    assert_eq!(summary.graph_kind, "function-graph");
    assert_eq!(summary.execution_domain, "kernel");
    assert_eq!(summary.default_time_mode, "logical");
    assert_eq!(summary.contract_family, "nustar.kernel");
    assert!(summary.lowering_targets.contains(&"coreml".to_owned()));
}

#[test]
fn domain_build_contract_summary_prefers_manifest_registered_bridge_fields() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    assert_eq!(
        manifest.bridge_lane_policy.as_deref(),
        Some("dispatch-lanes.io-bound")
    );
    assert_eq!(
        manifest.bridge_surface.as_deref(),
        Some("host-ffi.bridge.network")
    );
    assert_eq!(
        manifest.bridge_entry.as_deref(),
        Some("nuis.network.bridge.dispatch.v1")
    );
    assert_eq!(
        manifest.bridge_scheduler_binding.as_deref(),
        Some("network-poll-bridge")
    );
    assert_eq!(
        manifest.bridge_emission_kind.as_deref(),
        Some("sidecar-plan")
    );
    assert_eq!(
        manifest.bridge_kind.as_deref(),
        Some("managed-lifecycle-bridge")
    );
    let summary = domain_build_contract_summary(&manifest);
    assert_eq!(summary.lowering.lane_policy, "dispatch-lanes.io-bound");
    assert_eq!(summary.lowering.bridge_surface, "host-ffi.bridge.network");
    assert_eq!(summary.lowering.emission_kind, "sidecar-plan");
    assert_eq!(summary.backend.stub_kind, "network-host-bridge");
    assert_eq!(summary.backend.submission_mode, "request-response");
    assert_eq!(summary.backend.wake_policy, "io-ready");
    assert_eq!(
        summary.backend.transport_model.as_deref(),
        Some("client-session")
    );
    assert_eq!(summary.bridge.scheduler_binding, "network-poll-bridge");
    assert_eq!(summary.bridge.phase_submit, "packet-write-dispatch");
    assert_eq!(summary.bridge.phase_wait, "callback-or-read-ready");
    assert_eq!(summary.bridge.bridge_kind, "managed-lifecycle-bridge");
    assert_eq!(summary.host_bridge.host_ffi_surface, "socket,urlsession");
    assert_eq!(
        summary.host_bridge.handle_family,
        "network.request,network.response"
    );
    assert_eq!(
        summary.host_bridge.phase_bind_inputs,
        vec![
            "request.packet".to_owned(),
            "bridge.config".to_owned(),
            "host.session".to_owned()
        ]
    );
    assert_eq!(
        summary.host_bridge.phase_submit_outputs,
        vec!["inflight.request".to_owned(), "poll.token".to_owned()]
    );
    assert_eq!(summary.host_bridge.phase_wait_wake, "io-ready");
    assert!(summary.host_bridge.bridge_plan_begin);
    assert!(summary.host_bridge.bridge_plan_end);
}

#[test]
fn domain_contract_collects_registered_runtime_and_loader_facts() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    let contract = domain_contract(&manifest);
    assert_eq!(contract.contract_schema, NUSTAR_DOMAIN_CONTRACT_SCHEMA);
    assert_eq!(contract.contract_status, "complete");
    assert!(contract.missing_contract_groups.is_empty());
    assert!(contract
        .required_contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY.to_owned()));
    assert!(contract
        .required_contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME.to_owned()));
    assert!(contract
        .contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY.to_owned()));
    assert!(contract
        .contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER.to_owned()));
    assert!(contract
        .contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_ABI.to_owned()));
    assert!(contract
        .contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME.to_owned()));
    assert!(contract
        .contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION.to_owned()));
    assert!(contract
        .contract_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER.to_owned()));
    assert!(contract
        .extension_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned()));
    assert_eq!(contract.package_id, "official.network");
    assert_eq!(contract.domain_family, "network");
    assert_eq!(contract.frontend, "nustar-network");
    assert_eq!(contract.loader_abi, "nustar-loader-v1");
    assert_eq!(contract.loader_entry, "nustar.bootstrap.v1");
    assert_eq!(contract.machine_abi_policy, "exact-match");
    assert!(contract
        .abi_profiles
        .contains(&"network.socket.v1".to_owned()));
    assert!(contract
        .capability
        .support_surface
        .contains(&"network.profile.transport.v1".to_owned()));
    assert!(contract
        .capability
        .capability_tags
        .contains(&"socket-transport".to_owned()));
    assert_eq!(contract.execution.execution_domain, "network");
    assert_eq!(contract.execution.contract_family, "nustar.network");
    assert_eq!(contract.scheduler.clock.domain_id, "network.clock.io.v1");
    assert!(contract
        .std_net
        .recipe_samples
        .as_deref()
        .unwrap_or_default()
        .contains("net_http_client_recipe"));
    let json = domain_contract_json(&contract);
    assert!(json.contains("\"contract_status\":\"complete\""));
    assert!(json.contains("\"contract_complete\":true"));
    assert!(json.contains("\"required_contract_groups\":[\"package_identity\""));
    assert!(json.contains("\"missing_contract_groups\":[]"));
    assert!(json.contains("\"status\":\"complete\""));
    assert!(json.contains("\"complete\":true"));
    assert!(json.contains("\"execution_skeleton_version\":\"nustar-execution-skeleton-v1\""));
    assert!(json.contains("\"execution_contract_family\":\"nustar.network\""));
    assert!(json.contains("\"capability_tags\":[\"io-reactor\""));
}

#[test]
fn std_net_summary_is_owned_by_registry() {
    let summary = std_net_summary("network");
    assert_eq!(
            summary.sample_navigation.as_deref(),
            Some(
                "profile_core -> transport_edge -> syscall_edge -> socket_edge -> control_edge -> protocol_edge -> http_edge -> result_spine -> task_spine -> session"
            )
        );
    assert!(summary
        .recipe_samples
        .as_deref()
        .unwrap_or_default()
        .contains("net_http_client_recipe"));
}

#[test]
fn load_registered_domains_covers_all_indexed_nustar_modules() {
    let registrations = load_registered_domains(Path::new("nustar-packages")).unwrap();
    let domains = registrations
        .iter()
        .map(|item| item.domain_family.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        domains,
        vec!["cpu", "cpu", "data", "kernel", "network", "shader"]
    );
    let cpu_packages = registrations
        .iter()
        .filter(|item| item.domain_family == "cpu")
        .map(|item| item.package_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(cpu_packages, vec!["official.cpu", "official.cpu.aarch64"]);
    let network = registrations
        .iter()
        .find(|item| item.domain_family == "network")
        .unwrap();
    assert!(network
        .manifest_path
        .ends_with("nustar-packages/network.toml"));
    assert_eq!(
        network.contract.contract_schema,
        NUSTAR_DOMAIN_CONTRACT_SCHEMA
    );
    assert!(network
        .contract
        .extension_groups
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned()));
    assert!(!network.ops.is_empty());
}

#[test]
fn aarch64_cpu_nustar_is_independent_package_for_cpu_domain() {
    let generic_cpu = load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
    assert_eq!(generic_cpu.package_id, "official.cpu");
    assert!(generic_cpu
        .abi_profiles
        .contains(&"cpu.x86_64.sysv64".to_owned()));

    let aarch64_cpu = load_manifest(Path::new("nustar-packages"), "official.cpu.aarch64").unwrap();
    assert_eq!(aarch64_cpu.domain_family, "cpu");
    assert_eq!(aarch64_cpu.package_id, "official.cpu.aarch64");
    assert!(aarch64_cpu
        .capability_tags
        .contains(&"formal-verification-ready".to_owned()));
    assert!(aarch64_cpu
        .capability_tags
        .contains(&"aarch64-only".to_owned()));
    assert!(aarch64_cpu
        .part_verify
        .contains(&"verify.cpu.aarch64.call-frame.v1".to_owned()));
    assert!(aarch64_cpu
        .abi_profiles
        .iter()
        .all(|abi| abi.starts_with("cpu.arm64.")));
    assert!(aarch64_cpu
        .lowering_targets
        .contains(&"aarch64-proof-skeleton".to_owned()));
}

#[test]
fn validate_registered_domains_accepts_current_mainline_registry() {
    let issues = validate_registered_domains(Path::new("nustar-packages")).unwrap();
    assert!(issues.is_empty(), "unexpected registry issues: {issues:?}");
    ensure_registered_domains_valid(Path::new("nustar-packages")).unwrap();
}

#[test]
fn validate_registered_domains_allows_duplicate_domain_but_rejects_bad_lane_target() {
    let root = temp_registry_root("registry-duplicate-domain");
    let cpu = cpu_manifest_with_host_target();
    let mut network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    network.default_lanes.push("network.ghost=rx".to_owned());
    let entries = vec![
        NustarPackageIndexEntry {
            package_id: cpu.package_id.clone(),
            manifest: "cpu.toml".to_owned(),
            domain_family: cpu.domain_family.clone(),
        },
        NustarPackageIndexEntry {
            package_id: network.package_id.clone(),
            manifest: "network.toml".to_owned(),
            domain_family: cpu.domain_family.clone(),
        },
    ];
    write_registry_fixture(&root, &entries, &[cpu, network]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::DomainFamilyMismatch));
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::LaneContractMismatch));
    let error = ensure_registered_domains_valid(&root).unwrap_err();
    assert!(error.contains("NRV005"));
    assert!(error.contains("NRV010"));
}

#[test]
fn validate_registered_domains_rejects_loader_and_op_contract_mismatch() {
    let root = temp_registry_root("registry-loader-op");
    let mut cpu = cpu_manifest_with_host_target();
    cpu.loader_abi = "wrong-loader".to_owned();
    cpu.ops.push("shader.draw".to_owned());
    let entries = vec![NustarPackageIndexEntry {
        package_id: cpu.package_id.clone(),
        manifest: "cpu.toml".to_owned(),
        domain_family: cpu.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[cpu]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::LoaderContractMismatch));
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::OpContractMismatch));
}

#[test]
fn validate_registered_domains_rejects_shader_backend_without_lowering_target() {
    let root = temp_registry_root("registry-shader-backend");
    let mut shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();
    shader
        .lowering_targets
        .retain(|target| target != "cpu-fallback");
    let entries = vec![NustarPackageIndexEntry {
        package_id: shader.package_id.clone(),
        manifest: "shader.toml".to_owned(),
        domain_family: shader.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[shader]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("cpu-fallback")
    }));
}

#[test]
fn validate_registered_domains_rejects_shader_missing_texture_profile_slot() {
    let root = temp_registry_root("registry-shader-texture-slot");
    let mut shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();
    shader
        .support_profile_slots
        .retain(|slot| slot != "texture_format");
    let entries = vec![NustarPackageIndexEntry {
        package_id: shader.package_id.clone(),
        manifest: "shader.toml".to_owned(),
        domain_family: shader.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[shader]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("texture_format")
    }));
}

#[test]
fn validate_registered_domains_rejects_kernel_missing_profile_slot() {
    let root = temp_registry_root("registry-kernel-slot");
    let mut kernel = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
    kernel
        .support_profile_slots
        .retain(|slot| slot != "batch_lanes");
    let entries = vec![NustarPackageIndexEntry {
        package_id: kernel.package_id.clone(),
        manifest: "kernel.toml".to_owned(),
        domain_family: kernel.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[kernel]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("batch_lanes")
    }));
}

#[test]
fn validate_registered_domains_rejects_network_missing_socket_lowering_target() {
    let root = temp_registry_root("registry-network-lowering");
    let mut network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    network
        .lowering_targets
        .retain(|target| target != "socket-abi");
    let entries = vec![NustarPackageIndexEntry {
        package_id: network.package_id.clone(),
        manifest: "network.toml".to_owned(),
        domain_family: network.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[network]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("socket-abi")
    }));
}

#[test]
fn validate_registered_domains_rejects_incomplete_host_bridge_contract() {
    let root = temp_registry_root("registry-host-bridge-missing");
    let mut network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    network.host_bridge_phase_wait_wake = None;
    let entries = vec![NustarPackageIndexEntry {
        package_id: network.package_id.clone(),
        manifest: "network.toml".to_owned(),
        domain_family: network.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[network]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("host_bridge_phase_wait_wake")
    }));
}

#[test]
fn validate_registered_domains_rejects_invalid_host_bridge_phase_order() {
    let root = temp_registry_root("registry-host-bridge-order");
    let mut kernel = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
    kernel.host_bridge_phase_order = Some(vec![
        "bind".to_owned(),
        "wait".to_owned(),
        "submit".to_owned(),
        "finalize".to_owned(),
    ]);
    let entries = vec![NustarPackageIndexEntry {
        package_id: kernel.package_id.clone(),
        manifest: "kernel.toml".to_owned(),
        domain_family: kernel.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[kernel]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("phase_order")
    }));
}

#[test]
fn ensure_project_domain_registry_valid_accepts_registered_abi() {
    let plan = test_project_plan("network", "network.socket.macos.arm64.v1");
    let checks = validate_project_domain_registry(&plan);
    assert!(checks.iter().all(|check| check.issues.is_empty()));
    let network = checks
        .iter()
        .find(|check| check.domain == "network")
        .unwrap();
    assert_eq!(network.issue_count(), 0);
    assert!(network.summary_line().contains(": ok"));
    assert!(network.abi_registered);
    ensure_project_domain_registry_valid(&plan).unwrap();
}

#[test]
fn ensure_project_domain_registry_valid_rejects_unknown_abi() {
    let plan = test_project_plan("network", "network.socket.unknown.v1");
    let checks = validate_project_domain_registry(&plan);
    let network = checks
        .iter()
        .find(|check| check.domain == "network")
        .unwrap();
    assert!(network
        .issues
        .iter()
        .any(|issue| issue.kind == ProjectDomainRegistryIssueKind::AbiNotRegistered));
    assert!(network
        .issues
        .iter()
        .any(|issue| issue.kind.code() == "NRG003"));
    assert!(network.summary_line().contains("NRG003 abi_not_registered"));
    let error = ensure_project_domain_registry_valid(&plan).unwrap_err();
    assert!(error.contains("project domain registry validation failed"));
    assert!(error.contains("network"));
    assert!(error.contains("network.socket.unknown.v1"));
    assert!(error.contains("NRG003"));
    assert!(error.contains("abi_not_registered"));
}

#[test]
fn binding_plan_carries_execution_skeleton_summary() {
    let plan = binding_plan_from_source(
        r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    print(0);
  }
}
"#,
    );
    let shader = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should exist");
    assert_eq!(
        shader.execution.skeleton_version,
        "nustar-execution-skeleton-v1"
    );
    assert_eq!(shader.execution.function_kind, "function-node");
    assert_eq!(shader.execution.graph_kind, "function-graph");
    assert_eq!(shader.execution.execution_domain, "shader");
    assert_eq!(shader.execution.contract_family, "nustar.shader");
    assert!(shader
        .execution
        .lowering_targets
        .contains(&"metal".to_owned()));
}

#[test]
fn project_domain_registry_check_renderers_expose_codes_and_issue_counts() {
    let plan = test_project_plan("network", "network.socket.unknown.v1");
    let check = validate_project_domain_registry(&plan)
        .into_iter()
        .find(|check| check.domain == "network")
        .expect("network check");
    let json = project_domain_registry_check_json(&check);
    assert!(json.contains("\"domain\":\"network\""));
    assert!(json.contains("\"code\":\"NRG003\""));
    assert!(json.contains("\"kind\":\"abi_not_registered\""));
    let lines = render_project_domain_registry_check_lines(&check);
    assert!(!lines.is_empty());
    assert!(lines[0].contains("issues=1"));
    assert!(lines
        .iter()
        .any(|line| line.contains("NRG003 abi_not_registered")));
    let mut written = String::new();
    write_project_domain_registry_check_lines(&mut written, &check).unwrap();
    assert_eq!(written.lines().collect::<Vec<_>>(), lines);
}

#[test]
fn registered_abi_target_accepts_darwin_x86_64_domain_profiles() {
    let network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    let data = load_manifest_for_domain(Path::new("nustar-packages"), "data").unwrap();
    let shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();

    let network_target = registered_abi_target(&network, "network.socket.macos.x86_64.v1").unwrap();
    assert_eq!(network_target.machine_arch, "x86_64");
    assert_eq!(network_target.machine_os, "darwin");
    assert_eq!(network_target.clang_target, "x86_64-apple-darwin");

    let data_target = registered_abi_target(&data, "data.fabric.macos.x86_64.v1").unwrap();
    assert_eq!(data_target.machine_arch, "x86_64");
    assert_eq!(data_target.machine_os, "darwin");
    assert_eq!(data_target.clang_target, "x86_64-apple-darwin");

    let shader_target = registered_abi_target(&shader, "shader.metal.x86_64.msl2_4").unwrap();
    assert_eq!(shader_target.machine_arch, "x86_64");
    assert_eq!(shader_target.machine_os, "darwin");
    assert_eq!(shader_target.clang_target, "x86_64-apple-darwin");
    assert_eq!(shader_target.backend_family.as_deref(), Some("metal"));
    assert_eq!(shader_target.vendor.as_deref(), Some("apple"));
    assert_eq!(
        shader_target.device_class.as_deref(),
        Some("mac-discrete-or-integrated-gpu")
    );
}

#[test]
fn network_binding_plan_detects_profile_surfaces_and_slots() {
    let source = r#"
use network NetworkUnit;

mod cpu Main {
  fn capture_network_profile_summary() -> i64 {
    let bind_core: i64 = network_profile_bind_core("NetworkUnit");
    let endpoint_kind: i64 = network_profile_endpoint_kind("NetworkUnit");
    let timeout_budget: i64 = network_profile_timeout_budget("NetworkUnit");
    let retry_budget: i64 = network_profile_retry_budget("NetworkUnit");
    let stream_window: i64 = network_profile_stream_window("NetworkUnit");
    let recv_window: i64 = network_profile_recv_window("NetworkUnit");
    let send_window: i64 = network_profile_send_window("NetworkUnit");
    return bind_core + endpoint_kind + timeout_budget + retry_budget + stream_window + recv_window + send_window;
  }

  fn main() {
    print(capture_network_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);

    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "network")
        .expect("network binding should be present");

    for surface in [
        "network.profile.bind-core.v1",
        "network.profile.endpoint-kind.v1",
        "network.profile.timeout.v1",
        "network.profile.retry.v1",
        "network.profile.stream-window.v1",
        "network.profile.recv.v1",
        "network.profile.send.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched network surface `{surface}`"
        );
    }

    for slot in [
        "bind_core",
        "endpoint_kind",
        "timeout_budget",
        "retry_budget",
        "stream_window",
        "recv_window",
        "send_window",
    ] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched network slot `{slot}`"
        );
        assert!(
            binding
                .covered_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected covered network slot `{slot}`"
        );
    }
    assert!(binding.capability_tags.contains(&"async-bridge".to_owned()));
}

#[test]
fn data_binding_plan_detects_profile_surfaces_and_slots() {
    let plan = binding_plan_from_source(DATA_BINDING_SOURCE);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "data")
        .expect("data binding should be present");
    for surface in ["data.profile.bind-core.v1", "data.profile.window-layout.v1"] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched data surface `{surface}`"
        );
    }
    for slot in ["bind_core", "window_offset", "uplink_len", "downlink_len"] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched data slot `{slot}`"
        );
    }
}

#[test]
fn kernel_binding_plan_detects_profile_surfaces_and_slots() {
    let source = r#"
use kernel KernelUnit;

mod cpu Main {
  fn capture_kernel_profile_summary() -> i64 {
    let bind_core: i64 = kernel_profile_bind_core("KernelUnit");
    let queue_depth: i64 = kernel_profile_queue_depth("KernelUnit");
    let batch_lanes: i64 = kernel_profile_batch_lanes("KernelUnit");
    return bind_core + queue_depth + batch_lanes;
  }

  fn main() {
    print(capture_kernel_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "kernel")
        .expect("kernel binding should be present");
    for surface in [
        "kernel.profile.bind-core.v1",
        "kernel.profile.queue-depth.v1",
        "kernel.profile.batch-lanes.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched kernel surface `{surface}`"
        );
    }
    for slot in ["bind_core", "queue_depth", "batch_lanes"] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched kernel slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_profile_surfaces_and_slots() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn capture_shader_profile_summary() -> i64 {
    let target: Target = shader_profile_target("SurfaceShader");
    let viewport: Viewport = shader_profile_viewport("SurfaceShader");
    let pipeline: Pipeline = shader_profile_pipeline("SurfaceShader");
    let vertex_count: i64 = shader_profile_vertex_count("SurfaceShader");
    let instance_count: i64 = shader_profile_instance_count("SurfaceShader");
    let _ = target;
    let _ = viewport;
    let _ = pipeline;
    return vertex_count + instance_count;
  }

  fn main() {
    print(capture_shader_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should be present");
    for surface in [
        "shader.profile.target.v1",
        "shader.profile.viewport.v1",
        "shader.profile.pipeline.v1",
        "shader.profile.draw-budget.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched shader surface `{surface}`"
        );
    }
    for slot in [
        "target",
        "viewport",
        "pipeline",
        "vertex_count",
        "instance_count",
    ] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_nova_packet_surface_and_covered_slots() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let _ = packet;
    print(0);
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should be present");
    assert!(binding
        .matched_support_surface
        .iter()
        .any(|surface| surface == "shader.profile.packet.nova.v1"));
    for slot in [
        "slider_color_slot",
        "slider_speed_slot",
        "slider_radius_slot",
        "header_accent_slot",
        "toggle_live_slot",
        "focus_slot",
    ] {
        assert!(
            binding
                .covered_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected covered shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_nova_profile_slot_accessors() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn capture_shader_nova_profile_summary() -> i64 {
    let slider_color: i64 = shader_profile_slider_color_slot("SurfaceShader");
    let slider_speed: i64 = shader_profile_slider_speed_slot("SurfaceShader");
    let slider_radius: i64 = shader_profile_slider_radius_slot("SurfaceShader");
    let header_accent: i64 = shader_profile_header_accent_slot("SurfaceShader");
    let toggle_live: i64 = shader_profile_toggle_live_slot("SurfaceShader");
    let focus: i64 = shader_profile_focus_slot("SurfaceShader");
    return slider_color + slider_speed + slider_radius + header_accent + toggle_live + focus;
  }

  fn main() {
    print(capture_shader_nova_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should be present");
    for surface in [
        "shader.profile.packet-slots.v1",
        "shader.profile.packet.nova.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched shader surface `{surface}`"
        );
    }
    for slot in [
        "slider_color_slot",
        "slider_speed_slot",
        "slider_radius_slot",
        "header_accent_slot",
        "toggle_live_slot",
        "focus_slot",
    ] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_packet_binding_profile_contract_surface() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let binding: Binding = shader_packet_uniform_binding(4, packet);
    print(binding);
  }
}
"#;
    let nir = crate::frontend::parse_nuis_module(source).expect("source should lower to nir");
    let (matched_support_surface, matched_support_profile_slots) =
        detect_matched_support_usage(&nir, "shader");
    let covered_support_profile_slots = covered_profile_slots(
        "shader",
        &matched_support_surface,
        &matched_support_profile_slots,
    );
    assert!(matched_support_surface
        .iter()
        .any(|surface| surface == "shader.profile.packet.nova.v1"));
    for slot in [
        "slider_color_slot",
        "slider_speed_slot",
        "slider_radius_slot",
        "header_accent_slot",
        "toggle_live_slot",
        "focus_slot",
    ] {
        assert!(
            covered_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected covered shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_collects_packet_binding_resource_hints() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let binding: Binding = shader_packet_uniform_binding(4, packet);
    let pipeline: Pipeline = shader_profile_pipeline("SurfaceShader");
    let bindings: BindingSet = shader_bind_set(pipeline, binding);
    print(bindings);
  }
}
"#;
    let nir = crate::frontend::parse_nuis_module(source).expect("source should lower to nir");
    let mut resources = BTreeSet::new();
    collect_resource_usage_hints(&nir, "shader", &mut resources);
    for resource in [
        "shader.binding.uniform_binding",
        "shader.binding.layout.std140",
        "shader.binding.contract.shader.profile.packet.nova.v1",
        "shader.binding.set",
    ] {
        assert!(
            resources.iter().any(|candidate| candidate == resource),
            "expected matched shader resource hint `{resource}`"
        );
    }
}
