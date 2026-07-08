use super::*;

#[test]
fn c_shim_source_includes_native_serialization_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    fn host_extern(name: &str, params: &[&str]) -> AstExternFunction {
        AstExternFunction {
            visibility: AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: name.to_owned(),
            params: params
                .iter()
                .map(|param| nuis_semantics::model::AstParam {
                    name: (*param).to_owned(),
                    ty: i64_ty(),
                })
                .collect(),
            return_type: i64_ty(),
            host_symbol: None,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            host_extern(
                "host_serialize_text_into",
                &["text_handle", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_serialize_i64_into",
                &["value", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_serialize_bool_into",
                &["value", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_serialize_byte_into",
                &["value", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_deserialize_i64_from",
                &["buffer_handle", "offset", "len"],
            ),
            host_extern("host_deserialize_byte_from", &["buffer_handle", "offset"]),
            host_extern(
                "host_deserialize_bool_from",
                &["buffer_handle", "offset", "len"],
            ),
            host_extern(
                "host_deserialize_text_from",
                &["buffer_handle", "offset", "len"],
            ),
            host_extern(
                "host_fill_bytes",
                &["buffer_handle", "offset", "len", "value"],
            ),
            host_extern(
                "host_copy_bytes",
                &[
                    "dst_handle",
                    "dst_offset",
                    "dst_len",
                    "src_handle",
                    "src_offset",
                    "src_len",
                ],
            ),
            host_extern(
                "host_compare_bytes",
                &[
                    "lhs_handle",
                    "lhs_offset",
                    "lhs_len",
                    "rhs_handle",
                    "rhs_offset",
                    "rhs_len",
                ],
            ),
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("static int64_t nuis_host_serialize_text_into("));
    assert!(shim.contains("static int64_t nuis_host_text_line_count("));
    assert!(shim.contains("static int64_t nuis_host_text_word_count("));
    assert!(shim.contains("static int64_t nuis_host_serialize_i64_into("));
    assert!(shim.contains("static int64_t nuis_host_serialize_bool_into("));
    assert!(shim.contains("static int64_t nuis_host_serialize_byte_into("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_i64_from("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_byte_from("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_bool_from("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_from("));
    assert!(shim.contains("static int64_t nuis_host_parse_header_line("));
    assert!(shim.contains("static int64_t nuis_host_find_header_value("));
    assert!(shim.contains("static int64_t nuis_host_find_status_line_reason("));
    assert!(shim.contains("static int64_t nuis_host_parse_http_response_summary("));
    assert!(shim.contains("static int64_t nuis_host_parse_http_request_summary("));
    assert!(shim.contains("static int64_t nuis_host_parse_http_roundtrip_summary("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_equals("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_starts_with("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_contains("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_ends_with("));
    assert!(shim.contains("static int64_t nuis_host_buffer_find_byte("));
    assert!(shim.contains("static int64_t nuis_host_fill_bytes("));
    assert!(shim.contains("static int64_t nuis_host_copy_bytes("));
    assert!(shim.contains("static int64_t nuis_host_compare_bytes("));
    assert!(shim.contains("static int64_t nuis_host_buffer_find_text("));
    assert!(shim.contains("static int64_t nuis_host_buffer_find_line_end("));
    assert!(shim.contains("static int64_t nuis_host_buffer_trim_line_end("));
    assert!(
        shim.contains("return nuis_host_serialize_text_into(text_handle, buffer_handle, offset);")
    );
    assert!(shim.contains("return nuis_host_serialize_i64_into(value, buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_serialize_bool_into(value, buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_serialize_byte_into(value, buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_deserialize_i64_from(buffer_handle, offset, len);"));
    assert!(shim.contains("return nuis_host_deserialize_byte_from(buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_deserialize_bool_from("));
    assert!(shim.contains("return nuis_host_deserialize_text_from("));
    assert!(shim.contains("return nuis_host_fill_bytes("));
    assert!(shim.contains("return nuis_host_copy_bytes("));
    assert!(shim.contains("return nuis_host_compare_bytes("));
}

#[test]
fn c_shim_source_leaves_plain_system_externs_unstubbed() {
    fn ty(name: &str) -> AstTypeRef {
        AstTypeRef {
            name: name.to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![AstExternFunction {
            visibility: AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: "usleep".to_owned(),
            params: vec![nuis_semantics::model::AstParam {
                name: "usec".to_owned(),
                ty: ty("i64"),
            }],
            return_type: ty("i32"),
            host_symbol: None,
        }],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(!shim.contains("int32_t usleep("));
}

#[test]
fn c_shim_source_includes_exported_main_wrapper() {
    fn ty(name: &str) -> AstTypeRef {
        AstTypeRef {
            name: name.to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: Vec::new(),
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: vec![nuis_semantics::model::AstFunction {
            name: "main".to_owned(),
            visibility: nuis_semantics::model::AstVisibility::Private,
            attributes: vec![nuis_semantics::model::AstAttribute {
                name: "export".to_owned(),
                args: vec![nuis_semantics::model::AstAttributeArg {
                    name: Some("name".to_owned()),
                    value: nuis_semantics::model::AstAttributeValue::String(
                        "entry_main".to_owned(),
                    ),
                }],
            }],
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: Vec::new(),
            where_bounds: Vec::new(),
            params: Vec::new(),
            return_type: Some(ty("i64")),
            body: Vec::new(),
        }],
    };

    let shim = c_shim_source(&ast);
    assert!(shim.contains("int64_t entry_main(void) {"));
    assert!(shim.contains("return nuis_yir_entry();"));
}
