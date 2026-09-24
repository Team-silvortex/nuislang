use super::*;

#[test]
fn build_report_json_exposes_real_heterogeneous_runtime_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_profile_demo");
    let output_dir = temp_dir("build_report_shader_profile_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"domain_units_count\":2"));
    assert!(json.contains("\"heterogeneous_domain_count\":1"));
    assert!(json.contains("\"domain_family\":\"shader\""));
    assert!(json.contains("\"packaging_role\":\"hetero-contract\""));
    assert!(json.contains("\"artifact_payload_format\":\"ndpb-v2\""));
    assert!(json.contains("\"bridge_registry_units\":1"));
    assert!(json.contains("\"bridge_registry_checked\":1"));
    assert!(json.contains("\"host_bridge_plan_units\":1"));
    assert!(json.contains("\"domain_payload_blobs_checked\":1"));
    assert!(json.contains("\"domain_payload_bridge_plans_checked\":1"));
    assert!(json.contains("\"domain_bridge_stubs_checked\":1"));
    assert!(json.contains("\"link_plan_domain_units\":2"));
    assert!(json.contains("\"runtime_execution_attempted\":true"));
    assert!(json.contains("\"runtime_execution_ok\":true"));
    assert!(json.contains("\"runtime_execution_domains\":1"));
    assert!(json.contains("\"runtime_execution_plan_phases\":"));
    assert!(json.contains("\"runtime_execution_trace_events\":"));
    assert!(json.contains("\"runtime_payload_backed_heterogeneous_units\":1"));
    assert!(json.contains("\"runtime_cpu_fallback_units\":"));
    assert!(json.contains("\"runtime_host_consumable_units\":"));
}

#[test]
fn build_report_json_exposes_host_cpu_fallback_runtime_events() {
    let project_root = write_temp_project_fixture(
        "shader_cpu_fallback_runtime_demo",
        r#"
name = "shader_cpu_fallback_runtime_demo"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns", "surface_shader.ns"]
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "shader=shader.render.cpu-fallback.v1",
]
"#
        .trim_start(),
        r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let vertex_budget: i64 = shader_profile_vertex_count("SurfaceShader");
    let instance_budget: i64 = shader_profile_instance_count("SurfaceShader");
    let packet_tag: i64 = shader_profile_packet_tag("SurfaceShader");
    let swapchain: Target = shader_profile_target("SurfaceShader");
    print(vertex_budget + instance_budget + packet_tag);
  }
}
"#,
    );
    fs::write(
        project_root.join("surface_shader.ns"),
        r#"
mod shader SurfaceShader {
  fn profile() {
    const vertex_count: i64 = 4;
    const instance_count: i64 = 1;
    const packet_tag: i64 = 17;

    let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
    let profile_view: Viewport = shader_viewport(160, 120);
    let profile_pipe: Pipeline = shader_pipeline("cpu_fallback_surface", "triangle_strip");
  }
}
"#,
    )
    .expect("write shader surface");
    let output_dir = temp_dir("build_report_shader_cpu_fallback_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"domain_family\":\"shader\""));
    assert!(json.contains("\"runtime_payload_backed_heterogeneous_units\":1"));
    assert!(json.contains("\"runtime_cpu_fallback_units\":1"));
    assert!(json.contains("\"runtime_host_consumable_units\":1"));
    assert!(json.contains("\"runtime_execution_host_fallback_events\":"));
    assert!(!json.contains("\"runtime_execution_host_fallback_events\":0"));
}

#[test]
fn build_report_json_executes_host_yir_kernel_values() {
    let project_root = write_temp_project_fixture(
        "kernel_host_yir_runtime_demo",
        r#"
name = "kernel_host_yir_runtime_demo"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() {
    let input = kernel_tensor(1, 3, "2,4,6");
    let weights = kernel_tensor(3, 2, "1,-2,3,0,2,1");
    let projected = kernel_matmul(input, weights);
    let summary: i64 = kernel_reduce_sum(projected);
    print(summary);
  }
}
"#,
    );
    let output_dir = temp_dir("build_report_kernel_host_yir_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"runtime_host_yir_attempted\":true"));
    assert!(json.contains("\"runtime_host_yir_ok\":true"));
    assert!(json.contains("\"runtime_host_yir_kernel_nodes\":4"));
    assert!(json.contains("\"runtime_host_yir_tensor_values\":3"));
    assert!(json.contains("\"runtime_host_yir_scalar_values\":3"));
    assert!(json.contains("\"runtime_host_yir_kernel_integer_checksum\":73"));
    assert!(json.contains("\"runtime_execution_kernel_host_reference_events\":4"));
}

