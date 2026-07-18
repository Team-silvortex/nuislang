use super::*;

#[test]
fn owned_invoker_null_payload_reaches_failed_terminal_state() {
    let dir = temp_dir("owned_invoker_failed_state");
    let source_path = dir.join("owned_invoker_failed_state.c");
    let binary_path = dir.join("owned_invoker_failed_state");
    let mut source = String::new();
    crate::aot_c_shim_runtime::append_c_shim_prelude(&mut source, "0", "0", 0);
    crate::aot_c_shim_runtime::append_c_shim_lifecycle_runtime(&mut source);
    crate::aot_c_shim_text_runtime::append_c_shim_text_runtime(&mut source);
    source.push_str(
        r#"
static int64_t fail_owned_invoker(void* context) {
    (void)context;
    return 0;
}

static void drop_owned_payload(void* payload) {
    free(payload);
}

int64_t nuis_yir_entry(void) {
    void* context = malloc(1);
    int64_t task = nuis_scheduler_task_spawn_owned_invoker_v1(
        fail_owned_invoker, context, 8, 8, 17, drop_owned_payload
    );
    if (task == 0) return 10;
    (void)nuis_lifecycle_tick_once_v1();
    return nuis_scheduler_task_join_state_v1(task) == 4 ? 0 : 11;
}
"#,
    );
    crate::aot_c_shim_runtime::append_c_shim_main(&mut source);
    fs::write(&source_path, source).expect("write owned invoker failure harness");

    let compile = Command::new("clang")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("compile owned invoker failure harness");
    assert!(
        compile.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&binary_path)
        .output()
        .expect("run owned invoker failure harness");
    assert_eq!(run.status.code(), Some(0));
}

