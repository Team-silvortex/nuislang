use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use nuisc::registry::{HostFfiRegistryView, HostFfiSymbolRegistration};

fn compiled_source(path: &str) {
    nuisc::pipeline::compile_source_path(Path::new(path))
        .unwrap_or_else(|error| panic!("ffi source `{path}` should compile: {error}"));
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceExternSignature {
    abi: String,
    symbol: String,
    signature: String,
    path: PathBuf,
}

fn collect_ns_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("should read `{}`: {error}", root.display()));
    for entry in entries {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_ns_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("ns") {
            out.push(path);
        }
    }
}

fn collect_source_extern_signatures(root: &Path) -> Vec<SourceExternSignature> {
    let mut files = Vec::new();
    collect_ns_files(root, &mut files);

    let mut out = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("should read `{}`: {error}", path.display()));
        out.extend(parse_source_extern_signatures(&source).into_iter().map(
            |(abi, symbol, signature)| SourceExternSignature {
                abi,
                symbol,
                signature,
                path: path.clone(),
            },
        ));
    }
    out
}

fn parse_source_extern_signatures(source: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut rest = source;
    let needle = "extern \"";
    while let Some(index) = rest.find(needle) {
        let after_needle = &rest[index + needle.len()..];
        let Some(abi_end) = after_needle.find('"') else {
            break;
        };
        let abi = after_needle[..abi_end].trim();
        let after_abi = after_needle[abi_end + 1..].trim_start();
        let Some(after_fn) = after_abi.strip_prefix("fn ") else {
            rest = after_abi;
            continue;
        };
        let Some(open_index) = after_fn.find('(') else {
            break;
        };
        let symbol = after_fn[..open_index].trim();
        let after_open = &after_fn[open_index + 1..];
        let Some(close_index) = after_open.find(')') else {
            break;
        };
        let args = &after_open[..close_index];
        let after_close = &after_open[close_index + 1..];
        let Some(return_index) = after_close.find("->") else {
            rest = after_close;
            continue;
        };
        let after_return = &after_close[return_index + 2..];
        let return_type = after_return
            .split(|ch| ch == ';' || ch == '{' || ch == '\n')
            .next()
            .unwrap_or_default()
            .trim();
        if !symbol.is_empty() && !return_type.is_empty() {
            out.push((
                abi.to_owned(),
                symbol.to_owned(),
                render_source_ffi_signature(return_type, args),
            ));
        }
        rest = after_return;
    }
    out
}

fn render_source_ffi_signature(return_type: &str, args: &str) -> String {
    let arg_types = args
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty())
        .map(|arg| arg.rsplit_once(':').map(|(_, ty)| ty).unwrap_or(arg))
        .map(|ty| ty.trim().replace(' ', "_"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}({})", return_type.trim().replace(' ', "_"), arg_types)
}

fn source_extern_is_registered(
    view: &HostFfiRegistryView,
    abi: &str,
    symbol: &str,
    signature: &str,
) -> bool {
    let hash = yir_core::ffi::ffi_symbol_signature_hash(abi, symbol, signature);
    view.symbol_registrations(abi, symbol)
        .iter()
        .any(|registration| match registration {
            HostFfiSymbolRegistration::Signature(registered) => {
                registered.replace('+', ",") == signature
            }
            HostFfiSymbolRegistration::Hash(registered) => registered == &hash,
        })
}

#[test]
fn source_host_ffi_facades_are_exactly_registered_by_cpu_nustar() {
    let workspace = Path::new("../..");
    let manifest =
        nuisc::registry::load_manifest(&workspace.join("nustar-packages"), "official.cpu")
            .expect("official cpu manifest should load");
    let view = HostFfiRegistryView::from_manifest(&manifest);

    let mut declarations = collect_source_extern_signatures(&workspace.join("stdlib"));
    declarations.extend(collect_source_extern_signatures(
        &workspace.join("examples"),
    ));

    let mut unique = BTreeSet::new();
    let mut missing = BTreeMap::new();
    for declaration in declarations {
        if !unique.insert((
            declaration.abi.clone(),
            declaration.symbol.clone(),
            declaration.signature.clone(),
        )) {
            continue;
        }
        if !source_extern_is_registered(
            &view,
            &declaration.abi,
            &declaration.symbol,
            &declaration.signature,
        ) {
            missing.insert(
                format!(
                    "{} {} {}",
                    declaration.abi, declaration.symbol, declaration.signature
                ),
                declaration.path,
            );
        }
    }

    assert!(
        missing.is_empty(),
        "source C FFI declarations must be exact-registered by official.cpu; missing: {missing:#?}"
    );
}

#[test]
fn accepts_registered_libc_demo_signatures() {
    compiled_source("../../examples/ns/ffi/hello_clock_test_facades.ns");
    compiled_source("../../examples/ns/ffi/libc_usleep_demo.ns");
    compiled_source("../../examples/ns/ffi/libc_puts_demo.ns");
    compiled_source("../../examples/ns/ffi/libc_strlen_demo.ns");
    compiled_source("../../examples/ns/ffi/libc_write_demo.ns");
    compiled_source("../../examples/ns/ffi/libc_close_demo.ns");
    compiled_source("../../examples/ns/ffi/libc_read_buffer_demo.ns");
}

