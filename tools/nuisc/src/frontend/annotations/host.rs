use std::collections::BTreeSet;

use nuis_semantics::model::{
    AstAttributeValue, AstExpr, AstFunction, AstModule, AstStmt, AstTypeRef,
};

const STD_HOST_SYMBOLS: &[(&str, &str)] = &[
    ("argv.at", "host_argv_at"),
    ("argv.count", "host_argv_count"),
    ("cache.close", "host_cache_close"),
    ("cache.lookup", "host_cache_lookup"),
    ("cache.open", "host_cache_open"),
    ("cache.store", "host_cache_store"),
    ("clock.domain_id", "host_clock_domain_id"),
    ("clock.epoch_ns", "host_clock_epoch_ns"),
    ("clock.scale_ppm", "host_clock_scale_ppm"),
    ("command.spawn", "host_command_spawn"),
    ("command.status", "host_command_status"),
    ("command.wait", "host_command_wait"),
    ("command.wait_exit", "host_command_wait_exit"),
    ("config.close", "host_config_close"),
    ("config.dir_handle", "host_config_dir_handle"),
    ("config.get", "host_config_get"),
    ("config.open", "host_config_open"),
    ("cwd.handle", "host_cwd_handle"),
    ("cwd.len", "host_cwd_len"),
    ("diag.emit", "host_diag_emit"),
    ("diag.label", "host_diag_label"),
    ("diag.span", "host_diag_span"),
    ("dir.close", "host_dir_close"),
    ("dir.create", "host_dir_create"),
    ("dir.entry_count", "host_dir_entry_count"),
    ("dir.open", "host_dir_open"),
    ("dir.remove", "host_dir_remove"),
    ("env.get", "host_env_get"),
    ("env.has", "host_env_has"),
    ("error.code", "host_error_code"),
    ("error.message", "host_error_message"),
    ("error.severity", "host_error_severity"),
    ("file.close", "host_file_close"),
    ("file.open", "host_file_open"),
    ("file.read", "host_file_read"),
    ("file.write", "host_file_write"),
    ("fs.exists", "host_fs_exists"),
    ("fs.kind", "host_fs_kind"),
    ("fs.size", "host_fs_size"),
    ("home.dir_handle", "host_home_dir_handle"),
    ("home.dir_len", "host_home_dir_len"),
    ("json.array_len", "host_json_array_len"),
    ("json.object_len", "host_json_object_len"),
    ("json.pair_len", "host_json_pair_len"),
    ("kv.close", "host_kv_close"),
    ("kv.get", "host_kv_get"),
    ("kv.open", "host_kv_open"),
    ("kv.put", "host_kv_put"),
    ("line.len", "host_line_len"),
    ("line.read", "host_line_read"),
    ("network.accept", "host_network_accept_probe"),
    ("network.accept_owned", "host_network_accept_owned"),
    ("network.bind_udp", "host_network_bind_udp_datagram"),
    ("network.close", "host_network_close"),
    ("network.close_owned", "host_network_close_owned"),
    ("network.connect", "host_network_connect_probe"),
    ("network.open_tcp", "host_network_open_tcp_stream"),
    (
        "network.open_tcp_listener",
        "host_network_open_tcp_listener",
    ),
    ("network.open_udp", "host_network_open_udp_datagram"),
    ("network.recv", "host_network_recv_probe"),
    (
        "network.recv_http_status_owned",
        "host_network_recv_http_status_owned",
    ),
    ("network.recv_owned", "host_network_recv_owned"),
    ("network.send", "host_network_send_probe"),
    ("network.send_owned", "host_network_send_owned"),
    ("path.basename", "host_path_basename"),
    ("path.basename_matches", "host_path_basename_matches"),
    ("path.copy", "host_path_copy"),
    ("path.depth", "host_path_depth"),
    ("path.ends_with_slash", "host_path_ends_with_slash"),
    ("path.extension", "host_path_extension"),
    ("path.extension_is", "host_path_extension_is"),
    ("path.filename", "host_path_filename"),
    ("path.filename_matches", "host_path_filename_matches"),
    ("path.has_extension", "host_path_has_extension"),
    ("path.has_parent", "host_path_has_parent"),
    ("path.is_absolute", "host_path_is_absolute"),
    ("path.is_basename_only", "host_path_is_basename_only"),
    ("path.is_dot", "host_path_is_dot"),
    ("path.is_dotdot", "host_path_is_dotdot"),
    ("path.is_empty", "host_path_is_empty"),
    ("path.is_hidden", "host_path_is_hidden"),
    ("path.is_relative", "host_path_is_relative"),
    ("path.is_root", "host_path_is_root"),
    ("path.join_len", "host_path_join_len"),
    ("path.matches_extension", "host_path_matches_extension"),
    ("path.parent", "host_path_parent"),
    ("path.parent_matches", "host_path_parent_matches"),
    ("path.remove", "host_path_remove"),
    ("path.rename", "host_path_rename"),
    ("path.starts_with_dot", "host_path_starts_with_dot"),
    ("path.stem", "host_path_stem"),
    ("path.stem_matches", "host_path_stem_matches"),
    ("process.exit_code", "host_process_exit_code"),
    ("process.id", "host_process_id"),
    ("process.status", "host_process_status"),
    ("result.error", "host_result_error"),
    ("result.is_ok", "host_result_is_ok"),
    ("result.value", "host_result_value"),
    ("sleep.ns", "host_sleep_ns"),
    ("stat.ctime_ns", "host_stat_ctime_ns"),
    ("stat.mode", "host_stat_mode"),
    ("stat.mtime_ns", "host_stat_mtime_ns"),
    ("stderr.flush", "host_stderr_flush"),
    ("stderr.write", "host_stderr_write"),
    ("stdin.read", "host_stdin_read"),
    ("stdout.flush", "host_stdout_flush"),
    ("stdout.write", "host_stdout_write"),
    ("subprocess.join", "host_subprocess_join"),
    ("subprocess.join_exit", "host_subprocess_join_exit"),
    ("subprocess.signal", "host_subprocess_signal"),
    ("subprocess.spawn", "host_subprocess_spawn"),
    ("temp.dir_handle", "host_temp_dir_handle"),
    ("temp.file_handle", "host_temp_file_handle"),
    ("temp.path_len", "host_temp_path_len"),
    ("text.concat", "host_text_concat"),
    ("text.concat_len", "host_text_concat_len"),
    ("text.format_i64", "host_text_format_i64"),
    ("text.format_pair", "host_text_format_pair"),
    ("text.len", "host_text_len"),
    ("text.measure", "host_text_measure"),
    ("time.monotonic_ns", "host_monotonic_time_ns"),
    ("time.wall_ns", "host_wall_time_ns"),
    ("tty.height", "host_tty_height"),
    ("tty.isatty", "host_tty_isatty"),
    ("tty.width", "host_tty_width"),
];