#[test]
fn owned_blob_crosses_scheduler_with_glm_token_and_deep_copy() {
    let dir = temp_dir("owned_blob_scheduler_roundtrip");
    let source_path = dir.join("owned_blob_scheduler_roundtrip.c");
    let binary_path = dir.join("owned_blob_scheduler_roundtrip");
    let mut source = String::new();
    crate::aot_c_shim_runtime::append_c_shim_prelude(&mut source, "0", "0", 0);
    crate::aot_c_shim_runtime::append_c_shim_lifecycle_runtime(&mut source);
    crate::aot_c_shim_text_runtime::append_c_shim_text_runtime(&mut source);
    crate::aot_c_shim_owned_blob_runtime::append_c_shim_owned_blob_runtime(&mut source);
    source.push_str(
        r#"
int64_t nuis_yir_entry(void) {
    unsigned char source_bytes[4] = {3, 5, 8, 13};
    void* blob = nuis_scheduler_owned_blob_copy_v1(source_bytes, 4, 41);
    if (blob == NULL) return 10;
    if (nuis_scheduler_owned_blob_copy_v1(source_bytes, 4, 0) != NULL) return 11;
    NuisSchedulerOwnedPayloadV1 payload = {
        .data = blob,
        .size = (int64_t)(sizeof(NuisSchedulerOwnedBlobV1) + 4),
        .alignment = 8,
        .type_id = 91,
        .move_hook = nuis_scheduler_owned_blob_move_v1,
        .drop_hook = nuis_scheduler_owned_blob_drop_v1,
    };
    int64_t task = nuis_scheduler_task_spawn_owned_v1(&payload);
    if (task == 0) return 12;
    source_bytes[0] = 99;
    if (nuis_scheduler_task_join_state_v1(task) != 1) return 13;
    NuisSchedulerOwnedPayloadV1 taken = {0};
    if (nuis_scheduler_task_take_owned_v1(task, &taken) != 1) return 14;
    if (nuis_scheduler_owned_blob_len_v1(taken.data) != 4) return 15;
    if (nuis_scheduler_owned_blob_glm_token_v1(taken.data) != 41) return 16;
    const unsigned char* bytes =
        (const unsigned char*)nuis_scheduler_owned_blob_data_v1(taken.data);
    if (bytes == NULL || bytes[0] != 3 || bytes[3] != 13) return 17;
    nuis_scheduler_owned_payload_drop_v1(&taken);

    int64_t text_handle = nuis_host_text_lift("aggregate-text");
    void* poisoned_blob = nuis_scheduler_owned_blob_copy_text_v1(text_handle, 42);
    void* poisoned = nuis_scheduler_owned_aggregate_alloc_v1(2);
    if (!nuis_scheduler_owned_aggregate_set_blob_v1(poisoned, 0, poisoned_blob)) return 18;
    if (nuis_scheduler_owned_aggregate_set_scalar_v1(poisoned, 0, 99)) return 19;
    if (nuis_scheduler_owned_aggregate_finish_v1(poisoned) != NULL) return 20;
    if (nuis_scheduler_owned_blob_live_count_get_v1() != 0) return 21;

    void* cancelled_blob = nuis_scheduler_owned_blob_copy_text_v1(text_handle, 42);
    void* aggregate = nuis_scheduler_owned_aggregate_alloc_v1(2);
    if (aggregate == NULL) return 22;
    if (!nuis_scheduler_owned_aggregate_set_scalar_v1(aggregate, 0, 144)) return 23;
    if (!nuis_scheduler_owned_aggregate_set_blob_v1(aggregate, 1, cancelled_blob)) return 24;
    aggregate = nuis_scheduler_owned_aggregate_finish_v1(aggregate);
    if (aggregate == NULL) return 25;
    NuisSchedulerOwnedPayloadV1 cancelled_payload = {
        .data = aggregate,
        .size = (int64_t)(sizeof(NuisSchedulerOwnedAggregateV1)
            + 2 * sizeof(NuisSchedulerOwnedAggregateSlotV1)),
        .alignment = 8,
        .type_id = 92,
        .move_hook = NULL,
        .drop_hook = nuis_scheduler_owned_aggregate_drop_v1,
    };
    int64_t cancelled = nuis_scheduler_task_spawn_owned_v1(&cancelled_payload);
    if (cancelled == 0) return 26;
    nuis_scheduler_task_cancel_v1(cancelled);
    if (nuis_scheduler_task_join_state_v1(cancelled) != 3) return 27;
    if (nuis_scheduler_owned_blob_live_count_get_v1() != 0) return 28;

    unsigned char moved_bytes[3] = {21, 34, 55};
    void* moved_blob = nuis_scheduler_owned_blob_copy_v1(moved_bytes, 3, 93);
    void* moved_aggregate = nuis_scheduler_owned_aggregate_alloc_v1(1);
    if (!nuis_scheduler_owned_aggregate_set_blob_v1(moved_aggregate, 0, moved_blob)) return 39;
    moved_aggregate = nuis_scheduler_owned_aggregate_finish_v1(moved_aggregate);
    if (moved_aggregate == NULL) return 40;
    void* taken_blob = nuis_scheduler_owned_aggregate_take_blob_v1(moved_aggregate, 0);
    if (taken_blob == NULL) return 41;
    nuis_scheduler_owned_aggregate_drop_v1(moved_aggregate);
    if (nuis_scheduler_owned_blob_live_count_get_v1() != 1) return 42;
    const unsigned char* taken_bytes =
        (const unsigned char*)nuis_scheduler_owned_blob_data_v1(taken_blob);
    if (taken_bytes == NULL || taken_bytes[0] != 21 || taken_bytes[2] != 55) return 43;
    nuis_scheduler_owned_blob_drop_v1(taken_blob);
    if (nuis_scheduler_owned_blob_live_count_get_v1() != 0) return 44;

    unsigned char valid_utf8[7] = {0xe4, 0xbd, 0xa0, 0xe5, 0xa5, 0xbd, 0};
    void* valid_text_blob = nuis_scheduler_owned_blob_copy_v1(valid_utf8, 7, 43);
    int64_t valid_text = nuis_scheduler_owned_blob_text_lift_v1(valid_text_blob);
    if (valid_text == 0 || nuis_host_text_lookup_len(valid_text) != 6) return 29;
    nuis_scheduler_owned_blob_drop_v1(valid_text_blob);

    unsigned char overlong_utf8[3] = {0xc0, 0xaf, 0};
    unsigned char surrogate_utf8[4] = {0xed, 0xa0, 0x80, 0};
    unsigned char truncated_utf8[3] = {0xe2, 0x82, 0};
    unsigned char out_of_range_utf8[5] = {0xf4, 0x90, 0x80, 0x80, 0};
    const unsigned char* invalid_utf8[] = {
        overlong_utf8, surrogate_utf8, truncated_utf8, out_of_range_utf8
    };
    const int64_t invalid_lens[] = {3, 4, 3, 5};
    for (int index = 0; index < 4; index += 1) {
        void* invalid_blob = nuis_scheduler_owned_blob_copy_v1(
            invalid_utf8[index], invalid_lens[index], (uint64_t)(44 + index)
        );
        if (invalid_blob == NULL) return 30 + index;
        if (nuis_scheduler_owned_blob_text_lift_v1(invalid_blob) != 0) return 34 + index;
        nuis_scheduler_owned_blob_drop_v1(invalid_blob);
    }
    return nuis_scheduler_owned_blob_live_count_get_v1() == 0 ? 0 : 38;
}
"#,
    );
    crate::aot_c_shim_runtime::append_c_shim_main(&mut source);
    fs::write(&source_path, source).expect("write owned blob scheduler harness");

    let compile = Command::new("clang")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("compile owned blob scheduler harness");
    assert!(
        compile.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&binary_path)
        .output()
        .expect("run owned blob scheduler harness");
    assert_eq!(run.status.code(), Some(0));
}

