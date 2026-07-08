use super::*;

#[test]
fn c_shim_source_includes_native_file_stdin_and_tty_hooks() {
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
                name: "host_file_open".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "flags".to_owned(),
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
                name: "host_file_write".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "file_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "text_handle".to_owned(),
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
                name: "host_stdin_read".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
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
                name: "host_tty_width".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "fd".to_owned(),
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
    assert!(shim.contains("return nuis_host_file_open(path_handle, flags);"));
    assert!(shim.contains("return nuis_host_file_write(file_handle, text_handle);"));
    assert!(shim.contains("return nuis_host_stdin_read(buffer_handle, len);"));
    assert!(shim.contains("return nuis_host_tty_width(fd);"));
}

#[test]
fn c_shim_source_includes_network_control_hooks() {
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
                name: "host_network_connect_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "remote_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "connect_timeout_ms".to_owned(),
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
                name: "host_network_accept_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
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
                name: "host_network_open_tcp_listener".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
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
                name: "host_network_bind_udp_datagram".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
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
                name: "host_network_accept_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "listener_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
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
                name: "host_network_close".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_send_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "send_window".to_owned(),
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
                name: "host_network_recv_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "recv_window".to_owned(),
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
                name: "host_network_recv_http_status_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "recv_window".to_owned(),
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
                name: "host_network_send_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "send_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "remote_port".to_owned(),
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
                name: "host_network_recv_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "recv_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
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
    assert!(shim.contains("static int64_t nuis_lifecycle_network_enabled = 1;"));
    assert!(shim.contains("return nuis_host_network_fd_len;"));
    assert!(shim.contains(
        "return nuis_host_network_connect_probe(local_port, remote_port, connect_timeout_ms);"
    ));
    assert!(shim.contains(
        "return nuis_host_network_accept_probe(local_port, read_timeout_ms, write_timeout_ms);"
    ));
    assert!(shim.contains(
            "return nuis_host_network_open_tcp_listener(local_port, read_timeout_ms, write_timeout_ms);"
        ));
    assert!(shim.contains(
            "return nuis_host_network_bind_udp_datagram(local_port, read_timeout_ms, write_timeout_ms);"
        ));
    assert!(shim.contains(
            "return nuis_host_network_accept_owned(listener_handle, read_timeout_ms, write_timeout_ms);"
        ));
    assert!(shim.contains("return nuis_host_network_close(handle);"));
    assert!(
        shim.contains("return nuis_host_network_send_owned(handle, stream_window, send_window);")
    );
    assert!(
        shim.contains("return nuis_host_network_recv_owned(handle, stream_window, recv_window);")
    );
    assert!(shim.contains(
        "return nuis_host_network_recv_http_status_owned(handle, stream_window, recv_window);"
    ));
    assert!(shim
        .contains("return nuis_host_network_send_probe(stream_window, send_window, remote_port);"));
    assert!(shim
        .contains("return nuis_host_network_recv_probe(stream_window, recv_window, local_port);"));
}
