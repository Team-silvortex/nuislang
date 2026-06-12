use super::lower_project_ast_to_nir;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{
    AstVisibility, NirExpr, NirStmt, NirVisibility, TestClockDomain, TestClockPolicy,
};

#[test]
fn parses_test_function_with_explicit_label_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          test("smoke_add") fn add_test() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "add_test")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
}

#[test]
fn parses_trait_impl_and_generic_function_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs;
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(ast.traits.len(), 1);
    assert_eq!(ast.traits[0].name, "Addable");
    assert_eq!(ast.traits[0].visibility, AstVisibility::Public);
    assert_eq!(ast.traits[0].methods.len(), 1);
    assert_eq!(ast.impls.len(), 1);
    assert_eq!(ast.impls[0].trait_name, "Addable");
    assert_eq!(ast.impls[0].for_type.name, "i64");

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "sum_two")
        .unwrap();
    assert_eq!(function.generic_params.len(), 1);
    assert_eq!(function.generic_params[0].name, "T");
    assert_eq!(
        function.generic_params[0]
            .bound
            .as_ref()
            .map(|bound| bound.name.as_str()),
        Some("Addable")
    );
}

#[test]
fn rejects_pub_trait_methods_in_current_frontend() {
    let error = parse_nuis_ast(
        r#"
        mod cpu Main {
          pub trait Addable {
            pub fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("trait methods do not support independent `pub` visibility"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_pub_impl_methods_in_current_frontend() {
    let error = parse_nuis_ast(
        r#"
        mod cpu Main {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            pub fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("impl methods do not support independent `pub` visibility"),
        "unexpected error: {error}"
    );
}

#[test]
fn parses_function_annotations_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          @inline
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert_eq!(function.attributes.len(), 1);
    assert_eq!(function.attributes[0].name, "inline");
}

#[test]
fn parses_struct_and_field_annotations_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            id: i64,
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(ast.structs.len(), 1);
    assert_eq!(ast.structs[0].attributes.len(), 1);
    assert_eq!(ast.structs[0].attributes[0].name, "packet");
    assert_eq!(ast.structs[0].fields.len(), 1);
    assert_eq!(ast.structs[0].fields[0].attributes.len(), 1);
    assert_eq!(ast.structs[0].fields[0].attributes[0].name, "packet_field");
}

#[test]
fn parses_test_function_without_explicit_label_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          test() fn smoke_add() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "smoke_add")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
    assert!(!function.test_ignored);
    assert!(!function.test_should_fail);
}

#[test]
fn parses_test_function_modifiers_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          test("smoke_add", ignored=true, should_fail=true) fn smoke_add() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "smoke_add")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
    assert!(function.test_ignored);
    assert!(function.test_should_fail);
}

#[test]
fn parses_test_function_call_syntax_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          test("smoke_add", ignored=true, should_fail=true, reason="must reject zero", timeout_ms=25, clock_domain="wall") fn smoke_add() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "smoke_add")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
    assert!(function.test_ignored);
    assert!(function.test_should_fail);
    assert_eq!(function.test_reason.as_deref(), Some("must reject zero"));
    assert_eq!(function.test_timeout_ms, Some(25));
    assert_eq!(function.test_clock_domain, Some(TestClockDomain::Wall));
    assert_eq!(function.test_clock_policy, None);
    assert!(function
        .attributes
        .iter()
        .any(|attribute| attribute.name == "test"));
}

#[test]
fn parses_at_test_function_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          @test("slow_global", timeout_ms=25, clock_domain="global", clock_policy="bridge")
          async fn slow_global() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "slow_global")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("slow_global"));
    assert_eq!(function.test_timeout_ms, Some(25));
    assert_eq!(function.test_clock_domain, Some(TestClockDomain::Global));
    assert_eq!(function.test_clock_policy, Some(TestClockPolicy::Bridge));
    assert!(function
        .attributes
        .iter()
        .any(|attribute| attribute.name == "test"));
}

#[test]
fn lowers_test_function_label_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          test("smoke_add") fn smoke_add() -> i64 {
            return 1;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "smoke_add")
        .unwrap();
    assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
    assert!(!function.test_ignored);
    assert!(!function.test_should_fail);
}