#[test]
fn compiles_host_bridge_ffi_frontdoor_sources() {
    compiled_source("../../examples/ns/ffi/hello_ffi.ns");
    compiled_source("../../examples/ns/ffi/hello_c_ffi.ns");
    compiled_source("../../examples/ns/ffi/hello_c_i32_ffi.ns");
}

#[test]
fn lowers_i32_ffi_frontdoor_source_to_i32_extern_call() {
    let artifacts =
        nuisc::pipeline::compile_source_path(Path::new("../../examples/ns/ffi/hello_c_i32_ffi.ns"))
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
fn rejects_registered_stdout_facade_symbol_with_wide_family_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_stdout_facade_symbol_allowlist_{}_{}.ns",
        std::process::id(),
        "host_stdout_write"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "c" fn host_stdout_write(lhs: i64, rhs: i64) -> i64;

          fn main() -> i64 {
            return host_stdout_write(1, 2);
          }
        }
        "#,
    )
    .expect("should write temporary stdout facade ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("registered stdout facade symbol should not fall back to i64(*)");
    let _ = fs::remove_file(&path);

    assert!(error.contains("symbol `host_stdout_write` signature `i64(i64,i64)`"));
    assert!(error.contains("allowed symbol registrations: signature:i64(i64)"));
}

#[test]
fn rejects_registered_libc_symbol_with_wide_family_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_libc_symbol_allowlist_{}_{}.ns",
        std::process::id(),
        "usleep"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "libc" fn usleep(usec: i64) -> i32;

          fn main() -> i32 {
            return usleep(1);
          }
        }
        "#,
    )
    .expect("should write temporary libc ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("registered libc symbol should not fall back to a wide signature");
    let _ = fs::remove_file(&path);

    assert!(error.contains("symbol `usleep` signature `i32(i64)`"));
    assert!(error.contains("allowed symbol registrations: signature:i32(i32)"));
}

#[test]
fn rejects_unregistered_libc_symbol_even_with_known_text_signature() {
    let path = std::env::temp_dir().join(format!(
        "nuis_bad_libc_symbol_allowlist_{}_{}.ns",
        std::process::id(),
        "strlen_like"
    ));
    fs::write(
        &path,
        r#"
        mod cpu Main {
          extern "libc" fn strlen_like(message: String) -> i64;

          fn main() -> i64 {
            return strlen_like("nuis");
          }
        }
        "#,
    )
    .expect("should write temporary libc ffi source");

    let error = nuisc::pipeline::compile_source_path(&path)
        .err()
        .expect("unregistered libc symbol should not use a wide family allowlist");
    let _ = fs::remove_file(&path);

    assert!(error.contains("ABI `libc`"));
    assert!(error.contains("has no `ffi:` signature allowlist entries"));
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
    compiled_source("../../examples/ns/ffi/hello_task_runtime_facades.ns");
    compiled_source("../../examples/ns/ffi/hello_task_cli_facades.ns");
    compiled_source("../../examples/ns/ffi/hello_input_runtime_facades.ns");
}

#[test]
fn compiles_path_runtime_ffi_frontdoor_sources() {
    compiled_source("../../examples/ns/ffi/hello_path_runtime_facades.ns");
    compiled_source("../../examples/ns/ffi/hello_file_runtime_facades.ns");
    compiled_source("../../examples/ns/ffi/hello_directory_runtime_facades.ns");
}

#[test]
fn compiles_representative_ffi_companion_sources() {
    for path in [
        "../../examples/ns/ffi/hello_argv_runtime_facades.ns",
        "../../examples/ns/ffi/hello_benchmark_report_facades.ns",
        "../../examples/ns/ffi/hello_benchmark_report_count_facades.ns",
        "../../examples/ns/ffi/hello_benchmark_report_file_facades.ns",
        "../../examples/ns/ffi/hello_env_runtime_facades.ns",
        "../../examples/ns/ffi/hello_process_runtime_facades.ns",
        "../../examples/ns/ffi/hello_command_runtime_facades.ns",
        "../../examples/ns/ffi/hello_subprocess_runtime_facades.ns",
        "../../examples/ns/ffi/hello_host_text_runtime_facades.ns",
        "../../examples/ns/ffi/hello_io_runtime_facades.ns",
        "../../examples/ns/ffi/hello_terminal_io_facades.ns",
        "../../examples/ns/ffi/hello_cwd_runtime_facades.ns",
        "../../examples/ns/ffi/hello_temp_runtime_facades.ns",
        "../../examples/ns/ffi/hello_location_runtime_facades.ns",
        "../../examples/ns/ffi/hello_cache_runtime_facades.ns",
        "../../examples/ns/ffi/hello_config_cache_facades.ns",
        "../../examples/ns/ffi/hello_fs_metadata_runtime_facades.ns",
        "../../examples/ns/ffi/hello_stat_runtime_facades.ns",
        "../../examples/ns/ffi/hello_path_parent_facades.ns",
        "../../examples/ns/ffi/hello_path_depth_facades.ns",
    ] {
        compiled_source(path);
    }
}
