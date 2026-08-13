use super::{lower_project_ast_to_nir, parse_nuis_ast, parse_nuis_module};
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn rejects_ref_parameter_in_extern_function_signature() {
    let error = parse_nuis_module(
        r#"
        extern "c" fn host_take_ptr(head: ref Node) -> i64;
        mod cffi Main {
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("extern function `host_take_ptr` parameter `head`"));
    assert!(error.contains("non-optional `ref Buffer` bridge values"));
    assert!(error.contains("hash-bound owned `ref String` returns"));
}

#[test]
fn rejects_ref_return_in_extern_interface_signature() {
    let error = parse_nuis_module(
        r#"
        extern "c" interface Nodes {
          fn head() -> ref Node;
        }
        mod cffi Main {
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("extern method `Nodes.head` return type"));
    assert!(error.contains("non-optional `ref Buffer` bridge values"));
    assert!(error.contains("hash-bound owned `ref String` returns"));
}

#[test]
fn accepts_ref_buffer_parameter_in_extern_function_signature() {
    let module = parse_nuis_module(
        r#"
        extern "c" fn host_take_buffer(buffer: ref Buffer, len: i64) -> i64;
        mod cffi Main {
          fn main() -> i64 {
            let backing: ref Buffer = alloc_buffer(8, 0);
            return host_take_buffer(backing, 8);
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert_eq!(
        main.body,
        vec![
            NirStmt::Let {
                name: "backing".to_owned(),
                ty: Some(nuis_semantics::model::NirTypeRef {
                    name: "Buffer".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: true,
                }),
                value: NirExpr::AllocBuffer {
                    len: Box::new(NirExpr::Int(8)),
                    fill: Box::new(NirExpr::Int(0)),
                },
            },
            NirStmt::Return(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_take_buffer".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(NirExpr::Var("backing".to_owned()))),
                    NirExpr::Int(8),
                ],
            }))
        ],
    );
}

#[test]
fn helper_pub_externs_can_cross_module_but_private_ones_cannot() {
    let main_ast = parse_nuis_ast(
        r#"
        use cffi Helper;
        mod cpu Main {
          fn main() -> i64 {
            return host_clock() + hidden_clock();
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        pub extern "c" fn host_clock() -> i64;
        extern "c" fn hidden_clock() -> i64;
        mod cffi Helper {
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(error.contains("unknown function `hidden_clock`"));
}

#[test]
fn lowers_host_symbol_bridge_stub_calls_into_cpu_extern_calls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          @host_symbol("network.open_tcp")
          fn open_tcp(local_port: i64, remote_port: i64) -> i64 {
            return 0;
          }

          fn main() -> i64 {
            return open_tcp(80, 8080);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .all(|function| function.name != "open_tcp"));
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert_eq!(
        main.body,
        vec![NirStmt::Return(Some(NirExpr::CpuExternCall {
            abi: "c".to_owned(),
            interface: None,
            callee: "host_network_open_tcp_stream".to_owned(),
            args: vec![NirExpr::Int(80), NirExpr::Int(8080)],
        }))]
    );
}