pub(crate) fn validate_export_annotations(module: &AstModule) -> Result<(), String> {
    let mut seen_export_names = BTreeSet::new();
    for function in &module.functions {
        let export_name = function_export_name(function)?;
        let Some(export_name) = export_name else {
            continue;
        };
        if module.domain != "cpu" {
            return Err(format!(
                "function `{}::{}` can only use `@export(name = \"...\")` inside `mod cpu` in the current MVP",
                module.unit, function.name
            ));
        }
        if function.name != "main" {
            return Err(format!(
                "function `{}` uses `@export(name = \"{}\")`, but only `fn main()` can be exported in the current MVP",
                function.name, export_name
            ));
        }
        if !function.params.is_empty() {
            return Err(format!(
                "function `{}` uses `@export(name = \"{}\")`, but exported `fn main()` cannot take parameters in the current MVP",
                function.name, export_name
            ));
        }
        if !seen_export_names.insert(export_name.clone()) {
            return Err(format!(
                "module `{} {}` repeats exported symbol `{}`",
                module.domain, module.unit, export_name
            ));
        }
    }
    Ok(())
}

fn function_export_name(function: &AstFunction) -> Result<Option<String>, String> {
    let Some(attribute) = function
        .attributes
        .iter()
        .find(|attribute| attribute.name == "export")
    else {
        return Ok(None);
    };
    let Some(arg) = attribute.args.first() else {
        return Ok(None);
    };
    match &arg.value {
        AstAttributeValue::String(value) => Ok(Some(value.clone())),
        _ => Err(format!(
            "function `{}` annotation `@export` expects `name = \"...\"`",
            function.name
        )),
    }
}

pub(crate) fn function_host_symbol_name(function: &AstFunction) -> Result<Option<String>, String> {
    let Some(attribute) = function
        .attributes
        .iter()
        .find(|attribute| attribute.name == "host_symbol")
    else {
        return Ok(None);
    };
    let Some(arg) = attribute.args.first() else {
        return Ok(None);
    };
    match &arg.value {
        AstAttributeValue::String(value) => resolve_std_host_symbol(value)
            .map(|value| Some(value.to_owned()))
            .ok_or_else(|| {
                format!(
                    "function `{}` annotation `@host_symbol(\"{}\")` is not a recognized std-owned host symbol",
                    function.name, value
                )
            }),
        _ => Err(format!(
            "function `{}` annotation `@host_symbol` expects a string literal",
            function.name
        )),
    }
}

pub(crate) fn extern_function_symbol_name(
    function: &nuis_semantics::model::AstExternFunction,
) -> Result<String, String> {
    match &function.host_symbol {
        Some(symbol) => resolve_std_host_symbol(symbol)
            .map(str::to_owned)
            .ok_or_else(|| {
                format!(
                    "extern function `{}` host symbol `{}` is not a recognized std-owned host symbol",
                    function.name, symbol
                )
            }),
        None => Ok(match &function.interface {
            Some(interface) => format!("{interface}__{}", function.name),
            None => function.name.clone(),
        }),
    }
}

