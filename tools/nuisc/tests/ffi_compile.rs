use std::{fs, path::Path};

fn compiled_source(path: &str) {
    nuisc::pipeline::compile_source_path(Path::new(path))
        .unwrap_or_else(|error| panic!("ffi source `{path}` should compile: {error}"));
}

#[test]
fn compiles_host_bridge_ffi_frontdoor_sources() {
    compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns");
    compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_ffi.ns");
    compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_i32_ffi.ns");
}

#[test]
fn lowers_i32_ffi_frontdoor_source_to_i32_extern_call() {
    let artifacts = nuisc::pipeline::compile_source_path(Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_i32_ffi.ns",
    ))
    .unwrap_or_else(|error| panic!("i32 ffi source should compile: {error}"));

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "extern_call_i32"));
    assert!(artifacts
        .llvm_ir
        .contains("declare i32 @host_i32_curve(i32)"));
    assert!(artifacts.llvm_ir.contains("call i32 @host_i32_curve(i32"));
}

#[test]
fn rejects_c_ffi_signatures_outside_nustar_allowlist() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_ffi_allowlist_{}_{}.ns",
        std::process::id(),
        "f64"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "c" fn host_f64_curve(value: f64) -> f64;

          fn main() -> f64 {
            return host_f64_curve(1.0);
          }
        }
        "#,
    )
    .expect("should write temporary ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("f64 ffi source should be rejected by C ABI signature allowlist");
    let _ = fs::remove_file(&path);

    assert!(error.contains("signature `f64(f64)` is not allowed"));
    assert!(error.contains("allowed signatures"));
}

#[test]
fn rejects_registered_c_ffi_symbol_with_wrong_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_ffi_symbol_allowlist_{}_{}.ns",
        std::process::id(),
        "host_i32_curve"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "c" fn host_i32_curve(value: f64) -> f64;

          fn main() -> f64 {
            return host_i32_curve(1.0);
          }
        }
        "#,
    )
    .expect("should write temporary ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("registered C FFI symbol with wrong signature should be rejected");
    let _ = fs::remove_file(&path);

    assert!(error.contains("symbol `host_i32_curve` signature `f64(f64)`"));
    assert!(error.contains("allowed symbol registrations: signature:i32(i32)"));
}

#[test]
fn rejects_registered_host_runtime_symbol_with_wide_family_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_host_runtime_symbol_allowlist_{}_{}.ns",
        std::process::id(),
        "host_argv_count"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count(seed: i64) -> i64;

          fn main() -> i64 {
            return host_argv_count(1);
          }
        }
        "#,
    )
    .expect("should write temporary host runtime ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("registered host runtime symbol should not fall back to i64(*)");
    let _ = fs::remove_file(&path);

    assert!(error.contains("symbol `host_argv_count` signature `i64(i64)`"));
    assert!(error.contains("allowed symbol registrations: signature:i64()"));
}

#[test]
fn rejects_registered_network_probe_symbol_with_wide_family_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_network_probe_symbol_allowlist_{}_{}.ns",
        std::process::id(),
        "host_network_connect_probe"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "c" fn host_network_connect_probe(port: i64) -> i64;

          fn main() -> i64 {
            return host_network_connect_probe(8080);
          }
        }
        "#,
    )
    .expect("should write temporary network probe ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("registered network probe symbol should not fall back to i64(*)");
    let _ = fs::remove_file(&path);

    assert!(error.contains("symbol `host_network_connect_probe` signature `i64(i64)`"));
    assert!(error.contains("allowed symbol registrations: signature:i64(i64,i64,i64)"));
}

#[test]
fn accepts_hash_registered_c_ffi_symbol_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_hash_ffi_allowlist_{}_{}.ns",
        std::process::id(),
        "host_hashed_curve"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "c" fn host_hashed_curve(value: i64) -> i64;

          fn main() -> i64 {
            return host_hashed_curve(7);
          }
        }
        "#,
    )
    .expect("should write temporary hash ffi source");

    nuisc::pipeline::compile_source_path(&path)
        .unwrap_or_else(|error| panic!("hash-registered ffi source should compile: {error}"));
    let _ = fs::remove_file(&path);
}

#[test]
fn rejects_hash_registered_c_ffi_symbol_with_wrong_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_hash_ffi_allowlist_{}_{}.ns",
        std::process::id(),
        "host_hashed_curve"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "c" fn host_hashed_curve(value: f64) -> f64;

          fn main() -> f64 {
            return host_hashed_curve(1.0);
          }
        }
        "#,
    )
    .expect("should write temporary bad hash ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("hash-registered C FFI symbol with wrong signature should be rejected");
    let _ = fs::remove_file(&path);

    assert!(error.contains("symbol `host_hashed_curve` signature `f64(f64)`"));
    assert!(error.contains("hash `fnv1a64:a1c664e04682ecad`"));
    assert!(error.contains("hash:fnv1a64:38ca92f356fcb551"));
}

#[test]
fn compiles_task_runtime_ffi_frontdoor_sources() {
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_runtime_facades.ns",
    );
    compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns");
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns",
    );
}

#[test]
fn compiles_path_runtime_ffi_frontdoor_sources() {
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns",
    );
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_runtime_facades.ns",
    );
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_runtime_facades.ns",
    );
}

#[test]
fn compiles_representative_ffi_companion_sources() {
    for path in [
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_argv_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_benchmark_report_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_benchmark_report_count_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_benchmark_report_file_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_process_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_command_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_subprocess_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_host_text_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_io_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cwd_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_temp_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_fs_metadata_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stat_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_depth_facades.ns",
    ] {
        compiled_source(path);
    }
}
