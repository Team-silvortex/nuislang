use super::*;

#[test]
fn c_shim_source_includes_native_directory_temp_and_process_hooks() {
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
                name: "host_dir_open".to_owned(),
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
                name: "host_dir_create".to_owned(),
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
                name: "host_dir_remove".to_owned(),
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
                name: "host_path_rename".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
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
                name: "host_path_copy".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
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
                name: "host_path_remove".to_owned(),
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
                name: "host_temp_file_handle".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "prefix_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_command_spawn".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "program_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "argv_handle".to_owned(),
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
                name: "host_command_spawn_in".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "program_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "argv_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "cwd_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
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
    assert!(shim.contains("return nuis_host_dir_open(path_handle);"));
    assert!(shim.contains("return nuis_host_dir_create(path_handle);"));
    assert!(shim.contains("return nuis_host_dir_remove(path_handle);"));
    assert!(shim.contains("return nuis_host_path_rename(src_handle, dst_handle);"));
    assert!(shim.contains("return nuis_host_path_copy(src_handle, dst_handle);"));
    assert!(shim.contains("return nuis_host_path_remove(path_handle);"));
    assert!(shim.contains("return nuis_host_temp_file_handle(prefix_handle);"));
    assert!(shim.contains("return nuis_host_command_spawn(program_handle, argv_handle);"));
    assert!(shim.contains(
        "return nuis_host_command_spawn_in(program_handle, argv_handle, cwd_handle, timeout_ms);"
    ));
    assert!(shim.contains("static char* nuis_host_build_shell_command("));
    assert!(shim.contains("env %s %s %s"));
    assert!(shim.contains("static int64_t nuis_host_command_spawn_in("));
}

#[test]
fn c_shim_source_includes_native_command_and_subprocess_exit_hooks() {
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
                name: "host_subprocess_spawn_in".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "program_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "argv_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "env_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "cwd_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "timeout_ms".to_owned(),
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
                name: "host_command_wait_exit".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "command_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_subprocess_join_exit".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "process_handle".to_owned(),
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
    assert!(shim.contains("static int64_t nuis_host_command_wait_exit("));
    assert!(shim.contains("static int64_t nuis_host_subprocess_join_exit("));
    assert!(shim.contains("static int64_t nuis_host_subprocess_spawn_in("));
    assert!(shim.contains(
            "return nuis_host_subprocess_spawn_in(program_handle, argv_handle, env_handle, cwd_handle, timeout_ms);"
        ));
    assert!(shim.contains("return nuis_host_command_wait_exit(command_handle);"));
    assert!(shim.contains("return nuis_host_subprocess_join_exit(process_handle);"));
}

#[test]
fn c_shim_source_includes_native_text_concat_hook() {
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
        externs: vec![AstExternFunction {
            visibility: AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: "host_text_concat".to_owned(),
            params: vec![
                nuis_semantics::model::AstParam {
                    name: "lhs_handle".to_owned(),
                    ty: i64_ty(),
                },
                nuis_semantics::model::AstParam {
                    name: "rhs_handle".to_owned(),
                    ty: i64_ty(),
                },
            ],
            return_type: i64_ty(),
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
    assert!(shim.contains("static int64_t nuis_host_text_concat("));
    assert!(shim.contains("return nuis_host_text_concat(lhs_handle, rhs_handle);"));
}
