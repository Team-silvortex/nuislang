use std::collections::BTreeMap;

use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef};

pub(crate) struct ExportedEntry {
    pub(crate) function_name: String,
    pub(crate) symbol: String,
    pub(crate) param_count: usize,
}

pub(crate) fn ast_uses_network_lifecycle_surface(ast: &AstModule) -> bool {
    ast.domain == "network"
        || ast
            .externs
            .iter()
            .any(|function| function.name.starts_with("host_network_"))
}

pub(crate) fn ast_uses_hetero_lifecycle_surface(ast: &AstModule) -> bool {
    ast.domain == "shader"
        || ast.domain == "kernel"
        || ast.externs.iter().any(|function| {
            function.name.starts_with("host_shader_") || function.name.starts_with("host_kernel_")
        })
}

pub(crate) fn ast_uses_provider_worker_surface(ast: &AstModule) -> bool {
    ast.externs
        .iter()
        .any(|function| function.name.starts_with("host_provider_worker_"))
}

pub(crate) fn ast_hetero_lifecycle_surface_slots(ast: &AstModule) -> usize {
    let mut slots = 0usize;
    if ast.domain == "shader" || ast.domain == "kernel" {
        slots += 1;
    }
    slots
        + ast
            .externs
            .iter()
            .filter(|function| {
                function.name.starts_with("host_shader_")
                    || function.name.starts_with("host_kernel_")
            })
            .count()
}

pub(crate) fn collect_exported_entries(ast: &AstModule) -> Vec<ExportedEntry> {
    ast.functions
        .iter()
        .filter_map(|function| {
            function
                .attributes
                .iter()
                .find(|attribute| attribute.name == "export")
                .and_then(|attribute| attribute.args.first())
                .and_then(|arg| match &arg.value {
                    nuis_semantics::model::AstAttributeValue::String(value) => {
                        Some(ExportedEntry {
                            function_name: function.name.clone(),
                            symbol: value.clone(),
                            param_count: function.params.len(),
                        })
                    }
                    _ => None,
                })
        })
        .collect()
}