#[test]
fn build_report_json_exposes_kernel_result_profile_bundle_summary() {
    let project_root =
        checked_in_path("../../examples/projects/domains/kernel_result_profile_demo");
    let output_dir = temp_dir("build_report_kernel_result_profile_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"ready_to_run\":true"));
    assert!(json.contains("\"binary_name\":\"kernel_result_profile_demo\""));
    assert!(json.contains("\"packaging_mode\":\"native-cpu-llvm\""));
    assert!(json.contains("\"domain_units_count\":2"));
    assert!(json.contains("\"heterogeneous_domain_count\":1"));
    assert!(json.contains("\"domain_family\":\"cpu\""));
    assert!(json.contains("\"domain_family\":\"kernel\""));
    assert!(json.contains("\"selected_lowering_target\":\"llvm\""));
    assert!(json.contains("\"backend_family\":\"coreml\""));
    assert!(json.contains("\"selected_lowering_target\":\"coreml.apple-ane\""));
    assert!(json.contains("\"bridge_registry_units\":1"));
    assert!(json.contains("\"host_bridge_plan_units\":1"));
    assert!(json.contains("\"runtime_payload_backed_heterogeneous_units\":1"));
    assert!(json.contains("\"runtime_execution_kernel_host_reference_events\":4"));
    assert!(json.contains("\"runtime_host_yir_attempted\":true"));
    assert!(json.contains("\"runtime_host_yir_ok\":true"));
    assert!(json.contains("\"runtime_host_yir_kernel_nodes\":4"));
    assert!(json.contains("\"runtime_host_yir_kernel_integer_checksum\":18"));
    assert!(json.contains("\"link_plan_final_stage\":\"host-native-link\""));
    assert!(json.contains("\"link_plan_final_driver\":\"clang\""));
    assert!(json.contains("\"link_plan_domain_units\":2"));
}

#[test]
fn run_artifact_json_exposes_real_heterogeneous_runtime_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_profile_demo");
    let output_dir = temp_dir("run_artifact_shader_profile_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    let json = render_run_artifact_json(&output_dir.join("nuis.build.manifest.toml"));

    assert!(json.contains("\"binary_resolved\":true"));
    assert!(json.contains("\"heterogeneous_domain_count\":1"));
    assert!(json.contains("\"bridge_registry_units\":1"));
    assert!(json.contains("\"host_bridge_plan_units\":1"));
    assert!(json.contains("\"domain_payload_blobs_checked\":1"));
    assert!(json.contains("\"link_plan_domain_units\":2"));
    assert!(json.contains("\"domain_family\":\"shader\""));
}

#[test]
fn build_report_json_exposes_bridge_bearing_exchange_summary() {
    let project_root = checked_in_path("../../examples/projects/domains/shader_packet_bridge_demo");
    let output_dir = temp_dir("build_report_shader_packet_bridge_outputs");

    handle_build(
        project_root.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("build passes");
    let run_json = render_run_artifact_json(&output_dir);
    assert!(run_json.contains("\"payload_decoder_manifest_persisted\":true"));
    let json = render_build_report_json(&output_dir);

    assert!(json.contains("\"packaging_mode\":\"window-aot-bundle\""));
    assert!(json.contains("\"build_report_payload_decoder_manifest_available\":true"));
    assert!(json.contains("\"build_report_payload_decoder_manifest_status\":\"ready\""));
    assert!(json.contains("\"build_report_payload_decoder_manifest_record_count\":1"));
    assert!(json.contains("\"build_report_payload_decoder_manifest_invalid_record_count\":0"));
    assert!(json.contains("\"domain_units_count\":3"));
    assert!(json.contains("\"heterogeneous_domain_count\":2"));
    assert!(json.contains("\"domain_family\":\"data\""));
    assert!(json.contains("\"domain_family\":\"shader\""));
    assert!(json.contains("\"bridge_registry_units\":2"));
    assert!(json.contains("\"bridge_registry_entries_checked\":2"));
    assert!(json.contains("\"host_bridge_plan_units\":2"));
    assert!(json.contains("\"host_bridge_plan_entries_checked\":2"));
    assert!(json.contains("\"domain_payload_blobs_checked\":2"));
    assert!(json.contains("\"domain_payload_bridge_plans_checked\":2"));
    assert!(json.contains("\"domain_bridge_stubs_checked\":2"));
    assert!(json.contains("\"link_plan_final_stage\":\"heterogeneous-bundle-pack\""));
    assert!(json.contains("\"link_plan_final_driver\":\"yir-pack-aot\""));
    assert!(json.contains("\"link_plan_domain_units\":3"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_ready_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_registry_dispatch_ready_units\":2"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_units\":1"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_ready_units\":1"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_readiness_ready\":true"));
    assert!(json.contains("\"link_plan_heterogeneous_domain_families\":[\"data\",\"shader\"]"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_families\":["));
    assert!(json.contains("\"link_plan_heterogeneous_target_devices\":["));
    assert!(json.contains("\"link_plan_heterogeneous_domain_first_unready\":null"));
    assert!(json.contains("\"link_plan_heterogeneous_backend_artifact_first_unready\":null"));
    assert!(
        json.contains("\"link_plan_heterogeneous_domain_registry_dispatch_first_blocked\":null")
    );
    assert!(json.contains("\"link_plan_heterogeneous_domain_readiness\":[{"));
    assert!(json.contains("\"backend_artifact_candidate\":false"));
    assert!(json.contains("\"backend_artifact_candidate\":true"));
    assert!(json.contains("\"backend_artifact_ready\":true"));
    assert!(json.contains("\"backend_artifact_missing_signals\":[]"));
    assert!(json.contains("\"backend_artifact_key\":\"shader:metal:apple-silicon-gpu\""));
    assert!(json.contains("\"registry_dispatch_readiness_status\":\"ready\""));
    assert!(json.contains("\"registry_dispatch_readiness_ready\":true"));
    assert!(json.contains("\"registry_dispatch_missing_signals\":[]"));
    assert!(json.contains("\"registry_dispatch_bridge_materialized\":true"));
    assert!(json.contains("\"registry_execution_readiness_materialized\":true"));
}
