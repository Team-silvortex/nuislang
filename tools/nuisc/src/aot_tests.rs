use super::{
    build_nuis_lifecycle_contract, c_shim_source,
    compile_artifacts_for_output_dir_with_packaging_mode, decode_domain_build_unit_payload_blob,
    decode_nuis_compiled_artifact_binary, decode_nuis_executable_envelope_binary,
    encode_nuis_compiled_artifact_binary, encode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_executable_envelope_binary, inspect_nuis_compiled_artifact_container,
    parse_nuis_compiled_artifact, parse_nuis_executable_envelope,
    project_metadata_summary_mismatch_error, render_domain_build_unit_backend_stub,
    render_domain_build_unit_bridge_plan, render_domain_build_unit_host_bridge_stub,
    render_domain_build_unit_kernel_ir_sidecar, render_domain_build_unit_lowering_plan,
    render_domain_build_unit_network_ir_sidecar, render_domain_build_unit_shader_ir_sidecar,
    render_nuis_executable_envelope, render_relocated_unpacked_build_manifest,
    resolve_cpu_build_target, resolve_cpu_build_target_from_abi,
    resolve_cpu_build_target_from_target, verify_build_manifest, verify_nuis_compiled_artifact,
    write_build_manifest, write_nuis_compiled_artifact, write_nuis_executable_envelope,
    BuildManifestCacheInfo, BuildManifestContext, BuildManifestDomainBuildUnit,
    BuildManifestProjectInfo, CompileArtifacts, CpuBuildTarget, NuisExecutableEnvelope,
};
use nuis_artifact::{
    decode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_compiled_artifact_section_table,
    protocol::COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML,
};
use nuis_semantics::model::{AstExternFunction, AstModule, AstParam, AstTypeRef, AstVisibility};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn registry_root() -> PathBuf {
    let root = temp_dir("nustar_registry");
    fs::write(
            root.join("index.toml"),
            "[[package]]\npackage_id = \"official.cpu\"\nmanifest = \"cpu.toml\"\ndomain_family = \"cpu\"\n",
        )
        .unwrap();
    fs::write(
            root.join("cpu.toml"),
            "manifest_schema = \"nustar-manifest-v1\"\npackage_id = \"official.cpu\"\ndomain_family = \"cpu\"\nfrontend = \"nustar-cpu\"\nentry_crate = \"crates/yir-domain-cpu\"\nast_entry = \"cpu.ast.bootstrap.v1\"\nnir_entry = \"cpu.nir.bootstrap.v1\"\nyir_lowering_entry = \"cpu.yir.lowering.v1\"\npart_verify_entry = \"cpu.verify.partial.v1\"\nast_surface = [\"cpu.mod-ast.v1\"]\nnir_surface = [\"nir.cpu.surface.v1\"]\nyir_lowering = [\"yir.cpu.lowering.v1\"]\npart_verify = [\"verify.cpu.contract.v1\"]\nbinary_extension = \"nustar\"\npackage_layout = \"single-envelope\"\nmachine_abi_policy = \"exact-match\"\nabi_profiles = [\"cpu.arm64.apple_aapcs64\", \"cpu.x86_64.apple_sysv64\", \"cpu.x86_64.sysv64\", \"cpu.x86_64.win64\"]\nabi_capabilities = [\"cpu.arm64.apple_aapcs64:op:cpu.*\", \"cpu.x86_64.apple_sysv64:op:cpu.*\", \"cpu.x86_64.sysv64:op:cpu.*\", \"cpu.x86_64.win64:op:cpu.*\"]\nabi_targets = [\"cpu.arm64.apple_aapcs64:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin\", \"cpu.x86_64.apple_sysv64:arch=x86_64|os=darwin|object=mach-o|calling=sysv64|clang=x86_64-apple-darwin\", \"cpu.x86_64.sysv64:arch=x86_64|os=linux|object=elf|calling=sysv64|clang=x86_64-unknown-linux-gnu\", \"cpu.x86_64.win64:arch=x86_64|os=windows|object=coff|calling=win64|clang=x86_64-pc-windows-msvc\"]\nimplementation_kinds = [\"native-stub\"]\nloader_entry = \"nustar.bootstrap.v1\"\nloader_abi = \"nustar-loader-v1\"\nhost_ffi_surface = []\nhost_ffi_abis = []\nhost_ffi_bridge = \"none\"\nsupport_surface = []\nsupport_profile_slots = []\ndefault_lanes = []\nprofiles = [\"aot\"]\nresource_families = [\"cpu\", \"cpu.arm64\", \"cpu.x86_64\"]\nunit_types = [\"Main\"]\nlowering_targets = [\"llvm\", \"x86_64\"]\nops = [\"cpu.const\"]\n",
        )
        .unwrap();
    root
}

