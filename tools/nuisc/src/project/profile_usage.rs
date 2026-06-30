use nuis_semantics::model::{NirModule, NirStmt};

#[path = "profile_usage_exprs.rs"]
mod profile_usage_exprs;
#[path = "profile_usage_walk.rs"]
mod profile_usage_walk;

use profile_usage_exprs::{
    expr_uses_cpu_extern_call, expr_uses_data_profile_handle_table,
    expr_uses_data_profile_send_downlink, expr_uses_data_profile_send_uplink,
    expr_uses_network_profile_bind_core, expr_uses_network_profile_endpoint_kind,
    expr_uses_network_profile_slot, expr_uses_shader_binding_profile_contract,
    expr_uses_shader_profile_color_seed, expr_uses_shader_profile_draw_instanced,
    expr_uses_shader_profile_packet, expr_uses_shader_profile_radius_seed,
    expr_uses_shader_profile_render, expr_uses_shader_profile_speed_seed,
};
pub(super) use profile_usage_walk::{expr_walk_any, stmt_uses_expr_predicate};

pub(crate) fn nir_uses_shader_profile_render(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_render(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_draw_instanced(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_draw_instanced(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_packet(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_packet(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_color_seed(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_color_seed(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_speed_seed(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_speed_seed(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_radius_seed(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_radius_seed(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_binding_profile_contract(
    module: &NirModule,
    profile_contract: &str,
) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_binding_profile_contract(stmt, profile_contract))
    })
}

pub(crate) fn nir_uses_data_profile_handle_table(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_data_profile_handle_table(stmt, unit))
    })
}

pub(crate) fn nir_uses_data_profile_send_uplink(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_data_profile_send_uplink(stmt, unit))
    })
}

pub(crate) fn nir_uses_data_profile_send_downlink(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_data_profile_send_downlink(stmt, unit))
    })
}

pub(crate) fn nir_uses_network_profile_bind_core(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_network_profile_bind_core(stmt, unit))
    })
}

pub(crate) fn nir_uses_network_profile_endpoint_kind(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_network_profile_endpoint_kind(stmt, unit))
    })
}

pub(crate) fn nir_uses_network_profile_slot(module: &NirModule, unit: &str, slot: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_network_profile_slot(stmt, unit, slot))
    })
}

pub(crate) fn nir_uses_cpu_extern_call(module: &NirModule, callee: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_cpu_extern_call(stmt, callee))
    })
}

fn stmt_uses_shader_profile_render(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| expr_uses_shader_profile_render(value, unit))
}

fn stmt_uses_shader_profile_draw_instanced(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_draw_instanced(value, unit)
    })
}

fn stmt_uses_shader_profile_packet(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| expr_uses_shader_profile_packet(value, unit))
}

fn stmt_uses_shader_profile_color_seed(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_color_seed(value, unit)
    })
}

fn stmt_uses_shader_profile_speed_seed(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_speed_seed(value, unit)
    })
}

fn stmt_uses_shader_profile_radius_seed(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_radius_seed(value, unit)
    })
}

fn stmt_uses_shader_binding_profile_contract(stmt: &NirStmt, profile_contract: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_binding_profile_contract(value, profile_contract)
    })
}

fn stmt_uses_data_profile_handle_table(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_data_profile_handle_table(value, unit)
    })
}

fn stmt_uses_data_profile_send_uplink(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_data_profile_send_uplink(value, unit)
    })
}

fn stmt_uses_data_profile_send_downlink(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_data_profile_send_downlink(value, unit)
    })
}

fn stmt_uses_network_profile_bind_core(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_network_profile_bind_core(value, unit)
    })
}

fn stmt_uses_network_profile_endpoint_kind(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_network_profile_endpoint_kind(value, unit)
    })
}

fn stmt_uses_network_profile_slot(stmt: &NirStmt, unit: &str, slot: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_network_profile_slot(value, unit, slot)
    })
}

fn stmt_uses_cpu_extern_call(stmt: &NirStmt, callee: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| expr_uses_cpu_extern_call(value, callee))
}
