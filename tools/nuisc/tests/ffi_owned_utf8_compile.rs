use std::path::Path;

#[test]
fn lowers_registered_owned_utf8_with_validation_and_exact_cleanup() {
    let artifacts = nuisc::pipeline::compile_source_path(Path::new(
        "../../examples/ns/ffi/owned_return_utf8_demo.ns",
    ))
    .expect("owned UTF-8 example should compile");
    let producer = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "extern_call_owned_utf8")
        .expect("owned UTF-8 return should use its dedicated YIR operation");
    let contract = yir_core::ffi::parse_owned_utf8_return_contract(&producer.op.args)
        .expect("owned UTF-8 contract should revalidate");

    assert_eq!(contract.symbol, "host_owned_utf8_make");
    assert_eq!(contract.signature, "ref_String(i64)");
    assert_eq!(contract.destructor_symbol, "host_owned_utf8_destroy");
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @nuis_host_owned_utf8_validate_v1(ptr"));
    assert!(artifacts.llvm_ir.contains("getelementptr inbounds i8, ptr"));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_owned_utf8_destroy(ptr"));
    assert!(!artifacts.llvm_ir.contains("deferred lowering"));
}

#[test]
fn rejects_owned_utf8_without_exact_once_release() {
    let ast = nuisc::frontend::parse_nuis_ast(
        r#"
        mod cffi Main {
          extern "c" fn host_owned_utf8_make(seed: i64) -> ref String;
          fn main() -> i64 {
            let text: ref String = host_owned_utf8_make(13);
            return owned_utf8_len(text);
          }
        }
        "#,
    )
    .unwrap();
    let error = match nuisc::pipeline::compile_ast(ast) {
        Ok(_) => panic!("owned UTF-8 without release must remain closed"),
        Err(error) => error,
    };

    assert!(
        error.contains("exactly one direct free") || error.contains("owned address"),
        "{error}"
    );
}

#[test]
fn rejects_owned_utf8_write_surface() {
    let ast = nuisc::frontend::parse_nuis_ast(
        r#"
        mod cffi Main {
          extern "c" fn host_owned_utf8_make(seed: i64) -> ref String;
          fn main() -> i64 {
            let text: ref String = host_owned_utf8_make(13);
            store_at(text, 0, 65);
            free(text);
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let error = match nuisc::pipeline::compile_ast(ast) {
        Ok(_) => panic!("owned UTF-8 must not enter mutable buffer operations"),
        Err(error) => error,
    };

    assert!(
        error.contains("ref String")
            || error.contains("load/store target")
            || error.contains("escapes through unsupported `cpu.store_at`"),
        "{error}"
    );
}
