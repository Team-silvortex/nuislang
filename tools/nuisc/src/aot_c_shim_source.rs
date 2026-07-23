use nuis_semantics::model::AstModule;

use crate::aot_c_shim_buffer_runtime::append_c_shim_buffer_runtime;
use crate::aot_c_shim_env_io_runtime::append_c_shim_env_io_runtime;
use crate::aot_c_shim_file_runtime::append_c_shim_file_runtime;
use crate::aot_c_shim_fs_runtime::append_c_shim_fs_runtime;
use crate::aot_c_shim_header_runtime::append_c_shim_header_runtime;
use crate::aot_c_shim_helpers::{
    ast_hetero_lifecycle_surface_slots, ast_uses_hetero_lifecycle_surface,
    ast_uses_network_lifecycle_surface, ast_uses_provider_worker_surface, collect_exported_entries,
    collect_host_ffi_symbols, render_exported_entry_wrapper, render_host_ffi_stub,
    render_lifecycle_export_wrappers,
};
use crate::aot_c_shim_http_runtime::append_c_shim_http_runtime;
use crate::aot_c_shim_network_owned_runtime::append_c_shim_network_owned_runtime;
use crate::aot_c_shim_network_probe_runtime::append_c_shim_network_probe_runtime;
use crate::aot_c_shim_network_runtime::append_c_shim_network_runtime;
use crate::aot_c_shim_owned_blob_runtime::append_c_shim_owned_blob_runtime;
use crate::aot_c_shim_path_runtime::append_c_shim_path_runtime;
use crate::aot_c_shim_process_runtime::append_c_shim_process_runtime;
use crate::aot_c_shim_provider_worker_runtime::append_c_shim_provider_worker_runtime;
use crate::aot_c_shim_runtime::{
    append_c_shim_lifecycle_runtime, append_c_shim_main, append_c_shim_prelude,
};
use crate::aot_c_shim_serialization_runtime::append_c_shim_serialization_runtime;
use crate::aot_c_shim_text_runtime::append_c_shim_text_runtime;
use crate::aot_c_shim_time_debug_runtime::append_c_shim_time_debug_runtime;

pub(crate) fn render_c_shim_source(ast: &AstModule) -> String {
    let mut out = String::with_capacity(64 * 1024);
    let network_lifecycle_enabled = if ast_uses_network_lifecycle_surface(ast) {
        "1"
    } else {
        "0"
    };
    let hetero_lifecycle_enabled = if ast_uses_hetero_lifecycle_surface(ast) {
        "1"
    } else {
        "0"
    };
    let hetero_lifecycle_surface_slots = ast_hetero_lifecycle_surface_slots(ast);
    append_c_shim_prelude(
        &mut out,
        network_lifecycle_enabled,
        hetero_lifecycle_enabled,
        hetero_lifecycle_surface_slots,
    );
    append_c_shim_lifecycle_runtime(&mut out);
    append_c_shim_text_runtime(&mut out);
    append_c_shim_owned_blob_runtime(&mut out);
    append_c_shim_serialization_runtime(&mut out);
    append_c_shim_header_runtime(&mut out);
    append_c_shim_http_runtime(&mut out);
    append_c_shim_buffer_runtime(&mut out);
    append_c_shim_file_runtime(&mut out);
    append_c_shim_env_io_runtime(&mut out);
    append_c_shim_path_runtime(&mut out);
    append_c_shim_fs_runtime(&mut out);
    append_c_shim_process_runtime(&mut out);
    if ast_uses_provider_worker_surface(ast) {
        append_c_shim_provider_worker_runtime(&mut out);
    }
    append_c_shim_network_runtime(&mut out);
    append_c_shim_network_probe_runtime(&mut out);
    append_c_shim_network_owned_runtime(&mut out);
    append_c_shim_time_debug_runtime(&mut out);
    for (symbol, function) in collect_host_ffi_symbols(ast) {
        out.push('\n');
        out.push_str(&render_host_ffi_stub(&symbol, function));
    }
    for entry in collect_exported_entries(ast) {
        out.push('\n');
        out.push_str(&render_exported_entry_wrapper(&entry));
    }
    out.push('\n');
    out.push_str(&render_lifecycle_export_wrappers());
    append_c_shim_main(&mut out);
    out
}
