use super::*;

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
