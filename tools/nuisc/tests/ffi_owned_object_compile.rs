use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_output_dir() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("nuis_owned_object_{nonce}"))
}

#[test]
fn lowers_registered_owned_object_with_static_reads_and_exact_cleanup() {
    let artifacts = nuisc::pipeline::compile_source_path(Path::new(
        "../../examples/ns/ffi/owned_return_object_demo.ns",
    ))
    .expect("owned object example should compile");
    let producer = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "extern_call_owned_object")
        .expect("owned object return should use its dedicated YIR operation");
    let contract = yir_core::ffi::parse_owned_object_return_contract(&producer.op.args)
        .expect("owned object contract should revalidate");

    assert_eq!(contract.symbol, "host_owned_object_make");
    assert_eq!(contract.signature, "ref_FfiObject(i64)");
    assert_eq!(contract.size_policy, "static:16");
    assert_eq!(contract.read_policy, "i64_slots");
    assert_eq!(contract.destructor_symbol, "host_owned_object_destroy");
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @nuis_host_owned_object_validate_v1(ptr"));
    assert_eq!(
        artifacts
            .llvm_ir
            .matches("call i64 @host_owned_object_destroy(ptr")
            .count(),
        1
    );
    assert!(artifacts.llvm_ir.contains("owned_object_index_invalid"));
    assert!(!artifacts.llvm_ir.contains("deferred lowering"));
}

#[test]
fn rejects_owned_object_without_exact_once_release() {
    let ast = nuisc::frontend::parse_nuis_ast(
        r#"
        mod cffi Main {
          extern "c" fn host_owned_object_make(seed: i64) -> ref FfiObject;
          fn main() -> i64 {
            let object: ref FfiObject = host_owned_object_make(7);
            return owned_object_read_i64(object, 0);
          }
        }
        "#,
    )
    .unwrap();
    let error = match nuisc::pipeline::compile_ast(ast) {
        Ok(_) => panic!("owned object without release must remain closed"),
        Err(error) => error,
    };

    assert!(
        error.contains("exactly one direct free") || error.contains("owned address"),
        "{error}"
    );
}

#[test]
fn rejects_owned_object_raw_buffer_fallback() {
    let ast = nuisc::frontend::parse_nuis_ast(
        r#"
        mod cffi Main {
          extern "c" fn host_owned_object_make(seed: i64) -> ref FfiObject;
          fn main() -> i64 {
            let object: ref FfiObject = host_owned_object_make(7);
            let value: i64 = load_at(object, 0);
            free(object);
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let error = match nuisc::pipeline::compile_ast(ast) {
        Ok(_) => panic!("registered objects must not enter generic buffer reads"),
        Err(error) => error,
    };

    assert!(
        error.contains("ref FfiObject")
            || error.contains("load/store target")
            || error.contains("unsupported `cpu.load_at`"),
        "{error}"
    );
}

#[test]
fn aot_owned_object_returns_to_zero_live_allocations() {
    let output_dir = temp_output_dir();
    let compile = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args([
            "compile",
            "../../examples/projects/ffi/owned_return_object_demo",
            output_dir
                .to_str()
                .expect("temporary output path should be UTF-8"),
        ])
        .output()
        .expect("nuisc should launch");
    assert!(
        compile.status.success(),
        "owned object AOT compile failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(output_dir.join("owned_return_object_demo"))
        .output()
        .expect("owned object AOT binary should launch");
    assert_eq!(
        run.status.code(),
        Some(0),
        "owned object values or exact cleanup count drifted:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    fs::remove_dir_all(output_dir).expect("temporary AOT output should be removable");
}