fn write_minimal_cpu_artifact(label: &str) -> (PathBuf, PathBuf) {
    let dir = temp_dir(label);
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(&bin, "bin").unwrap();

    let written = CompileArtifacts {
        ast_path: ast.display().to_string(),
        nir_path: nir.display().to_string(),
        yir_path: yir.display().to_string(),
        llvm_ir_path: ll.display().to_string(),
        binary_path: bin.display().to_string(),
        packaging_mode: "native-cpu-llvm".to_owned(),
    };
    let cpu_target = CpuBuildTarget {
        abi: "cpu.x86_64.sysv64".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        clang_target: "x86_64-unknown-linux-gnu".to_owned(),
        isa_family: "x86_64".to_owned(),
        isa_features: vec!["x86-64".to_owned(), "sse2".to_owned()],
        cross_compile: true,
    };
    let manifest = super::write_build_manifest(
        &dir,
        &written,
        &BuildManifestContext {
            input_path: "/tmp/demo.ns".to_owned(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache: None,
            project: None,
            doc_index: None,
            cpu_target,
        },
    )
    .unwrap();
    (dir, PathBuf::from(manifest))
}

#[test]
fn cached_compile_artifacts_accept_self_contained_nsb_packaging_mode() {
    let dir = temp_dir("cached_self_contained_packaging_mode");
    let input = dir.join("main.ns");
    fs::write(&input, "mod cpu Main { fn main() -> i64 { return 0; } }").unwrap();

    let written = compile_artifacts_for_output_dir_with_packaging_mode(
        &input,
        &dir,
        "nuis-self-contained-image",
    )
    .unwrap();

    assert_eq!(written.packaging_mode, "nuis-self-contained-image");
    assert_eq!(written.binary_path, dir.join("main").display().to_string());
}

fn i64_ty() -> AstTypeRef {
    AstTypeRef {
        name: "i64".to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: false,
    }
}

fn host_i64_param(name: &str) -> AstParam {
    AstParam {
        name: name.to_owned(),
        ty: i64_ty(),
    }
}

fn host_i64_extern(name: &str, params: &[&str]) -> AstExternFunction {
    AstExternFunction {
        visibility: AstVisibility::Private,
        abi: "c".to_owned(),
        interface: None,
        name: name.to_owned(),
        params: params.iter().map(|param| host_i64_param(param)).collect(),
        return_type: i64_ty(),
        host_symbol: None,
    }
}

fn host_runtime_hooks_ast() -> AstModule {
    let extern_specs: &[(&str, &[&str])] = &[
        ("host_argv_count", &[] as &[&str]),
        (
            "host_deserialize_text_ends_with",
            &["buffer_handle", "offset", "len", "suffix_handle"] as &[&str],
        ),
        ("host_cwd_handle", &[] as &[&str]),
        ("host_monotonic_time_ns", &[] as &[&str]),
        (
            "host_deserialize_bool_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_parse_header_line",
            &["buffer_handle", "offset", "len", "expected_name_handle"] as &[&str],
        ),
        (
            "host_find_header_value",
            &["buffer_handle", "offset", "len", "expected_name_handle"] as &[&str],
        ),
        (
            "host_find_status_line_reason",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_parse_http_response_summary",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_parse_http_request_summary",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_parse_http_roundtrip_summary",
            &[
                "request_buffer_handle",
                "request_offset",
                "request_len",
                "response_buffer_handle",
                "response_offset",
                "response_len",
            ] as &[&str],
        ),
        (
            "host_deserialize_text_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_deserialize_bool_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_deserialize_text_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_deserialize_bool_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_deserialize_text_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_deserialize_bool_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_deserialize_text_from",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_deserialize_text_equals",
            &["buffer_handle", "offset", "len", "expected_handle"] as &[&str],
        ),
        (
            "host_deserialize_text_starts_with",
            &["buffer_handle", "offset", "len", "prefix_handle"] as &[&str],
        ),
        (
            "host_deserialize_text_equals",
            &["buffer_handle", "offset", "len", "expected_handle"] as &[&str],
        ),
        (
            "host_deserialize_text_starts_with",
            &["buffer_handle", "offset", "len", "prefix_handle"] as &[&str],
        ),
        (
            "host_deserialize_text_equals",
            &["buffer_handle", "offset", "len", "expected_handle"] as &[&str],
        ),
        (
            "host_deserialize_text_starts_with",
            &["buffer_handle", "offset", "len", "prefix_handle"] as &[&str],
        ),
        (
            "host_deserialize_text_contains",
            &["buffer_handle", "offset", "len", "needle_handle"] as &[&str],
        ),
        (
            "host_buffer_find_byte",
            &["buffer_handle", "offset", "len", "needle"] as &[&str],
        ),
        (
            "host_fill_bytes",
            &["buffer_handle", "offset", "len", "value"] as &[&str],
        ),
        (
            "host_copy_bytes",
            &[
                "dst_handle",
                "dst_offset",
                "dst_len",
                "src_handle",
                "src_offset",
                "src_len",
            ] as &[&str],
        ),
        (
            "host_compare_bytes",
            &[
                "lhs_handle",
                "lhs_offset",
                "lhs_len",
                "rhs_handle",
                "rhs_offset",
                "rhs_len",
            ] as &[&str],
        ),
        (
            "host_buffer_find_text",
            &["buffer_handle", "offset", "len", "needle_handle"] as &[&str],
        ),
        (
            "host_buffer_find_text",
            &["buffer_handle", "offset", "len", "needle_handle"] as &[&str],
        ),
        (
            "host_buffer_find_text",
            &["buffer_handle", "offset", "len", "needle_handle"] as &[&str],
        ),
        (
            "host_buffer_find_line_end",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_buffer_trim_line_end",
            &["buffer_handle", "offset", "len"] as &[&str],
        ),
        (
            "host_fill_bytes",
            &["buffer_handle", "offset", "len", "value"] as &[&str],
        ),
        (
            "host_copy_bytes",
            &[
                "dst_handle",
                "dst_offset",
                "dst_len",
                "src_handle",
                "src_offset",
                "src_len",
            ] as &[&str],
        ),
        (
            "host_compare_bytes",
            &[
                "lhs_handle",
                "lhs_offset",
                "lhs_len",
                "rhs_handle",
                "rhs_offset",
                "rhs_len",
            ] as &[&str],
        ),
        (
            "host_fill_bytes",
            &["buffer_handle", "offset", "len", "value"] as &[&str],
        ),
        (
            "host_copy_bytes",
            &[
                "dst_handle",
                "dst_offset",
                "dst_len",
                "src_handle",
                "src_offset",
                "src_len",
            ] as &[&str],
        ),
        (
            "host_compare_bytes",
            &[
                "lhs_handle",
                "lhs_offset",
                "lhs_len",
                "rhs_handle",
                "rhs_offset",
                "rhs_len",
            ] as &[&str],
        ),
        (
            "host_fill_bytes",
            &["buffer_handle", "offset", "len", "value"] as &[&str],
        ),
        (
            "host_copy_bytes",
            &[
                "dst_handle",
                "dst_offset",
                "dst_len",
                "src_handle",
                "src_offset",
                "src_len",
            ] as &[&str],
        ),
        (
            "host_compare_bytes",
            &[
                "lhs_handle",
                "lhs_offset",
                "lhs_len",
                "rhs_handle",
                "rhs_offset",
                "rhs_len",
            ] as &[&str],
        ),
        (
            "host_fill_bytes",
            &["buffer_handle", "offset", "len", "value"] as &[&str],
        ),
        (
            "host_copy_bytes",
            &[
                "dst_handle",
                "dst_offset",
                "dst_len",
                "src_handle",
                "src_offset",
                "src_len",
            ] as &[&str],
        ),
        (
            "host_compare_bytes",
            &[
                "lhs_handle",
                "lhs_offset",
                "lhs_len",
                "rhs_handle",
                "rhs_offset",
                "rhs_len",
            ] as &[&str],
        ),
    ];
    AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: extern_specs
            .iter()
            .map(|(name, params)| host_i64_extern(name, params))
            .collect(),
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    }
}

#[path = "aot_tests/artifact_paths.rs"]
mod artifact_paths;
#[path = "aot_tests/artifact_verify.rs"]
mod artifact_verify;
#[path = "aot_tests/domain_profiles.rs"]
mod domain_profiles;
#[path = "aot_tests/manifest_cpu_target.rs"]
mod manifest_cpu_target;
#[path = "aot_tests/manifest_domain_units.rs"]
mod manifest_domain_units;
#[path = "aot_tests/shader_sidecar.rs"]
mod shader_sidecar;
#[path = "aot_tests/shim_file_io.rs"]
mod shim_file_io;
#[path = "aot_tests/shim_lifecycle.rs"]
mod shim_lifecycle;
#[path = "aot_tests/shim_process.rs"]
mod shim_process;
#[path = "aot_tests/shim_runtime_hooks.rs"]
mod shim_runtime_hooks;
#[path = "aot_tests/shim_serialization.rs"]
mod shim_serialization;
