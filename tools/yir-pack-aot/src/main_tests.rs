use super::{
    append_host_ffi_manifest_entries, collect_host_ffi_symbols, default_host_ffi_registry,
    host_ffi_registry_abis, host_ffi_registry_hash, is_ffi_symbol_hash_token,
    parse_host_ffi_registry_lines, parse_manifest_string_array, render_host_ffi_stubs,
    render_host_ffi_symbol_hash_manifest, render_host_ffi_symbol_manifest,
    verify_host_ffi_manifest_against_registry, verify_host_ffi_manifest_against_registry_lines,
    verify_host_ffi_manifest_lines, HostFfiArgType, HostFfiReturnType, HostFfiSignature,
};
use std::collections::BTreeMap;
use yir_core::{Node, Operation, YirModule};

fn cpu_node(name: &str, instruction: &str, args: &[&str]) -> Node {
    Node {
        name: name.to_owned(),
        resource: "cpu.main".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        },
    }
}

fn extern_node(name: &str, instruction: &str, symbol: &str, args: &[&str]) -> Node {
    extern_node_with_abi("c", name, instruction, symbol, args)
}

fn extern_node_with_abi(
    abi: &str,
    name: &str,
    instruction: &str,
    symbol: &str,
    args: &[&str],
) -> Node {
    let mut op_args = vec!["c".to_owned(), symbol.to_owned()];
    op_args[0] = abi.to_owned();
    op_args.extend(args.iter().map(|arg| (*arg).to_owned()));
    cpu_node(
        name,
        instruction,
        &op_args.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

fn host_ffi_signature(
    abi: &str,
    return_type: HostFfiReturnType,
    arg_types: Vec<HostFfiArgType>,
) -> HostFfiSignature {
    HostFfiSignature {
        abi: abi.to_owned(),
        return_type,
        arg_types,
    }
}

fn i64_host_ffi_signature(abi: &str, arg_count: usize) -> HostFfiSignature {
    host_ffi_signature(
        abi,
        HostFfiReturnType::I64,
        vec![HostFfiArgType::I64; arg_count],
    )
}

fn insert_i64_host_ffi_symbol(
    symbols: &mut BTreeMap<String, HostFfiSignature>,
    abi: &str,
    symbol: &str,
    arg_count: usize,
) {
    symbols.insert(symbol.to_owned(), i64_host_ffi_signature(abi, arg_count));
}

fn registry_lines(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|line| (*line).to_owned()).collect()
}

#[test]
fn host_ffi_stub_tracks_i32_return_and_arg_type() {
    let mut module = YirModule::new("1");
    module.nodes.push(cpu_node("seed", "const_i32", &["7"]));
    module.nodes.push(extern_node(
        "curve",
        "extern_call_i32",
        "host_i32_curve",
        &["seed"],
    ));

    let symbols = collect_host_ffi_symbols(&module).unwrap();
    let signature = symbols.get("host_i32_curve").unwrap();
    assert_eq!(signature.return_type, HostFfiReturnType::I32);
    assert_eq!(signature.arg_types, vec![HostFfiArgType::I32]);
    assert_eq!(signature.render(), "i32(i32)");
    assert_eq!(signature.hash("host_i32_curve"), "fnv1a64:b0042e2b5ee2c2aa");

    let stubs = render_host_ffi_stubs(&symbols);
    assert!(stubs.contains("int32_t host_i32_curve(int32_t arg0)"));
}

#[test]
fn libc_ref_buffer_signature_is_recorded_but_not_stubbed() {
    let mut module = YirModule::new("1");
    module.nodes.push(cpu_node("fd", "const_i32", &["-1"]));
    module.nodes.push(cpu_node("len", "const_i64", &["8"]));
    module
        .nodes
        .push(cpu_node("scratch", "alloc_buffer", &["len", "len"]));
    module.nodes.push(extern_node_with_abi(
        "libc",
        "read_call",
        "extern_call_i64",
        "read",
        &["fd", "scratch", "len"],
    ));

    let symbols = collect_host_ffi_symbols(&module).unwrap();
    let signature = symbols.get("read").unwrap();
    assert_eq!(signature.abi, "libc");
    assert_eq!(
        signature.arg_types,
        vec![
            HostFfiArgType::I32,
            HostFfiArgType::RefBuffer,
            HostFfiArgType::I64
        ]
    );
    assert_eq!(signature.render(), "i64(i32,ref_Buffer,i64)");

    let stubs = render_host_ffi_stubs(&symbols);
    assert!(!stubs.contains(" read("));
}

#[test]
fn host_ffi_manifest_hashes_are_self_verifying() {
    let mut module = YirModule::new("1");
    module.nodes.push(cpu_node("lhs", "const_i32", &["7"]));
    module.nodes.push(cpu_node("rhs", "const_i32", &["5"]));
    module.nodes.push(extern_node(
        "mix",
        "extern_call_i64",
        "host_i32_mix",
        &["lhs", "rhs"],
    ));

    let symbols = collect_host_ffi_symbols(&module).unwrap();
    let symbol_manifest = render_host_ffi_symbol_manifest(&symbols);
    let hash_manifest = render_host_ffi_symbol_hash_manifest(&symbols);

    assert_eq!(symbol_manifest, "host_i32_mix@c:i64(i32,i32)");
    assert!(hash_manifest.starts_with("host_i32_mix:fnv1a64:"));
    verify_host_ffi_manifest_lines(&symbol_manifest, &hash_manifest).unwrap();
}

#[test]
fn host_ffi_manifest_hashes_reject_drift() {
    let mut module = YirModule::new("1");
    module.nodes.push(cpu_node("seed", "const_i32", &["7"]));
    module.nodes.push(extern_node(
        "curve",
        "extern_call_i32",
        "host_i32_curve",
        &["seed"],
    ));

    let symbols = collect_host_ffi_symbols(&module).unwrap();
    let symbol_manifest = render_host_ffi_symbol_manifest(&symbols);
    let error =
        verify_host_ffi_manifest_lines(&symbol_manifest, "host_i32_curve:fnv1a64:0000000000000000")
            .expect_err("mismatched host ffi hash should be rejected");

    assert!(error.contains("host ffi manifest hash mismatch for `host_i32_curve`"));
    assert!(error.contains("fnv1a64:b0042e2b5ee2c2aa"));
}

#[test]
fn host_ffi_manifest_line_verifier_rejects_abi_drift() {
    let error = verify_host_ffi_manifest_lines(
        "host_i32_curve@nurs:i32(i32)",
        "host_i32_curve:fnv1a64:b0042e2b5ee2c2aa",
    )
    .expect_err("manifest hash should bind the ABI as well as the signature");

    assert!(error.contains("host ffi manifest hash mismatch for `host_i32_curve`"));
}

#[test]
fn host_ffi_manifest_registry_verifier_accepts_registered_symbol() {
    let symbol_line = "host_i32_curve@c:i32(i32)";
    let hash_line = "host_i32_curve:fnv1a64:b0042e2b5ee2c2aa";
    verify_host_ffi_manifest_against_registry_lines(
        symbol_line,
        hash_line,
        &registry_lines(&["c:ffi_symbol:host_i32_curve=i32(i32)"]),
    )
    .unwrap();
}

#[test]
fn host_ffi_manifest_registry_verifier_accepts_parsed_registry_view() {
    let registry =
        parse_host_ffi_registry_lines(&registry_lines(&["c:ffi_symbol:host_i32_curve=i32(i32)"]))
            .unwrap();

    verify_host_ffi_manifest_against_registry(
        "host_i32_curve@c:i32(i32)",
        "host_i32_curve:fnv1a64:b0042e2b5ee2c2aa",
        &registry,
    )
    .unwrap();
}

#[test]
fn host_ffi_registry_manifest_parser_preserves_commas_inside_signatures() {
    let values = parse_manifest_string_array(
            r#"abi_capabilities = ["c:ffi_symbol:host_file_open=i64(i64,i64)", "c:ffi_symbol:host_stdout_write=i64(i64)"]"#,
            "abi_capabilities",
        )
        .expect("abi_capabilities array should parse");

    assert_eq!(
        values,
        vec![
            "c:ffi_symbol:host_file_open=i64(i64,i64)".to_owned(),
            "c:ffi_symbol:host_stdout_write=i64(i64)".to_owned(),
        ]
    );
}

#[test]
fn parsed_host_ffi_registry_manifest_lines_are_registry_compatible() {
    let values = parse_manifest_string_array(
            r#"abi_capabilities = ["c:ffi_symbol:host_file_open=i64(i64,i64)|ffi_symbol:host_stdout_write=i64(i64)", "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)"]"#,
            "abi_capabilities",
        )
        .expect("abi_capabilities array should parse");
    let registry = parse_host_ffi_registry_lines(&values).unwrap();

    assert!(registry.contains_key(&("c".to_owned(), "host_file_open".to_owned())));
    assert!(registry.contains_key(&("c".to_owned(), "host_stdout_write".to_owned())));
    assert!(registry.contains_key(&("nurs".to_owned(), "HostMath__speed_curve".to_owned())));
}

#[test]
fn default_host_ffi_registry_loads_cpu_manifest_facades() {
    let registry_lines = default_host_ffi_registry();
    assert!(registry_lines.source.starts_with("cpu-manifest:"));
    let registry = parse_host_ffi_registry_lines(&registry_lines.lines).unwrap();
    assert!(registry.contains_key(&("c".to_owned(), "host_stdout_write".to_owned())));
    assert!(registry.contains_key(&("c".to_owned(), "host_file_open".to_owned())));
    assert!(registry.contains_key(&("c".to_owned(), "host_command_spawn_in".to_owned())));
    assert!(registry.contains_key(&("libc".to_owned(), "getpid".to_owned())));
    assert!(registry.contains_key(&("libc".to_owned(), "usleep".to_owned())));
    assert!(registry.contains_key(&("libc".to_owned(), "puts".to_owned())));
    assert!(registry.contains_key(&("libc".to_owned(), "strlen".to_owned())));
    assert!(registry.contains_key(&("libc".to_owned(), "write".to_owned())));
    assert!(registry.contains_key(&("libc".to_owned(), "close".to_owned())));
    assert!(registry.contains_key(&("libc".to_owned(), "read".to_owned())));
}

#[test]
fn host_ffi_manifest_entries_record_registry_source() {
    let mut symbols = BTreeMap::new();
    insert_i64_host_ffi_symbol(&mut symbols, "c", "host_stdout_write", 1);

    let mut manifest = Vec::new();
    append_host_ffi_manifest_entries(&mut manifest, &symbols).unwrap();

    assert!(manifest
        .iter()
        .any(|line| line == "host_ffi_symbols=host_stdout_write@c:i64(i64)"));
    assert!(manifest
        .iter()
        .any(|line| line.starts_with("host_ffi_symbol_hashes=host_stdout_write:fnv1a64:")));
    let footprint_hash = manifest
        .iter()
        .find_map(|line| line.strip_prefix("host_ffi_footprint_hash="))
        .expect("footprint hash should be recorded");
    assert!(is_ffi_symbol_hash_token(footprint_hash));
    assert!(manifest
        .iter()
        .any(|line| line == "host_ffi_used_symbols=1"));
    assert!(manifest.iter().any(|line| line == "host_ffi_used_abis=c"));
    assert!(manifest
        .iter()
        .any(|line| line.starts_with("host_ffi_registry_source=cpu-manifest:")));
    assert!(manifest
        .iter()
        .any(|line| line == "host_ffi_registry_abis=c,libc,nurs"));
    let registry_hash = manifest
        .iter()
        .find_map(|line| line.strip_prefix("host_ffi_registry_hash="))
        .expect("registry hash should be recorded");
    assert!(is_ffi_symbol_hash_token(registry_hash));
    let registry_line_count = manifest
        .iter()
        .find_map(|line| line.strip_prefix("host_ffi_registry_lines="))
        .and_then(|value| value.parse::<usize>().ok())
        .expect("registry line count should be recorded");
    let registry_symbol_count = manifest
        .iter()
        .find_map(|line| line.strip_prefix("host_ffi_registry_symbols="))
        .and_then(|value| value.parse::<usize>().ok())
        .expect("registry symbol count should be recorded");
    assert!(registry_line_count > 0);
    assert!(registry_symbol_count > 0);
}

#[test]
fn host_ffi_manifest_entries_record_no_registry_when_unused() {
    let mut manifest = Vec::new();
    append_host_ffi_manifest_entries(&mut manifest, &BTreeMap::new()).unwrap();

    assert_eq!(
        manifest,
        vec![
            "host_ffi_symbols=none".to_owned(),
            "host_ffi_symbol_hashes=none".to_owned(),
            "host_ffi_footprint_hash=none".to_owned(),
            "host_ffi_used_symbols=0".to_owned(),
            "host_ffi_used_abis=none".to_owned(),
            "host_ffi_registry_source=none".to_owned(),
            "host_ffi_registry_lines=0".to_owned(),
            "host_ffi_registry_symbols=0".to_owned(),
            "host_ffi_registry_abis=none".to_owned(),
            "host_ffi_registry_hash=none".to_owned(),
        ]
    );
}

#[test]
fn host_ffi_registry_hash_is_order_stable() {
    let lhs = registry_lines(&[
        "c:ffi_symbol:host_stdout_write=i64(i64)",
        "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)",
    ]);
    let rhs = registry_lines(&[
        "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)",
        "c:ffi_symbol:host_stdout_write=i64(i64)",
    ]);

    assert_eq!(host_ffi_registry_hash(&lhs), host_ffi_registry_hash(&rhs));
    assert!(is_ffi_symbol_hash_token(&host_ffi_registry_hash(&lhs)));
}

#[test]
fn host_ffi_registry_abis_are_sorted_and_deduplicated() {
    let registry = parse_host_ffi_registry_lines(&registry_lines(&[
            "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)",
            "libc:ffi_symbol:getpid=i32()|ffi_symbol:usleep=i32(i32)|ffi_symbol:puts=i32(String)|ffi_symbol:strlen=i64(String)|ffi_symbol:write=i64(i32,String,i64)|ffi_symbol:close=i32(i32)|ffi_symbol:read=i64(i32,ref_Buffer,i64)",
            "c:ffi_symbol:host_stdout_write=i64(i64)|ffi_symbol:host_stderr_write=i64(i64)",
        ]))
        .unwrap();

    assert_eq!(host_ffi_registry_abis(&registry), "c,libc,nurs");
}

#[test]
fn default_host_ffi_registry_accepts_all_builtin_stub_symbols() {
    let mut symbols = BTreeMap::new();
    for (symbol, arg_count) in [
        ("HostRenderCurves__color_bias", 1),
        ("HostRenderCurves__speed_curve", 1),
        ("HostRenderCurves__radius_curve", 1),
        ("HostRenderCurves__mix_tick", 2),
        ("HostMath__speed_curve", 1),
    ] {
        insert_i64_host_ffi_symbol(&mut symbols, "nurs", symbol, arg_count);
    }
    for (symbol, arg_count) in [
        ("host_speed_curve", 1),
        ("host_hashed_curve", 1),
        ("host_argv_count", 0),
        ("host_monotonic_time_ns", 0),
        ("host_network_connect_probe", 3),
        ("host_network_open_tcp_stream", 2),
        ("host_network_open_tcp_listener", 3),
        ("host_network_open_udp_datagram", 2),
        ("host_network_bind_udp_datagram", 3),
        ("host_network_accept_owned", 3),
        ("host_network_close_owned", 1),
        ("host_network_send_owned", 3),
        ("host_network_recv_owned", 3),
        ("host_network_recv_http_status_owned", 3),
        ("host_network_accept_probe", 3),
        ("host_network_close", 1),
        ("host_network_send_probe", 3),
        ("host_network_recv_probe", 3),
        ("host_stdout_write", 1),
        ("host_file_open", 2),
        ("host_serialize_i64_into", 3),
        ("host_command_spawn_in", 4),
    ] {
        insert_i64_host_ffi_symbol(&mut symbols, "c", symbol, arg_count);
    }
    symbols.insert(
        "host_i32_curve".to_owned(),
        host_ffi_signature("c", HostFfiReturnType::I32, vec![HostFfiArgType::I32]),
    );

    let symbol_manifest = render_host_ffi_symbol_manifest(&symbols);
    let hash_manifest = render_host_ffi_symbol_hash_manifest(&symbols);

    verify_host_ffi_manifest_lines(&symbol_manifest, &hash_manifest).unwrap();
    verify_host_ffi_manifest_against_registry_lines(
        &symbol_manifest,
        &hash_manifest,
        &default_host_ffi_registry().lines,
    )
    .unwrap();
}

#[test]
fn host_ffi_manifest_registry_verifier_rejects_unregistered_symbol() {
    let error = verify_host_ffi_manifest_against_registry_lines(
        "host_unregistered@c:i64(i64)",
        "host_unregistered:fnv1a64:f8a191df2b6270f9",
        &registry_lines(&["c:ffi_symbol:host_i32_curve=i32(i32)"]),
    )
    .expect_err("unregistered host ffi symbol should be rejected");

    assert!(error.contains("host ffi symbol `host_unregistered` ABI `c` is not registered"));
}

#[test]
fn host_ffi_collection_rejects_conflicting_symbol_signatures() {
    let mut module = YirModule::new("1");
    module.nodes.push(extern_node(
        "curve_i64",
        "extern_call_i64",
        "host_curve",
        &["seed"],
    ));
    module.nodes.push(extern_node(
        "curve_i32",
        "extern_call_i32",
        "host_curve",
        &["seed"],
    ));

    let error = collect_host_ffi_symbols(&module)
        .expect_err("same host symbol with different return width should be rejected");
    assert!(error.contains("host ffi symbol `host_curve` is used with conflicting signatures"));
    assert!(error.contains("i64(i64)"));
    assert!(error.contains("i32(i64)"));
}
