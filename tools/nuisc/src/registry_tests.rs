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

#[test]
fn string_array_parser_accepts_multiline_manifest_arrays() {
    let values = parse_optional_string_array(
        r#"code_assets = [
          "asset.one",
          "asset.two",
        ]
        profiles = ["aot"]"#,
        "code_assets",
    )
    .expect("multiline array should parse");

    assert_eq!(values, vec!["asset.one", "asset.two"]);
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
        provider_bundles: Vec::new(),
        code_assets: Vec::new(),
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
            "provider_bundles = {}\n",
            "code_assets = {}\n",
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
        render_array(&manifest.provider_bundles),
        render_array(&manifest.code_assets),
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
    assert!(summary.lowering_targets.contains(&"cuda".to_owned()));
    assert!(manifest
        .abi_profiles
        .contains(&"kernel.cuda.ptx8_0.v1".to_owned()));
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
        .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_DISPATCH_READINESS.to_owned()));
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
    assert_eq!(contract.dispatch_readiness.status, "ready");
    assert!(contract.dispatch_readiness.missing_signals.is_empty());
    assert!(contract
        .dispatch_readiness
        .required_signals
        .contains(&"bridge_entry".to_owned()));
    assert!(contract.dispatch_readiness.dispatch_bridge_materialized);
    assert!(contract.dispatch_readiness.execution_readiness_materialized);
    assert_eq!(
        contract.dispatch_readiness.bridge_entry,
        "nuis.network.bridge.dispatch.v1"
    );
    assert_eq!(
        contract.dispatch_readiness.lifecycle_phase_order,
        vec![
            "bind".to_owned(),
            "submit".to_owned(),
            "wait".to_owned(),
            "finalize".to_owned()
        ]
    );
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
    assert!(json.contains("\"dispatch_readiness_status\":\"ready\""));
    assert!(json.contains("\"dispatch_bridge_materialized\":true"));
    assert!(json.contains("\"execution_readiness_materialized\":true"));
    assert!(json.contains("\"dispatch_bridge_entry\":\"nuis.network.bridge.dispatch.v1\""));
    assert!(json.contains("\"dispatch_readiness_contract\""));
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
fn kernel_cuda_abi_resolves_through_registered_contract_only() {
    let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
    let target = registered_abi_target(&manifest, "kernel.cuda.ptx8_0.v1").unwrap();

    assert_eq!(target.machine_arch, "x86_64");
    assert_eq!(target.machine_os, "linux");
    assert_eq!(target.object_format, "elf");
    assert_eq!(target.calling_abi, "sysv64");
    assert_eq!(target.backend_family.as_deref(), Some("cuda"));
    assert_eq!(target.vendor.as_deref(), Some("nvidia"));
    assert_eq!(target.device_class.as_deref(), Some("nvidia-gpu"));
    assert_eq!(
        crate::project::selected_lowering_target_for_registered_abi_target(
            "kernel",
            &target,
            &manifest.lowering_targets,
        )
        .as_deref(),
        Some("cuda.nvidia-gpu")
    );
}

#[path = "registry_validation_tests.rs"]
mod validation_tests;