pub(crate) fn render_exported_entry_wrapper(entry: &ExportedEntry) -> String {
    if entry.function_name == "main" {
        return format!(
            "int64_t {}(void) {{\n    return nuis_yir_entry();\n}}\n",
            entry.symbol
        );
    }
    let declaration_params = (0..entry.param_count)
        .map(|index| format!("int64_t arg{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let call_params = (0..entry.param_count)
        .map(|index| format!("arg{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "extern int64_t nuis_fn_{}({declaration_params});\nint64_t {}({declaration_params}) {{\n    return nuis_fn_{}({call_params});\n}}\n",
        entry.function_name, entry.symbol, entry.function_name
    )
}

pub(crate) fn render_lifecycle_export_wrappers() -> String {
    r#"int64_t nuis_lifecycle_bootstrap_export_v1(void) {
    return nuis_lifecycle_bootstrap_entry_v1();
}

int64_t nuis_lifecycle_tick_export_v1(void) {
    return nuis_lifecycle_tick_once_v1();
}

int64_t nuis_lifecycle_shutdown_export_v1(int64_t status) {
    return nuis_lifecycle_shutdown_v1(status);
}

int64_t nuis_lifecycle_yalivia_rpc_export_v1(void) {
    return nuis_lifecycle_yalivia_rpc_hook_v1();
}

int64_t nuis_lifecycle_network_bridge_progress_export_v1(void) {
    return nuis_lifecycle_state.network_bridge_progress_count;
}

int64_t nuis_lifecycle_hetero_submission_progress_export_v1(void) {
    return nuis_lifecycle_state.hetero_submission_progress_count;
}
"#
    .to_owned()
}

pub(crate) fn collect_host_ffi_symbols(ast: &AstModule) -> BTreeMap<String, AstExternFunction> {
    let mut out = BTreeMap::new();
    out.insert(
        "host_text_handle".to_owned(),
        AstExternFunction {
            visibility: nuis_semantics::model::AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: "host_text_handle".to_owned(),
            params: vec![nuis_semantics::model::AstParam {
                name: "text".to_owned(),
                ty: AstTypeRef {
                    name: "String".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: false,
                },
            }],
            return_type: AstTypeRef {
                name: "i64".to_owned(),
                generic_args: vec![],
                is_optional: false,
                is_ref: false,
            },
            host_symbol: None,
        },
    );
    for function in &ast.externs {
        if function.name.starts_with("host_") {
            out.insert(function.name.clone(), function.clone());
        }
    }
    for interface in &ast.extern_interfaces {
        for method in &interface.methods {
            out.insert(
                format!("{}__{}", interface.name, method.name),
                method.clone(),
            );
        }
    }
    out
}

pub(crate) fn render_host_ffi_stub(symbol: &str, function: AstExternFunction) -> String {
    let mut signature = String::new();
    if function.params.is_empty() {
        signature.push_str("void");
    } else {
        let mut first = true;
        for param in &function.params {
            if !first {
                signature.push_str(", ");
            }
            first = false;
            signature.push_str(&format!(
                "{} {}",
                c_type_for_ast_type(&param.ty),
                param.name
            ));
        }
    }
    let body = render_registered_host_ffi_body(symbol, &function)
        .unwrap_or_else(|| render_generic_host_ffi_body(&function));
    format!(
        "{} {}({}) {{\n{}\n}}\n",
        c_type_for_ast_type(&function.return_type),
        symbol,
        signature,
        body
    )
}

pub(crate) fn arg_name(index: usize, function: &AstExternFunction) -> String {
    function
        .params
        .get(index)
        .map(|param| param.name.clone())
        .unwrap_or_else(|| "0".to_owned())
}

pub(crate) fn render_generic_host_ffi_body(function: &AstExternFunction) -> String {
    if function.params.is_empty() {
        return "    return 0;".to_owned();
    }
    if function.params.len() == 1 {
        return format!("    return {};", function.params[0].name);
    }
    let mut expr = String::new();
    for (idx, param) in function.params.iter().enumerate() {
        if idx > 0 {
            expr.push_str(" + ");
        }
        expr.push_str(&param.name);
    }
    format!("    return {};", expr)
}

pub(crate) fn render_registered_host_ffi_body(
    symbol: &str,
    function: &AstExternFunction,
) -> Option<String> {
    for (suffix, target, arity) in [
        ("color_bias", "host_color_bias", 1usize),
        ("speed_curve", "host_speed_curve", 1),
        ("radius_curve", "host_radius_curve", 1),
        ("mix_tick", "host_mix_tick", 2),
    ] {
        if symbol.ends_with(suffix) {
            return Some(render_host_call(target, function, arity));
        }
    }

    let (target, arity) = match symbol {
        "host_argv_count" => ("nuis_host_argv_count", 0),
        "host_argv_at" => ("nuis_host_argv_at", 1),
        "host_env_has" => ("nuis_host_env_has", 1),
        "host_env_get" => ("nuis_host_env_get", 1),
        "host_text_handle" => ("nuis_host_text_handle", 1),
        "host_text_len" => ("nuis_host_text_len_value", 1),
        "host_text_line_count" => ("nuis_host_text_line_count", 1),
        "host_text_word_count" => ("nuis_host_text_word_count", 1),
        "host_text_concat" => ("nuis_host_text_concat", 2),
        "host_serialize_text_into" => ("nuis_host_serialize_text_into", 3),
        "host_serialize_i64_into" => ("nuis_host_serialize_i64_into", 3),
        "host_serialize_bool_into" => ("nuis_host_serialize_bool_into", 3),
        "host_serialize_byte_into" => ("nuis_host_serialize_byte_into", 3),
        "host_deserialize_i64_from" => ("nuis_host_deserialize_i64_from", 3),
        "host_deserialize_bool_from" => ("nuis_host_deserialize_bool_from", 3),
        "host_deserialize_byte_from" => ("nuis_host_deserialize_byte_from", 2),
        "host_deserialize_text_from" => ("nuis_host_deserialize_text_from", 3),
        "host_parse_header_line" => ("nuis_host_parse_header_line", 4),
        "host_find_header_value" => ("nuis_host_find_header_value", 4),
        "host_find_status_line_reason" => ("nuis_host_find_status_line_reason", 3),
        "host_parse_http_response_summary" => ("nuis_host_parse_http_response_summary", 3),
        "host_parse_http_request_summary" => ("nuis_host_parse_http_request_summary", 3),
        "host_parse_http_roundtrip_summary" => ("nuis_host_parse_http_roundtrip_summary", 6),
        "host_deserialize_text_equals" => ("nuis_host_deserialize_text_equals", 4),
        "host_deserialize_text_starts_with" => ("nuis_host_deserialize_text_starts_with", 4),
        "host_deserialize_text_contains" => ("nuis_host_deserialize_text_contains", 4),
        "host_deserialize_text_ends_with" => ("nuis_host_deserialize_text_ends_with", 4),
        "host_buffer_find_byte" => ("nuis_host_buffer_find_byte", 4),
        "host_fill_bytes" => ("nuis_host_fill_bytes", 4),
        "host_copy_bytes" => ("nuis_host_copy_bytes", 6),
        "host_compare_bytes" => ("nuis_host_compare_bytes", 6),
        "host_buffer_find_text" => ("nuis_host_buffer_find_text", 4),
        "host_buffer_find_line_end" => ("nuis_host_buffer_find_line_end", 3),
        "host_buffer_trim_line_end" => ("nuis_host_buffer_trim_line_end", 3),
        "host_file_open" => ("nuis_host_file_open", 2),
        "host_file_read" => ("nuis_host_file_read", 3),
        "host_file_write" => ("nuis_host_file_write", 2),
        "host_file_close" => ("nuis_host_file_close", 1),
        "host_network_connect_probe" => ("nuis_host_network_connect_probe", 3),
        "host_network_open_tcp_stream" => ("nuis_host_network_open_tcp_stream", 2),
        "host_network_open_tcp_listener" => ("nuis_host_network_open_tcp_listener", 3),
        "host_network_open_udp_datagram" => ("nuis_host_network_open_udp_datagram", 2),
        "host_network_bind_udp_datagram" => ("nuis_host_network_bind_udp_datagram", 3),
        "host_network_accept_owned" => ("nuis_host_network_accept_owned", 3),
        "host_network_close_owned" => ("nuis_host_network_close_owned", 1),
        "host_network_send_owned" => ("nuis_host_network_send_owned", 3),
        "host_network_recv_owned" => ("nuis_host_network_recv_owned", 3),
        "host_network_recv_http_status_owned" => ("nuis_host_network_recv_http_status_owned", 3),
        "host_network_accept_probe" => ("nuis_host_network_accept_probe", 3),
        "host_network_close" => ("nuis_host_network_close", 1),
        "host_network_send_probe" => ("nuis_host_network_send_probe", 3),
        "host_network_recv_probe" => ("nuis_host_network_recv_probe", 3),
        "host_dir_open" => ("nuis_host_dir_open", 1),
        "host_dir_entry_count" => ("nuis_host_dir_entry_count", 1),
        "host_dir_close" => ("nuis_host_dir_close", 1),
        "host_dir_create" => ("nuis_host_dir_create", 1),
        "host_dir_remove" => ("nuis_host_dir_remove", 1),
        "host_stdin_read" => ("nuis_host_stdin_read", 2),
        "host_stdout_write" => ("nuis_host_stdout_write", 1),
        "host_stderr_write" => ("nuis_host_stderr_write", 1),
        "host_stdout_flush" => ("nuis_host_stdout_flush", 0),
        "host_stderr_flush" => ("nuis_host_stderr_flush", 0),
        "host_tty_isatty" => ("nuis_host_tty_isatty", 1),
        "host_tty_width" => ("nuis_host_tty_width", 1),
        "host_tty_height" => ("nuis_host_tty_height", 1),
        "host_cwd_handle" => ("nuis_host_cwd_handle", 0),
        "host_cwd_len" => ("nuis_host_cwd_len_value", 0),
        "host_temp_dir_handle" => ("nuis_host_temp_dir_handle", 0),
        "host_temp_path_len" => ("nuis_host_temp_path_len", 0),
        "host_temp_file_handle" => ("nuis_host_temp_file_handle", 1),
        "host_chdir" => ("nuis_host_chdir_value", 1),
        "host_path_join_len" => ("nuis_host_path_join_len", 2),
        "host_path_is_absolute" => ("nuis_host_path_is_absolute", 1),
        "host_path_is_empty" => ("nuis_host_path_is_empty", 1),
        "host_path_is_dot" => ("nuis_host_path_is_dot", 1),
        "host_path_is_dotdot" => ("nuis_host_path_is_dotdot", 1),
        "host_path_is_relative" => ("nuis_host_path_is_relative", 1),
        "host_path_is_root" => ("nuis_host_path_is_root", 1),
        "host_path_basename" => ("nuis_host_path_basename", 1),
        "host_path_filename" => ("nuis_host_path_filename", 1),
        "host_path_basename_matches" => ("nuis_host_path_basename_matches", 2),
        "host_path_filename_matches" => ("nuis_host_path_filename_matches", 2),
        "host_path_parent_matches" => ("nuis_host_path_parent_matches", 2),
        "host_path_stem_matches" => ("nuis_host_path_stem_matches", 2),
        "host_path_parent" => ("nuis_host_path_parent", 1),
        "host_path_has_parent" => ("nuis_host_path_has_parent", 1),
        "host_path_is_basename_only" => ("nuis_host_path_is_basename_only", 1),
        "host_path_depth" => ("nuis_host_path_depth", 1),
        "host_path_stem" => ("nuis_host_path_stem", 1),
        "host_path_extension" => ("nuis_host_path_extension", 1),
        "host_path_has_extension" => ("nuis_host_path_has_extension", 1),
        "host_path_matches_extension" => ("nuis_host_path_matches_extension", 2),
        "host_path_extension_is" => ("nuis_host_path_extension_is", 2),
        "host_path_starts_with_dot" => ("nuis_host_path_starts_with_dot", 1),
        "host_path_ends_with_slash" => ("nuis_host_path_ends_with_slash", 1),
        "host_path_is_hidden" => ("nuis_host_path_is_hidden", 1),
        "host_path_rename" => ("nuis_host_path_rename", 2),
        "host_path_copy" => ("nuis_host_path_copy", 2),
        "host_path_remove" => ("nuis_host_path_remove", 1),
        "host_fs_exists" => ("nuis_host_fs_exists", 1),
        "host_fs_kind" => ("nuis_host_fs_kind", 1),
        "host_fs_size" => ("nuis_host_fs_size", 1),
        "host_stat_mode" => ("nuis_host_stat_mode", 1),
        "host_stat_mtime_ns" => ("nuis_host_stat_mtime_ns", 1),
        "host_stat_ctime_ns" => ("nuis_host_stat_ctime_ns", 1),
        "host_process_id" => ("nuis_host_process_id", 0),
        "host_process_status" => ("nuis_host_process_status", 0),
        "host_process_exit_code" => ("nuis_host_process_exit_code", 1),
        "host_provider_worker_open" => ("nuis_host_provider_worker_open", 2),
        "host_provider_worker_receive" => ("nuis_host_provider_worker_receive", 0),
        "host_provider_worker_request" => ("nuis_host_provider_worker_request", 0),
        "host_provider_worker_descriptor_table" => {
            ("nuis_host_provider_worker_descriptor_table", 0)
        }
        "host_provider_worker_descriptor_count" => {
            ("nuis_host_provider_worker_descriptor_count", 0)
        }
        "host_provider_worker_provider_key" => ("nuis_host_provider_worker_provider_key", 0),
        "host_provider_worker_capability_hash" => ("nuis_host_provider_worker_capability_hash", 0),
        "host_provider_worker_is_close" => ("nuis_host_provider_worker_is_close", 0),
        "host_provider_worker_launch_provider_key" => {
            ("nuis_host_provider_worker_launch_provider_key", 0)
        }
        "host_provider_worker_launch_capability_hash" => {
            ("nuis_host_provider_worker_launch_capability_hash", 0)
        }
        "host_provider_worker_reply" => ("nuis_host_provider_worker_reply", 1),
        "host_provider_worker_close" => ("nuis_host_provider_worker_close", 0),
        "host_command_spawn" => ("nuis_host_command_spawn", 2),
        "host_command_spawn_in" => ("nuis_host_command_spawn_in", 4),
        "host_command_status" => ("nuis_host_command_status", 1),
        "host_command_wait" => ("nuis_host_command_wait", 1),
        "host_command_wait_exit" => ("nuis_host_command_wait_exit", 1),
        "host_subprocess_spawn" => ("nuis_host_subprocess_spawn", 3),
        "host_subprocess_spawn_in" => ("nuis_host_subprocess_spawn_in", 5),
        "host_subprocess_signal" => ("nuis_host_subprocess_signal", 2),
        "host_subprocess_join" => ("nuis_host_subprocess_join", 1),
        "host_subprocess_join_exit" => ("nuis_host_subprocess_join_exit", 1),
        "host_wall_time_ns" => ("nuis_host_wall_time_ns", 0),
        "host_monotonic_time_ns" => ("nuis_host_monotonic_time_ns", 0),
        "host_sleep_ns" => ("nuis_host_sleep_ns", 1),
        _ => return None,
    };
    Some(render_host_call(target, function, arity))
}

fn render_host_call(target: &str, function: &AstExternFunction, arity: usize) -> String {
    let args = (0..arity)
        .map(|index| arg_name(index, function))
        .collect::<Vec<_>>();
    if args.is_empty() {
        format!("    return {target}();")
    } else {
        format!("    return {target}({});", args.join(", "))
    }
}

pub(crate) fn c_type_for_ast_type(ty: &AstTypeRef) -> &'static str {
    match ty.name.as_str() {
        "i32" => "int32_t",
        "i64" => "int64_t",
        "f32" => "float",
        "f64" => "double",
        "bool" => "int32_t",
        _ => "int64_t",
    }
}