#[test]
fn lifecycle_shutdown_releases_text_intern_pool() {
    let dir = temp_dir("shutdown_text_pool");
    let source_path = dir.join("shutdown_text_pool.c");
    let binary_path = dir.join("shutdown_text_pool");
    let mut source = String::new();
    crate::aot_c_shim_runtime::append_c_shim_prelude(&mut source, "0", "0", 0);
    crate::aot_c_shim_runtime::append_c_shim_lifecycle_runtime(&mut source);
    crate::aot_c_shim_text_runtime::append_c_shim_text_runtime(&mut source);
    source.push_str(
        r#"
int64_t nuis_yir_entry(void) {
    if (nuis_host_text_lift("shutdown-owned-text") == 0) return 10;
    if (nuis_host_text_len != 1) return 11;
    if (nuis_lifecycle_shutdown_v1(0) != 0) return 12;
    if (nuis_host_text_len != 0) return 13;
    if (nuis_host_text_slots[0] != NULL) return 14;
    return 0;
}
"#,
    );
    crate::aot_c_shim_runtime::append_c_shim_main(&mut source);
    fs::write(&source_path, source).expect("write shutdown text pool harness");

    let compile = Command::new("clang")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("compile shutdown text pool harness");
    assert!(
        compile.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&binary_path)
        .output()
        .expect("run shutdown text pool harness");
    assert_eq!(run.status.code(), Some(0));
}

#[test]
fn lifecycle_contract_expands_export_surface_for_network_and_hetero_domains() {
    let envelope = NuisExecutableEnvelope {
        schema: "nuis-executable-envelope-v1".to_owned(),
        executable_kind: "native-cpu-llvm".to_owned(),
        package_count: 3,
        domain_families: vec!["cpu".to_owned(), "network".to_owned(), "kernel".to_owned()],
        contract_families: vec![
            "nustar.cpu".to_owned(),
            "nustar.network".to_owned(),
            "nustar.kernel".to_owned(),
        ],
        function_kind: "function-node".to_owned(),
        graph_kind: "function-graph".to_owned(),
        default_time_mode: "host-monotonic".to_owned(),
    };

    let lifecycle = build_nuis_lifecycle_contract(&envelope, "native-cpu-llvm");
    assert!(lifecycle
        .hook_surface
        .contains(&"on_network_bridge_progress".to_owned()));
    assert!(lifecycle
        .hook_surface
        .contains(&"on_hetero_submission_progress".to_owned()));
    assert!(lifecycle
        .export_surface
        .contains(&"nuis_lifecycle_network_bridge_progress_export_v1".to_owned()));
    assert!(lifecycle
        .export_surface
        .contains(&"nuis_lifecycle_hetero_submission_progress_export_v1".to_owned()));
    assert_eq!(lifecycle.export_surface.len(), 6);
    assert!(lifecycle
        .runtime_capability_flags
        .contains(&"runtime.progress.network".to_owned()));
    assert!(lifecycle
        .runtime_capability_flags
        .contains(&"runtime.progress.hetero".to_owned()));
}

#[test]
fn c_shim_source_enables_hetero_lifecycle_surface_for_shader_modules() {
    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "shader".to_owned(),
        unit: "SurfaceShader".to_owned(),
        externs: Vec::new(),
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
    assert!(shim.contains("static int64_t nuis_lifecycle_network_enabled = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_enabled = 1;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_surface_slots = 1;"));
    assert!(shim.contains("return nuis_lifecycle_hetero_surface_slots;"));
}

#[test]
fn c_shim_source_includes_native_env_path_and_stat_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
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
        externs: vec![
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_env_has".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "key_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_basename".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_filename".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_basename_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_filename_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_parent_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_stem_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_parent".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_has_parent".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_basename_only".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_depth".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_empty".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_dot".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_dotdot".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_relative".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_root".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_stem".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_extension".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_has_extension".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_matches_extension".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "ext_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_extension_is".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "ext_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_starts_with_dot".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_ends_with_slash".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_hidden".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_stat_mode".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
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
    assert!(shim.contains("return nuis_host_env_has(key_handle);"));
    assert!(shim.contains("return nuis_host_path_is_empty(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_dot(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_dotdot(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_relative(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_root(path_handle);"));
    assert!(shim.contains("return nuis_host_path_basename(path_handle);"));
    assert!(shim.contains("return nuis_host_path_filename(path_handle);"));
    assert!(shim.contains("return nuis_host_path_basename_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_filename_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_parent_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_stem_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_parent(path_handle);"));
    assert!(shim.contains("return nuis_host_path_has_parent(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_basename_only(path_handle);"));
    assert!(shim.contains("return nuis_host_path_depth(path_handle);"));
    assert!(shim.contains("return nuis_host_path_stem(path_handle);"));
    assert!(shim.contains("return nuis_host_path_extension(path_handle);"));
    assert!(shim.contains("return nuis_host_path_has_extension(path_handle);"));
    assert!(shim.contains("return nuis_host_path_matches_extension(path_handle, ext_handle);"));
    assert!(shim.contains("return nuis_host_path_extension_is(path_handle, ext_handle);"));
    assert!(shim.contains("return nuis_host_path_starts_with_dot(path_handle);"));
    assert!(shim.contains("return nuis_host_path_ends_with_slash(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_hidden(path_handle);"));
    assert!(shim.contains("return nuis_host_stat_mode(path_handle);"));
}
