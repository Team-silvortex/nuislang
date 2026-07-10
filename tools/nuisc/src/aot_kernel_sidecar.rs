use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_profile::{
    derived_lowering_profile_for_unit, kernel_registered_feature_surfaces_for_profile,
    kernel_registered_lane_groups_for_profile, kernel_supported_dispatch_kinds_for_profile,
    render_schedule_contract_fields, render_target_specific_backend_fields,
};
use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn render_domain_build_unit_kernel_ir_sidecar(
    unit: &BuildManifestDomainBuildUnit,
) -> String {
    let profile = derived_lowering_profile_for_unit(unit);
    let dispatch_kinds = kernel_supported_dispatch_kinds_for_profile(unit, &profile).unwrap_or(&[]);
    let mut out = String::new();
    out.push_str("schema = \"nuis-kernel-ir-sidecar-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "target_device = \"{}\"\n",
        escape_toml_string(unit.target_device.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "ir_format = \"{}\"\n",
        escape_toml_string(unit.ir_format.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "dispatch_abi = \"{}\"\n",
        escape_toml_string(unit.dispatch_abi.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "backend_priority = {}\n",
        unit.backend_priority.unwrap_or(usize::MAX)
    ));
    out.push_str(&format!(
        "verification = \"{}\"\n",
        escape_toml_string(unit.verification.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    if !dispatch_kinds.is_empty() {
        out.push_str(&format!(
            "supported_dispatch_kinds = {}\n",
            render_string_array(
                &dispatch_kinds
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(feature_surfaces) = kernel_registered_feature_surfaces_for_profile(unit, &profile) {
        out.push_str(&format!(
            "registered_feature_surfaces = {}\n",
            render_string_array(
                &feature_surfaces
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(lane_groups) = kernel_registered_lane_groups_for_profile(unit, &profile) {
        out.push_str(&format!(
            "registered_lane_groups = {}\n",
            render_string_array(
                &lane_groups
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    out.push_str(&render_target_specific_backend_fields(unit, &profile));
    out.push_str(&render_schedule_contract_fields(&profile));
    out.push_str("[lowering_capabilities]\n");
    out.push_str("binary_role = \"linker-input-sidecar\"\n");
    out.push_str("capability_owner = \"kernel-nustar\"\n");
    match profile.profile_key {
        "coreml.apple-ane" => {
            out.push_str("frontend_ir = \"nuis-yir.kernel\"\n");
            out.push_str("native_ir = \"coreml-program\"\n");
            out.push_str("tensor_lowering = \"ranked-tensor-graph\"\n");
            out.push_str("dispatch_lowering = \"ane-graph-submit\"\n");
            out.push_str("memory_lowering = \"tensor-argument-table\"\n");
            out.push_str("result_lowering = \"managed-result-buffer\"\n");
            out.push_str(
                "validation_contracts = [\"glm.tensor-lifetime\", \"time.graph-completion\", \"kernel.shape-contract\"]\n",
            );
        }
        "vulkan.discrete-or-integrated-gpu" => {
            out.push_str("frontend_ir = \"nuis-yir.kernel\"\n");
            out.push_str("native_ir = \"spirv1.6\"\n");
            out.push_str("tensor_lowering = \"storage-buffer-tensor-view\"\n");
            out.push_str("dispatch_lowering = \"compute-grid-or-indirect\"\n");
            out.push_str("memory_lowering = \"descriptor-set-storage-buffer\"\n");
            out.push_str("result_lowering = \"storage-buffer-result\"\n");
            out.push_str(
                "validation_contracts = [\"glm.buffer-lifetime\", \"time.compute-fence\", \"spirv.compute-layout\"]\n",
            );
        }
        "cpu-fallback.cpu-host" => {
            out.push_str("frontend_ir = \"nuis-yir.kernel\"\n");
            out.push_str("native_ir = \"host-simd\"\n");
            out.push_str("tensor_lowering = \"slice-backed-tensor-view\"\n");
            out.push_str("dispatch_lowering = \"threadpool-range-or-tile\"\n");
            out.push_str("memory_lowering = \"host-buffer-slices\"\n");
            out.push_str("result_lowering = \"owned-output-slice\"\n");
            out.push_str(
                "validation_contracts = [\"glm.slice-lifetime\", \"time.threadpool-join\", \"host.slice-bounds\"]\n",
            );
        }
        _ => {
            out.push_str("frontend_ir = \"nuis-yir.kernel\"\n");
            out.push_str("native_ir = \"unknown\"\n");
            out.push_str("tensor_lowering = \"unimplemented\"\n");
            out.push_str("dispatch_lowering = \"unimplemented\"\n");
            out.push_str("memory_lowering = \"unimplemented\"\n");
            out.push_str("result_lowering = \"unimplemented\"\n");
            out.push_str("validation_contracts = [\"glm.buffer-lifetime\"]\n");
        }
    }
    out.push_str("[dispatch_shapes]\n");
    match profile.profile_key {
        "coreml.apple-ane" => {
            out.push_str("primary = \"graph\"\n");
            out.push_str("fallback = \"batch\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"tensor.input0, tensor.output0\"\n");
            out.push_str("argument_model = \"tensor-argument-table\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("graph = \"infer_main\"\n");
            out.push_str("batch = \"infer_batch\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("graph_body = \"program infer_main(tensor<1x4xf32> input) -> tensor<1x4xf32> { return input; }\"\n");
            out.push_str(
                "batch_body = \"batch infer_batch(count: i32) { /* coreml batch stub */ }\"\n",
            );
        }
        "vulkan.discrete-or-integrated-gpu" => {
            out.push_str("primary = \"grid\"\n");
            out.push_str("fallback = \"indirect\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"set0.buffer0, set0.buffer1\"\n");
            out.push_str("argument_model = \"descriptor-set-layout\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("grid = \"main\"\n");
            out.push_str("indirect = \"main_indirect\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("grid_body = \"OpEntryPoint GLCompute %main \\\"main\\\"\"\n");
            out.push_str(
                "indirect_body = \"OpEntryPoint GLCompute %main_indirect \\\"main_indirect\\\"\"\n",
            );
        }
        "cpu-fallback.cpu-host" => {
            out.push_str("primary = \"range\"\n");
            out.push_str("fallback = \"tile\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"slice.input, slice.output\"\n");
            out.push_str("argument_model = \"host-buffer-slices\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("range = \"run_range\"\n");
            out.push_str("tile = \"run_tile\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("range_body = \"fn run_range(start: u32, end: u32) { }\"\n");
            out.push_str("tile_body = \"fn run_tile(tile: u32) { }\"\n");
        }
        _ => {
            out.push_str("primary = \"graph\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("graph = \"unimplemented\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("graph_body = \"unimplemented\"\n");
        }
    }
    out
}