#[test]
fn lowers_trait_impl_and_generic_function_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs;
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.traits.len(), 1);
    assert_eq!(module.traits[0].name, "Addable");
    assert_eq!(module.traits[0].visibility, NirVisibility::Public);
    assert_eq!(module.traits[0].methods.len(), 1);
    assert_eq!(module.impls.len(), 1);
    assert_eq!(module.impls[0].trait_name, "Addable");
    assert_eq!(module.impls[0].methods.len(), 1);
    assert!(module
        .functions
        .iter()
        .all(|function| function.name != "sum_two"));
}

#[test]
fn lowers_function_annotations_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          @inline
          @export(name = "main")
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert_eq!(function.annotations.len(), 2);
    assert_eq!(function.annotations[0].name, "inline");
    assert_eq!(function.annotations[1].name, "export");
}

#[test]
fn lowers_struct_and_field_annotations_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            id: i64,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.structs.len(), 1);
    assert_eq!(module.structs[0].annotations.len(), 1);
    assert_eq!(module.structs[0].annotations[0].name, "packet");
    assert_eq!(module.structs[0].fields[0].annotations.len(), 1);
    assert_eq!(
        module.structs[0].fields[0].annotations[0].name,
        "packet_field"
    );
}

#[test]
fn parses_extern_host_symbol_bridge_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          extern "c" @host_symbol("network.open_tcp") fn open_tcp(local_port: i64, remote_port: i64) -> i64;
        }
        "#,
    )
    .unwrap();

    assert_eq!(ast.externs.len(), 1);
    assert_eq!(ast.externs[0].name, "open_tcp");
    assert_eq!(
        ast.externs[0].host_symbol.as_deref(),
        Some("network.open_tcp")
    );
}

#[test]
fn parses_pub_extern_items_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        pub extern "c" fn host_clock() -> i64;
        pub extern "c" interface Clock {
          fn now() -> i64;
        }
        mod cpu Main {
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(ast.externs.len(), 1);
    assert_eq!(ast.externs[0].visibility, AstVisibility::Public);
    assert_eq!(ast.extern_interfaces.len(), 1);
    assert_eq!(ast.extern_interfaces[0].visibility, AstVisibility::Public);
    assert_eq!(
        ast.extern_interfaces[0].methods[0].visibility,
        AstVisibility::Private
    );
}

#[test]
fn lowers_extern_host_symbol_bridge_into_nir_signature() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" @host_symbol("network.open_tcp") fn open_tcp(local_port: i64, remote_port: i64) -> i64;

          fn main() -> i64 {
            return open_tcp(80, 8080);
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.externs.len(), 1);
    assert_eq!(module.externs[0].name, "open_tcp");
    assert_eq!(
        module.externs[0].host_symbol.as_deref(),
        Some("network.open_tcp")
    );
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

#[test]
fn lowers_pub_extern_items_into_nir() {
    let module = parse_nuis_module(
        r#"
        pub extern "c" fn host_clock() -> i64;
        pub extern "c" interface Clock {
          fn now() -> i64;
        }
        mod cpu Main {
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.externs.len(), 1);
    assert_eq!(module.externs[0].visibility, NirVisibility::Public);
    assert_eq!(module.extern_interfaces.len(), 1);
    assert_eq!(
        module.extern_interfaces[0].visibility,
        NirVisibility::Public
    );
    assert_eq!(
        module.extern_interfaces[0].methods[0].visibility,
        NirVisibility::Private
    );
}

#[test]
fn rejects_ref_parameter_in_extern_function_signature() {
    let error = parse_nuis_module(
        r#"
        extern "c" fn host_take_ptr(head: ref Node) -> i64;
        mod cpu Main {
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("extern function `host_take_ptr` parameter `head`"));
    assert!(error.contains("host-boundary pointer parameters and returns are not stabilized yet"));
}

#[test]
fn rejects_ref_return_in_extern_interface_signature() {
    let error = parse_nuis_module(
        r#"
        extern "c" interface Nodes {
          fn head() -> ref Node;
        }
        mod cpu Main {
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("extern method `Nodes.head` return type"));
    assert!(error.contains("host-boundary pointer parameters and returns are not stabilized yet"));
}

#[test]
fn helper_pub_externs_can_cross_module_but_private_ones_cannot() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;
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
        mod cpu Helper {
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