pub(crate) fn resolve_std_host_symbol(symbol: &str) -> Option<&'static str> {
    STD_HOST_SYMBOLS
        .iter()
        .find_map(|(logical, lowered)| (*logical == symbol).then_some(*lowered))
}

pub(crate) fn validate_host_symbol_bridge_annotations(module: &AstModule) -> Result<(), String> {
    for function in &module.functions {
        let Some(attribute) = function
            .attributes
            .iter()
            .find(|attribute| attribute.name == "host_symbol")
        else {
            continue;
        };
        let Some(arg) = attribute.args.first() else {
            continue;
        };
        let AstAttributeValue::String(logical_symbol) = &arg.value else {
            continue;
        };
        if resolve_std_host_symbol(logical_symbol).is_none() {
            return Err(format!(
                "function `{}` annotation `@host_symbol(\"{}\")` is not a recognized std-owned host symbol",
                function.name, logical_symbol
            ));
        }
        if module.domain != "cpu" {
            return Err(format!(
                "function `{}::{}` can only use `@host_symbol(\"{}\")` inside `mod cpu` in the current MVP; prefer `extern \"c\" @host_symbol(...) fn ...;` for the stable host-boundary form",
                module.unit, function.name, logical_symbol
            ));
        }
        if function.is_async {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but async host bridge stubs are not supported in the current MVP; prefer `extern \"c\" @host_symbol(...) fn ...;` for stable host-boundary declarations",
                function.name, logical_symbol
            ));
        }
        if !function.generic_params.is_empty() {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but generic host bridge stubs are not supported in the current MVP; prefer `extern \"c\" @host_symbol(...) fn ...;` for stable host-boundary declarations",
                function.name, logical_symbol
            ));
        }
        if !function
            .params
            .iter()
            .all(|param| is_i64_type_ref(&param.ty))
        {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require all parameters to be `i64`; prefer the `extern \"c\" @host_symbol(...) fn ...;` form for the stable host-boundary path",
                function.name, logical_symbol
            ));
        }
        let Some(return_type) = &function.return_type else {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require `-> i64`; prefer the `extern \"c\" @host_symbol(...) fn ...;` form for the stable host-boundary path",
                function.name, logical_symbol
            ));
        };
        if !is_i64_type_ref(return_type) {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require `-> i64`; prefer the `extern \"c\" @host_symbol(...) fn ...;` form for the stable host-boundary path",
                function.name, logical_symbol
            ));
        }
        if !matches!(
            function.body.as_slice(),
            [AstStmt::Return(Some(AstExpr::Int(0)))]
        ) {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require a trivial `return 0;` body; prefer `extern \"c\" @host_symbol(...) fn ...;` for the stable host-boundary form",
                function.name, logical_symbol
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_extern_host_symbols(module: &AstModule) -> Result<(), String> {
    for function in &module.externs {
        validate_extern_host_symbol(module, function)?;
    }
    for interface in &module.extern_interfaces {
        for function in &interface.methods {
            validate_extern_host_symbol(module, function)?;
        }
    }
    Ok(())
}

fn validate_extern_host_symbol(
    module: &AstModule,
    function: &nuis_semantics::model::AstExternFunction,
) -> Result<(), String> {
    let Some(logical_symbol) = &function.host_symbol else {
        return Ok(());
    };
    if resolve_std_host_symbol(logical_symbol).is_none() {
        return Err(format!(
            "extern function `{}` annotation `@host_symbol(\"{}\")` is not a recognized std-owned host symbol",
            function.name, logical_symbol
        ));
    }
    if module.domain != "cpu" {
        return Err(format!(
            "extern function `{}::{}` can only use `@host_symbol(\"{}\")` inside `mod cpu` in the current MVP",
            module.unit, function.name, logical_symbol
        ));
    }
    if function.abi != "c" {
        return Err(format!(
            "extern function `{}` uses `@host_symbol(\"{}\")`, but std-owned host symbol externs currently require `extern \"c\"`",
            function.name, logical_symbol
        ));
    }
    if !function
        .params
        .iter()
        .all(|param| is_i64_type_ref(&param.ty))
        || !is_i64_type_ref(&function.return_type)
    {
        return Err(format!(
            "extern function `{}` uses `@host_symbol(\"{}\")`, but std-owned host symbol externs currently require only `i64` parameters and `-> i64`",
            function.name, logical_symbol
        ));
    }
    Ok(())
}

fn is_i64_type_ref(ty: &AstTypeRef) -> bool {
    ty.name == "i64" && ty.generic_args.is_empty() && !ty.is_optional && !ty.is_ref
}
