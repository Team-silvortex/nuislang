use super::*;
use nuis_semantics::model::{NirExpr, NirStmt};
use std::{collections::BTreeMap, fs, path::PathBuf};
use yir_core::EdgeKind;

#[path = "shader_nova_contracts/fixtures.rs"]
mod fixtures;
use fixtures::*;

#[path = "shader_nova_contracts/abi.rs"]
mod abi;
#[path = "shader_nova_contracts/async_lowering.rs"]
mod async_lowering;
#[path = "shader_nova_contracts/data_links.rs"]
mod data_links;
#[path = "shader_nova_contracts/packet_contracts.rs"]
mod packet_contracts;
#[path = "shader_nova_contracts/rendering.rs"]
mod rendering;
#[path = "shader_nova_contracts/reverse_links.rs"]
mod reverse_links;

// Preserve the existing fully qualified libtest names; suites own assertions and fixtures.

#[test]
fn validates_shader_packet_contract_from_cpu_usage() {
    packet_contracts::validates_shader_packet_contract_from_cpu_usage();
}

#[test]
fn validates_nova_panel_contract_from_struct_literal_usage() {
    packet_contracts::validates_nova_panel_contract_from_struct_literal_usage();
}

#[test]
fn compiles_real_shader_async_policy_project() {
    async_lowering::compiles_real_shader_async_policy_project();
}

#[test]
fn lowers_real_shader_async_policy_project_with_expected_async_summary_shape() {
    async_lowering::lowers_real_shader_async_policy_project_with_expected_async_summary_shape();
}

#[test]
fn lowers_real_shader_async_windowed_batch_project_with_expected_batch_summary_shape() {
    async_lowering::lowers_real_shader_async_windowed_batch_project_with_expected_batch_summary_shape();
}

#[test]
fn lowers_real_shader_async_fallback_project_with_expected_fallback_shape() {
    async_lowering::lowers_real_shader_async_fallback_project_with_expected_fallback_shape();
}

#[test]
fn lowers_real_shader_result_project_with_expected_result_shape() {
    rendering::lowers_real_shader_result_project_with_expected_result_shape();
}

#[test]
fn lowers_real_shader_draw_render_project_with_expected_dual_result_shape() {
    rendering::lowers_real_shader_draw_render_project_with_expected_dual_result_shape();
}

#[test]
fn lowers_real_pixelmagic_packet_bridge_project_with_expected_bridge_shape() {
    rendering::lowers_real_pixelmagic_packet_bridge_project_with_expected_bridge_shape();
}

#[test]
fn lowers_real_pixelmagic_render_project_with_expected_render_shape() {
    rendering::lowers_real_pixelmagic_render_project_with_expected_render_shape();
}

#[test]
fn lowers_real_pixelmagic_texture_resource_project_with_expected_resource_shape() {
    rendering::lowers_real_pixelmagic_texture_resource_project_with_expected_resource_shape();
}

#[test]
fn lowers_real_pixelmagic_pipeline_project_with_expected_pipeline_shape() {
    rendering::lowers_real_pixelmagic_pipeline_project_with_expected_pipeline_shape();
}

#[test]
fn lowers_real_shader_async_schedule_project_with_expected_schedule_shape() {
    async_lowering::lowers_real_shader_async_schedule_project_with_expected_schedule_shape();
}

#[test]
fn lowers_real_shader_async_fanin_project_with_expected_fanin_shape() {
    async_lowering::lowers_real_shader_async_fanin_project_with_expected_fanin_shape();
}

#[test]
fn lowers_real_shader_packet_profile_project_with_expected_packet_shape() {
    packet_contracts::lowers_real_shader_packet_profile_project_with_expected_packet_shape();
}

#[test]
fn lowers_real_shader_packet_bridge_project_with_expected_bridge_shape() {
    packet_contracts::lowers_real_shader_packet_bridge_project_with_expected_bridge_shape();
}

#[test]
fn rejects_shader_packet_field_count_mismatch() {
    packet_contracts::rejects_shader_packet_field_count_mismatch();
}

#[test]
fn nova_panel_packet_requires_nova_support_surface() {
    packet_contracts::nova_panel_packet_requires_nova_support_surface();
}

#[test]
fn project_link_accepts_draw_only_shader_bridge_at_nir_level() {
    data_links::project_link_accepts_draw_only_shader_bridge_at_nir_level();
}

#[test]
fn infers_project_route_payload_type_from_entry_with_shared_cpu_helper() {
    data_links::infers_project_route_payload_type_from_entry_with_shared_cpu_helper();
}

#[test]
fn validates_project_links_against_nir_with_shared_cpu_helper_indirection() {
    data_links::validates_project_links_against_nir_with_shared_cpu_helper_indirection();
}

#[test]
fn materializes_shader_and_data_type_contract_nodes_into_yir() {
    data_links::materializes_shader_and_data_type_contract_nodes_into_yir();
}

#[test]
fn validates_reverse_shader_project_links_against_nir_via_data_bridge() {
    reverse_links::validates_reverse_shader_project_links_against_nir_via_data_bridge();
}

#[test]
fn validates_reverse_shader_project_links_against_yir_via_data_bridge() {
    reverse_links::validates_reverse_shader_project_links_against_yir_via_data_bridge();
}

#[test]
fn compiles_reverse_shader_project_via_data_bridge() {
    reverse_links::compiles_reverse_shader_project_via_data_bridge();
}

#[test]
fn rejects_reverse_shader_project_via_data_bridge_when_shader_to_data_xfer_is_missing() {
    reverse_links::rejects_reverse_shader_project_via_data_bridge_when_shader_to_data_xfer_is_missing();
}

#[test]
fn rejects_reverse_shader_project_links_against_yir_when_shader_to_data_xfer_is_missing() {
    reverse_links::rejects_reverse_shader_project_links_against_yir_when_shader_to_data_xfer_is_missing();
}

#[test]
fn project_support_modules_accept_darwin_x86_64_shader_and_data_abis() {
    abi::project_support_modules_accept_darwin_x86_64_shader_and_data_abis();
}
