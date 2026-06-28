use nuis_artifact::parse_domain_build_unit_blocks as shared_parse_domain_build_unit_blocks;
use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef, NirModule};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use yir_core::YirModule;

pub use crate::aot_artifact::{
    decode_nuis_compiled_artifact_binary, decode_nuis_executable_envelope_binary,
    encode_nuis_compiled_artifact_binary, encode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_executable_envelope_binary, inspect_nuis_compiled_artifact_container,
    parse_nuis_compiled_artifact, parse_nuis_executable_envelope,
    parse_nuis_executable_envelope_from_source, render_nuis_executable_envelope,
    validate_nuis_compiled_artifact_layout, write_nuis_compiled_artifact,
    write_nuis_executable_envelope, NuisCompiledArtifactContainerInspect,
    NuisCompiledArtifactLoweringUnitInspect,
};
use crate::aot_compiled_artifact_verify::verify_nuis_compiled_artifact_impl;
use crate::aot_domain_index_render::{
    append_relocated_bridge_registry_manifest_section,
    append_relocated_domain_lowering_plan_index_manifest_section,
    append_relocated_host_bridge_plan_index_manifest_section, render_domain_bridge_registry,
    render_domain_lowering_plan_index, render_host_bridge_plan_index,
};
use crate::aot_domain_index_verify::verify_domain_index_artifacts;
use crate::aot_domain_payload_blob::{
    decode_domain_build_unit_payload_blob, encode_domain_build_unit_payload_blob,
};
use crate::aot_domain_payload_verify::verify_domain_payload_blobs;
use crate::aot_domain_render::{
    render_domain_build_unit_backend_stub, render_domain_build_unit_bridge_plan,
    render_domain_build_unit_host_bridge_stub, render_domain_build_unit_lowering_plan,
};
use crate::aot_domain_unit_render::{
    render_domain_build_unit_manifest_block, render_domain_build_unit_payload,
    render_domain_build_unit_stub,
};
use crate::aot_domain_unit_verify::verify_domain_build_units;
use crate::aot_encoding::{fnv1a64_hex, hex_encode_bytes};
use crate::aot_kernel_sidecar::render_domain_build_unit_kernel_ir_sidecar;
use crate::aot_lifecycle::{
    build_nuis_envelope as build_nuis_envelope_from_domain_summaries,
    build_nuis_lifecycle_contract, NuisEnvelopeDomainSummary,
};
use crate::aot_manifest_core_verify::{verify_manifest_artifacts, verify_manifest_core};
use crate::aot_manifest_fields::verify_manifest_fields;
use crate::aot_manifest_report::build_manifest_verify_report;
use crate::aot_network_sidecar::render_domain_build_unit_network_ir_sidecar;
use crate::aot_project_metadata_verify::{
    project_metadata_summary_mismatch_error, verify_project_metadata_artifacts,
};
use crate::aot_shader_sidecar::render_domain_build_unit_shader_ir_sidecar;
use crate::aot_toml::{escape_toml_string, render_string_array};
pub use crate::aot_verify_report::{BuildManifestVerifyReport, NuisCompiledArtifactVerifyReport};
use crate::render;

pub struct CompileArtifacts {
    pub ast_path: String,
    pub nir_path: String,
    pub yir_path: String,
    pub llvm_ir_path: String,
    pub binary_path: String,
    pub packaging_mode: String,
}

pub struct BuildManifestProjectInfo {
    pub name: String,
    pub abi_mode: String,
    pub abi_graph_summary: Option<String>,
    pub abi_entries: Vec<(String, String)>,
    pub plan_summary: Option<String>,
    pub effective_input: Option<String>,
    pub text_handle_rewrite_helper_hits: usize,
    pub text_handle_rewrite_local_hits: usize,
    pub manifest_copy_path: Option<String>,
    pub plan_index_path: Option<String>,
    pub organization_index_path: Option<String>,
    pub exchange_index_path: Option<String>,
    pub modules_index_path: Option<String>,
    pub docs_index_path: Option<String>,
    pub docs_module_count: usize,
    pub docs_documented_module_count: usize,
    pub docs_documented_item_count: usize,
    pub imports_index_path: Option<String>,
    pub imports_library_count: usize,
    pub imports_visible_library_count: usize,
    pub imports_visible_module_count: usize,
    pub imports_documented_visible_module_count: usize,
    pub imports_documented_visible_item_count: usize,
    pub galaxy_index_path: Option<String>,
    pub galaxy_count: usize,
    pub galaxy_documented_count: usize,
    pub galaxy_documented_library_module_count: usize,
    pub galaxy_documented_item_count: usize,
    pub links_index_path: Option<String>,
    pub packet_index_path: Option<String>,
    pub host_ffi_index_path: Option<String>,
    pub abi_index_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildManifestDocIndexInfo {
    pub path: String,
    pub module_count: usize,
    pub documented_item_count: usize,
}

pub struct BuildManifestContext {
    pub input_path: String,
    pub output_dir: String,
    pub loaded_nustar: Vec<String>,
    pub compile_cache: Option<BuildManifestCacheInfo>,
    pub project: Option<BuildManifestProjectInfo>,
    pub doc_index: Option<BuildManifestDocIndexInfo>,
    pub cpu_target: CpuBuildTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BuildManifestExecutionContract {
    package_id: String,
    domain_family: String,
    execution: crate::registry::NustarExecutionSummary,
}

pub use nuis_artifact::{
    BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob, NuisCompiledArtifact,
    NuisExecutableEnvelope, NuisLifecycleContract,
};

fn normalize_manifest_path(path: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => out.push(part),
            Component::RootDir => out.push(component.as_os_str()),
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::ParentDir => {
                return Err(format!(
                    "path `{}` contains parent-directory traversal",
                    path.display()
                ));
            }
        }
    }
    Ok(out)
}

pub(crate) fn validate_manifest_path_in_output_dir(
    field: &str,
    value: &str,
    output_dir: &str,
    context: &Path,
) -> Result<(), String> {
    let output_path = Path::new(output_dir);
    let candidate_path = Path::new(value);
    if output_path.is_absolute() != candidate_path.is_absolute() {
        return Err(format!(
            "`{}` has unsafe {field} `{}`; path kind must match output_dir `{}`",
            context.display(),
            value,
            output_dir
        ));
    }
    let normalized_output = normalize_manifest_path(output_path).map_err(|error| {
        format!(
            "`{}` has unsafe output_dir `{}` while validating {field}: {error}",
            context.display(),
            output_dir
        )
    })?;
    let normalized_candidate = normalize_manifest_path(candidate_path).map_err(|error| {
        format!(
            "`{}` has unsafe {field} `{}`: {error}",
            context.display(),
            value
        )
    })?;
    if !normalized_candidate.starts_with(&normalized_output) {
        return Err(format!(
            "`{}` has unsafe {field} `{}`; expected path under output_dir `{}`",
            context.display(),
            value,
            output_dir
        ));
    }
    Ok(())
}

pub struct BuildManifestCacheInfo {
    pub status: String,
    pub key: String,
    pub root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuBuildTarget {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub isa_family: String,
    pub isa_features: Vec<String>,
    pub cross_compile: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VcsInfo {
    Git {
        root: String,
        head: String,
        dirty: bool,
    },
    None,
}

pub fn host_cpu_build_target() -> CpuBuildTarget {
    let machine_arch = host_machine_arch().to_owned();
    let machine_os = host_machine_os().to_owned();
    let object_format = host_object_format().to_owned();
    let calling_abi = host_calling_abi().to_owned();
    CpuBuildTarget {
        abi: format!("cpu.{machine_arch}.{calling_abi}"),
        machine_arch: machine_arch.clone(),
        machine_os: machine_os.clone(),
        object_format,
        calling_abi,
        clang_target: clang_target_triple(&machine_arch, &machine_os),
        isa_family: cpu_isa_family(&machine_arch).to_owned(),
        isa_features: default_cpu_isa_features(&machine_arch, &machine_os),
        cross_compile: false,
    }
}

fn cpu_isa_family(machine_arch: &str) -> &'static str {
    match machine_arch {
        "arm64" | "aarch64" => "aarch64",
        "x86_64" | "amd64" => "x86_64",
        _ => "generic",
    }
}

fn default_cpu_isa_features(machine_arch: &str, machine_os: &str) -> Vec<String> {
    let features = match cpu_isa_family(machine_arch) {
        "aarch64" => match machine_os {
            "darwin" => &["a64", "neon", "fp-armv8", "crc", "lse", "atomics"][..],
            "linux" => &["a64", "neon", "fp-armv8", "crc", "atomics"][..],
            _ => &["a64", "neon", "fp-armv8"][..],
        },
        "x86_64" => match machine_os {
            "windows" => &["x86-64", "sse2", "sse4.2", "popcnt"][..],
            _ => &["x86-64", "sse2", "sse4.2", "avx2", "bmi2", "popcnt"][..],
        },
        _ => &["scalar"][..],
    };
    features.iter().map(|item| (*item).to_owned()).collect()
}

fn canonical_machine_arch(machine_arch: &str) -> &str {
    match machine_arch {
        "amd64" => "x86_64",
        other => other,
    }
}

fn canonical_target_triple(target: &str) -> String {
    if let Some(rest) = target.strip_prefix("amd64-") {
        format!("x86_64-{rest}")
    } else {
        target.to_owned()
    }
}

pub fn resolve_cpu_build_target_from_project_abi(
    registry_root: &Path,
    resolution: Option<&crate::project::ProjectAbiResolution>,
) -> Result<CpuBuildTarget, String> {
    let Some(cpu_abi) = resolution.and_then(|resolution| {
        resolution
            .requirements
            .iter()
            .find(|item| item.domain == "cpu")
            .map(|item| item.abi.as_str())
    }) else {
        return Ok(host_cpu_build_target());
    };
    resolve_cpu_build_target_from_abi(registry_root, cpu_abi)
}

pub fn resolve_cpu_build_target(
    registry_root: &Path,
    resolution: Option<&crate::project::ProjectAbiResolution>,
    cpu_abi_override: Option<&str>,
    target_override: Option<&str>,
) -> Result<CpuBuildTarget, String> {
    let mut target = if let Some(cpu_abi) = cpu_abi_override {
        resolve_cpu_build_target_from_abi(registry_root, cpu_abi)?
    } else if let Some(target) = target_override {
        resolve_cpu_build_target_from_target(registry_root, target)?
    } else {
        resolve_cpu_build_target_from_project_abi(registry_root, resolution)?
    };

    if let Some(target_text) = target_override {
        let explicit_target = resolve_cpu_build_target_from_target(registry_root, target_text)?;
        if target.machine_arch != explicit_target.machine_arch
            || target.machine_os != explicit_target.machine_os
        {
            return Err(format!(
                "`--cpu-abi {}` resolves to {}-{}, but `--target {}` resolves to {}-{}",
                target.abi,
                target.machine_arch,
                target.machine_os,
                target_text,
                explicit_target.machine_arch,
                explicit_target.machine_os
            ));
        }
        target.clang_target = explicit_target.clang_target;
        target.machine_arch = explicit_target.machine_arch;
        target.machine_os = explicit_target.machine_os;
        target.object_format = explicit_target.object_format;
        target.calling_abi = explicit_target.calling_abi;
        target.cross_compile = explicit_target.cross_compile;
    }

    Ok(target)
}

pub fn resolve_cpu_build_target_from_abi(
    registry_root: &Path,
    abi: &str,
) -> Result<CpuBuildTarget, String> {
    let manifest = crate::registry::load_manifest_for_domain(registry_root, "cpu")?;
    crate::registry::validate_manifest_abi(&manifest, abi)?;
    let registered = crate::registry::registered_abi_target(&manifest, abi)?;
    Ok(CpuBuildTarget {
        abi: registered.abi,
        machine_arch: registered.machine_arch.clone(),
        machine_os: registered.machine_os.clone(),
        object_format: registered.object_format,
        calling_abi: registered.calling_abi,
        clang_target: registered.clang_target,
        isa_family: cpu_isa_family(&registered.machine_arch).to_owned(),
        isa_features: default_cpu_isa_features(&registered.machine_arch, &registered.machine_os),
        cross_compile: registered.machine_arch != host_machine_arch()
            || registered.machine_os != host_machine_os(),
    })
}

pub fn resolve_cpu_build_target_from_target(
    registry_root: &Path,
    target: &str,
) -> Result<CpuBuildTarget, String> {
    let manifest = crate::registry::load_manifest_for_domain(registry_root, "cpu")?;
    let canonical_target = canonical_target_triple(target);
    let registered =
        crate::registry::registered_abi_target_for_clang(&manifest, &canonical_target)?;
    Ok(CpuBuildTarget {
        abi: registered.abi,
        machine_arch: registered.machine_arch.clone(),
        machine_os: registered.machine_os.clone(),
        object_format: registered.object_format,
        calling_abi: registered.calling_abi,
        clang_target: registered.clang_target,
        isa_family: cpu_isa_family(&registered.machine_arch).to_owned(),
        isa_features: default_cpu_isa_features(&registered.machine_arch, &registered.machine_os),
        cross_compile: registered.machine_arch != host_machine_arch()
            || registered.machine_os != host_machine_os(),
    })
}

pub fn write_and_link(
    input: &Path,
    output_dir: &Path,
    ast: &AstModule,
    nir: &NirModule,
    yir: &YirModule,
    llvm_ir: &str,
    cpu_target: &CpuBuildTarget,
) -> Result<CompileArtifacts, String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;

    let layout = output_layout(input, output_dir);
    let ast_path = layout.ast_path;
    let nir_path = layout.nir_path;
    let yir_path = layout.yir_path;
    let ll_path = layout.llvm_ir_path;
    let shim_path = layout.shim_path;
    let exe_path = layout.binary_stub_path;

    fs::write(&ast_path, render::render_ast(ast))
        .map_err(|error| format!("failed to write `{}`: {error}", ast_path.display()))?;
    fs::write(&nir_path, render::render_nir(nir))
        .map_err(|error| format!("failed to write `{}`: {error}", nir_path.display()))?;
    fs::write(&yir_path, render::render_yir(yir))
        .map_err(|error| format!("failed to write `{}`: {error}", yir_path.display()))?;
    fs::write(&ll_path, llvm_ir)
        .map_err(|error| format!("failed to write `{}`: {error}", ll_path.display()))?;
    fs::write(&shim_path, c_shim_source(ast))
        .map_err(|error| format!("failed to write `{}`: {error}", shim_path.display()))?;

    let (binary_path, packaging_mode) = if requires_window_bundle(yir) {
        build_window_bundle(&yir_path, output_dir, &exe_path, cpu_target)?
    } else {
        compile_native_binary(&ll_path, &shim_path, &exe_path, cpu_target)?;
        (exe_path.display().to_string(), "native-cpu-llvm".to_owned())
    };

    Ok(CompileArtifacts {
        ast_path: ast_path.display().to_string(),
        nir_path: nir_path.display().to_string(),
        yir_path: yir_path.display().to_string(),
        llvm_ir_path: ll_path.display().to_string(),
        binary_path,
        packaging_mode,
    })
}

pub fn compile_artifacts_for_output_dir(
    input: &Path,
    output_dir: &Path,
    yir: &YirModule,
) -> Result<CompileArtifacts, String> {
    let packaging_mode = if requires_window_bundle(yir) {
        "window-aot-bundle"
    } else {
        "native-cpu-llvm"
    };
    compile_artifacts_for_output_dir_with_packaging_mode(input, output_dir, packaging_mode)
}

pub fn compile_artifacts_for_output_dir_with_packaging_mode(
    input: &Path,
    output_dir: &Path,
    packaging_mode: &str,
) -> Result<CompileArtifacts, String> {
    let layout = output_layout(input, output_dir);
    if packaging_mode != "window-aot-bundle" && packaging_mode != "native-cpu-llvm" {
        return Err(format!(
            "unsupported cached packaging_mode `{packaging_mode}` for `{}`",
            output_dir.display()
        ));
    }
    Ok(CompileArtifacts {
        ast_path: layout.ast_path.display().to_string(),
        nir_path: layout.nir_path.display().to_string(),
        yir_path: layout.yir_path.display().to_string(),
        llvm_ir_path: layout.llvm_ir_path.display().to_string(),
        binary_path: layout.binary_stub_path.display().to_string(),
        packaging_mode: packaging_mode.to_owned(),
    })
}

struct OutputLayout {
    ast_path: PathBuf,
    nir_path: PathBuf,
    yir_path: PathBuf,
    llvm_ir_path: PathBuf,
    shim_path: PathBuf,
    binary_stub_path: PathBuf,
}

fn output_layout(input: &Path, output_dir: &Path) -> OutputLayout {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("nuis_module");
    OutputLayout {
        ast_path: output_dir.join(format!("{stem}.ast.txt")),
        nir_path: output_dir.join(format!("{stem}.nir.txt")),
        yir_path: output_dir.join(format!("{stem}.yir")),
        llvm_ir_path: output_dir.join(format!("{stem}.ll")),
        shim_path: output_dir.join(format!("{stem}_shim.c")),
        binary_stub_path: output_dir.join(stem),
    }
}

pub fn write_build_manifest(
    output_dir: &Path,
    written: &CompileArtifacts,
    context: &BuildManifestContext,
) -> Result<String, String> {
    let path = output_dir.join("nuis.build.manifest.toml");
    let envelope_path = output_dir.join("nuis.executable.envelope.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_secs();
    let engine = crate::engine::default_engine();
    let vcs = detect_vcs_info(&context.input_path, &context.output_dir);

    let mut loaded_nustar = context.loaded_nustar.clone();
    loaded_nustar.sort();
    loaded_nustar.dedup();
    let execution_contracts = resolve_execution_contracts(&loaded_nustar)?;
    let mut domain_build_units = build_manifest_domain_units(context, &execution_contracts)?;
    let envelope = build_nuis_envelope(&execution_contracts, &written.packaging_mode);
    let lifecycle = build_nuis_lifecycle_contract(&envelope, &written.packaging_mode);
    let compiled_binary_name = Path::new(&written.binary_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-binary")
        .to_owned();
    let compiled_binary_bytes = fs::metadata(&written.binary_path)
        .map_err(|error| format!("failed to stat `{}`: {error}", written.binary_path))?
        .len() as usize;

    let mut artifacts = vec![
        ("ast".to_owned(), PathBuf::from(&written.ast_path)),
        ("nir".to_owned(), PathBuf::from(&written.nir_path)),
        ("yir".to_owned(), PathBuf::from(&written.yir_path)),
        ("llvm_ir".to_owned(), PathBuf::from(&written.llvm_ir_path)),
        ("binary".to_owned(), PathBuf::from(&written.binary_path)),
    ];
    artifacts.extend(write_domain_build_unit_stubs(
        output_dir,
        &mut domain_build_units,
    )?);
    let hetero_units = domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let bridge_registry_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_domain_bridge_registry(&hetero_units))
    };
    let host_bridge_plan_index_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_host_bridge_plan_index(&hetero_units))
    };
    let lowering_plan_index_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_domain_lowering_plan_index(&hetero_units))
    };

    let bridge_registry_path = write_domain_bridge_registry(output_dir, &domain_build_units)?;
    if let Some(bridge_registry_path) = &bridge_registry_path {
        artifacts.push((
            "domain_bridge_registry".to_owned(),
            bridge_registry_path.clone(),
        ));
    }
    let host_bridge_plan_index_path =
        write_host_bridge_plan_index(output_dir, &domain_build_units)?;
    if let Some(host_bridge_plan_index_path) = &host_bridge_plan_index_path {
        artifacts.push((
            "host_bridge_plan_index".to_owned(),
            host_bridge_plan_index_path.clone(),
        ));
    }
    let lowering_plan_index_path =
        write_domain_lowering_plan_index(output_dir, &domain_build_units)?;
    if let Some(lowering_plan_index_path) = &lowering_plan_index_path {
        artifacts.push((
            "domain_lowering_plan_index".to_owned(),
            lowering_plan_index_path.clone(),
        ));
    }

    let mut out = String::new();
    out.push_str("manifest_schema = \"nuis-build-manifest-v1\"\n");
    out.push_str(&format!("generated_at_unix = {generated_at_unix}\n"));
    out.push_str(&format!(
        "input = \"{}\"\n",
        escape_toml_string(&context.input_path)
    ));
    out.push_str(&format!(
        "output_dir = \"{}\"\n",
        escape_toml_string(&context.output_dir)
    ));
    out.push_str(&format!(
        "packaging_mode = \"{}\"\n",
        escape_toml_string(&written.packaging_mode)
    ));
    out.push_str(&format!(
        "cpu_target_abi = \"{}\"\n",
        escape_toml_string(&context.cpu_target.abi)
    ));
    out.push_str(&format!(
        "cpu_target_machine_arch = \"{}\"\n",
        escape_toml_string(&context.cpu_target.machine_arch)
    ));
    out.push_str(&format!(
        "cpu_target_machine_os = \"{}\"\n",
        escape_toml_string(&context.cpu_target.machine_os)
    ));
    out.push_str(&format!(
        "cpu_target_object_format = \"{}\"\n",
        escape_toml_string(&context.cpu_target.object_format)
    ));
    out.push_str(&format!(
        "cpu_target_calling_abi = \"{}\"\n",
        escape_toml_string(&context.cpu_target.calling_abi)
    ));
    out.push_str(&format!(
        "cpu_target_clang = \"{}\"\n",
        escape_toml_string(&context.cpu_target.clang_target)
    ));
    out.push_str(&format!(
        "cpu_target_cross = {}\n",
        if context.cpu_target.cross_compile {
            "true"
        } else {
            "false"
        }
    ));
    out.push_str(&format!(
        "tool_nuisc = \"{}\"\n",
        escape_toml_string(env!("CARGO_PKG_VERSION"))
    ));
    out.push_str(&format!(
        "engine_version = \"{}\"\n",
        escape_toml_string(engine.version)
    ));
    out.push_str(&format!(
        "engine_profile = \"{}\"\n",
        escape_toml_string(engine.profile)
    ));
    match vcs {
        VcsInfo::Git { root, head, dirty } => {
            out.push_str("vcs = \"git\"\n");
            out.push_str(&format!(
                "vcs_dirty = {}\n",
                if dirty { "true" } else { "false" }
            ));
            out.push_str(&format!("vcs_head = \"{}\"\n", escape_toml_string(&head)));
            out.push_str(&format!("vcs_root = \"{}\"\n", escape_toml_string(&root)));
        }
        VcsInfo::None => {
            out.push_str("vcs = \"none\"\n");
        }
    }
    out.push_str(&format!(
        "loaded_nustar = {}\n",
        render_string_array(&loaded_nustar)
    ));
    write_nuis_executable_envelope(&envelope_path, &envelope)?;
    out.push('\n');
    out.push_str("[nuis_envelope]\n");
    out.push_str(&format!(
        "path = \"{}\"\n",
        escape_toml_string(&envelope_path.display().to_string())
    ));
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(&envelope.schema)
    ));
    out.push_str(&format!(
        "executable_kind = \"{}\"\n",
        escape_toml_string(&envelope.executable_kind)
    ));
    out.push_str(&format!("package_count = {}\n", envelope.package_count));
    out.push_str(&format!(
        "domain_families = {}\n",
        render_string_array(&envelope.domain_families)
    ));
    out.push_str(&format!(
        "contract_families = {}\n",
        render_string_array(&envelope.contract_families)
    ));
    out.push_str(&format!(
        "function_kind = \"{}\"\n",
        escape_toml_string(&envelope.function_kind)
    ));
    out.push_str(&format!(
        "graph_kind = \"{}\"\n",
        escape_toml_string(&envelope.graph_kind)
    ));
    out.push_str(&format!(
        "default_time_mode = \"{}\"\n",
        escape_toml_string(&envelope.default_time_mode)
    ));
    out.push('\n');
    out.push_str("[nuis_artifact]\n");
    out.push_str(&format!(
        "artifact_path = \"{}\"\n",
        escape_toml_string(&artifact_path.display().to_string())
    ));
    out.push_str(&format!(
        "artifact_schema = \"{}\"\n",
        escape_toml_string("nuis-compiled-artifact-v1")
    ));
    out.push_str(&format!(
        "artifact_binary_name = \"{}\"\n",
        escape_toml_string(&compiled_binary_name)
    ));
    out.push_str(&format!(
        "artifact_binary_bytes = {}\n",
        compiled_binary_bytes
    ));
    out.push('\n');
    out.push_str("[nuis_lifecycle]\n");
    out.push_str(&format!(
        "lifecycle_schema = \"{}\"\n",
        escape_toml_string(&lifecycle.schema)
    ));
    out.push_str(&format!(
        "lifecycle_bootstrap_entry = \"{}\"\n",
        escape_toml_string(&lifecycle.bootstrap_entry)
    ));
    out.push_str(&format!(
        "lifecycle_tick_policy = \"{}\"\n",
        escape_toml_string(&lifecycle.tick_policy)
    ));
    out.push_str(&format!(
        "lifecycle_shutdown_policy = \"{}\"\n",
        escape_toml_string(&lifecycle.shutdown_policy)
    ));
    out.push_str(&format!(
        "lifecycle_yalivia_rpc = \"{}\"\n",
        escape_toml_string(&lifecycle.yalivia_rpc)
    ));
    out.push_str(&format!(
        "lifecycle_hook_surface = {}\n",
        render_string_array(&lifecycle.hook_surface)
    ));
    out.push_str(&format!(
        "lifecycle_export_surface = {}\n",
        render_string_array(&lifecycle.export_surface)
    ));
    out.push_str(&format!(
        "lifecycle_runtime_capability_flags = {}\n",
        render_string_array(&lifecycle.runtime_capability_flags)
    ));
    if let Some(cache) = &context.compile_cache {
        out.push_str(&format!(
            "compile_cache_status = \"{}\"\n",
            escape_toml_string(&cache.status)
        ));
        out.push_str(&format!(
            "compile_cache_key = \"{}\"\n",
            escape_toml_string(&cache.key)
        ));
        out.push_str(&format!(
            "compile_cache_root = \"{}\"\n",
            escape_toml_string(&cache.root)
        ));
    }
    if let Some(doc_index) = &context.doc_index {
        out.push_str(&format!(
            "doc_index_path = \"{}\"\n",
            escape_toml_string(&doc_index.path)
        ));
        out.push_str(&format!(
            "doc_index_module_count = {}\n",
            doc_index.module_count
        ));
        out.push_str(&format!(
            "doc_index_documented_item_count = {}\n",
            doc_index.documented_item_count
        ));
    }
    out.push('\n');
    out.push_str("[artifacts]\n");
    for (kind, artifact_path) in &artifacts {
        out.push_str(&format!(
            "{kind} = \"{}\"\n",
            escape_toml_string(&artifact_path.display().to_string())
        ));
    }

    if let Some(bridge_registry_path) = &bridge_registry_path {
        out.push('\n');
        out.push_str("[bridge_registry]\n");
        out.push_str(&format!(
            "bridge_registry_path = \"{}\"\n",
            escape_toml_string(&bridge_registry_path.display().to_string())
        ));
        out.push_str("bridge_registry_schema = \"nuis-bridge-registry-v1\"\n");
        out.push_str(&format!(
            "bridge_registry_units = {}\n",
            domain_build_units
                .iter()
                .filter(|unit| unit.domain_family != "cpu")
                .count()
        ));
        if let Some(source) = &bridge_registry_inline {
            out.push_str(&format!(
                "bridge_registry_inline = \"{}\"\n",
                escape_toml_string(source)
            ));
        }
    }
    if let Some(host_bridge_plan_index_path) = &host_bridge_plan_index_path {
        out.push('\n');
        out.push_str("[host_bridge_plan_index]\n");
        out.push_str(&format!(
            "host_bridge_plan_index_path = \"{}\"\n",
            escape_toml_string(&host_bridge_plan_index_path.display().to_string())
        ));
        out.push_str("host_bridge_plan_index_schema = \"nuis-host-bridge-plan-index-v1\"\n");
        out.push_str(&format!(
            "host_bridge_plan_units = {}\n",
            domain_build_units
                .iter()
                .filter(|unit| unit.domain_family != "cpu")
                .count()
        ));
        if let Some(source) = &host_bridge_plan_index_inline {
            out.push_str(&format!(
                "host_bridge_plan_index_inline = \"{}\"\n",
                escape_toml_string(source)
            ));
        }
    }
    if let Some(lowering_plan_index_path) = &lowering_plan_index_path {
        out.push('\n');
        out.push_str("[domain_lowering_plan_index]\n");
        out.push_str(&format!(
            "lowering_plan_index_path = \"{}\"\n",
            escape_toml_string(&lowering_plan_index_path.display().to_string())
        ));
        out.push_str("lowering_plan_index_schema = \"nuis-domain-lowering-plan-index-v1\"\n");
        out.push_str(&format!(
            "lowering_plan_units = {}\n",
            domain_build_units
                .iter()
                .filter(|unit| unit.domain_family != "cpu")
                .count()
        ));
        if let Some(source) = &lowering_plan_index_inline {
            out.push_str(&format!(
                "lowering_plan_index_inline = \"{}\"\n",
                escape_toml_string(source)
            ));
        }
    }

    for (kind, artifact_path) in &artifacts {
        let bytes = fs::read(artifact_path).map_err(|error| {
            format!(
                "failed to read artifact `{}`: {error}",
                artifact_path.display()
            )
        })?;
        out.push('\n');
        out.push_str("[[artifact_hash]]\n");
        out.push_str(&format!("kind = \"{}\"\n", escape_toml_string(kind)));
        out.push_str(&format!(
            "path = \"{}\"\n",
            escape_toml_string(&artifact_path.display().to_string())
        ));
        out.push_str(&format!("bytes = {}\n", bytes.len()));
        out.push_str(&format!("fnv1a64 = \"{}\"\n", fnv1a64_hex(&bytes)));
    }

    for contract in &execution_contracts {
        out.push('\n');
        out.push_str("[[execution_contract]]\n");
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&contract.package_id)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&contract.domain_family)
        ));
        out.push_str(&format!(
            "skeleton_version = \"{}\"\n",
            escape_toml_string(&contract.execution.skeleton_version)
        ));
        out.push_str(&format!(
            "function_kind = \"{}\"\n",
            escape_toml_string(&contract.execution.function_kind)
        ));
        out.push_str(&format!(
            "graph_kind = \"{}\"\n",
            escape_toml_string(&contract.execution.graph_kind)
        ));
        out.push_str(&format!(
            "execution_domain = \"{}\"\n",
            escape_toml_string(&contract.execution.execution_domain)
        ));
        out.push_str(&format!(
            "default_time_mode = \"{}\"\n",
            escape_toml_string(&contract.execution.default_time_mode)
        ));
        out.push_str(&format!(
            "contract_family = \"{}\"\n",
            escape_toml_string(&contract.execution.contract_family)
        ));
        out.push_str(&format!(
            "lowering_targets = {}\n",
            render_string_array(&contract.execution.lowering_targets)
        ));
    }

    for unit in &domain_build_units {
        out.push('\n');
        out.push_str("[[domain_build_unit]]\n");
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        if let Some(value) = &unit.abi {
            out.push_str(&format!("abi = \"{}\"\n", escape_toml_string(value)));
        }
        if let Some(value) = &unit.machine_arch {
            out.push_str(&format!(
                "machine_arch = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.machine_os {
            out.push_str(&format!("machine_os = \"{}\"\n", escape_toml_string(value)));
        }
        if let Some(value) = &unit.backend_family {
            out.push_str(&format!(
                "backend_family = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.vendor {
            out.push_str(&format!("vendor = \"{}\"\n", escape_toml_string(value)));
        }
        if let Some(value) = &unit.device_class {
            out.push_str(&format!(
                "device_class = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.selected_lowering_target {
            out.push_str(&format!(
                "selected_lowering_target = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_stub_path {
            out.push_str(&format!(
                "artifact_stub_path = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_stub_inline {
            out.push_str(&format!(
                "artifact_stub_inline = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_payload_path {
            out.push_str(&format!(
                "artifact_payload_path = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_bridge_stub_path {
            out.push_str(&format!(
                "artifact_bridge_stub_path = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_ir_sidecar_path {
            out.push_str(&format!(
                "artifact_ir_sidecar_path = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_bridge_stub_inline {
            out.push_str(&format!(
                "artifact_bridge_stub_inline = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_payload_blob_path {
            out.push_str(&format!(
                "artifact_payload_blob_path = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = unit.artifact_payload_blob_bytes {
            out.push_str(&format!("artifact_payload_blob_bytes = {}\n", value));
        }
        if let Some(value) = &unit.artifact_payload_format {
            out.push_str(&format!(
                "artifact_payload_format = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_payload_blob_inline {
            out.push_str(&format!(
                "artifact_payload_blob_inline = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        out.push_str(&format!(
            "contract_family = \"{}\"\n",
            escape_toml_string(&unit.contract_family)
        ));
        out.push_str(&format!(
            "packaging_role = \"{}\"\n",
            escape_toml_string(&unit.packaging_role)
        ));
    }

    if let Some(project) = &context.project {
        out.push('\n');
        out.push_str("[project]\n");
        out.push_str(&format!(
            "name = \"{}\"\n",
            escape_toml_string(&project.name)
        ));
        out.push_str(&format!(
            "abi_mode = \"{}\"\n",
            escape_toml_string(&project.abi_mode)
        ));
        if let Some(value) = &project.abi_graph_summary {
            out.push_str(&format!("abi_graph = \"{}\"\n", escape_toml_string(value)));
        }
        if let Some(value) = &project.plan_summary {
            out.push_str(&format!(
                "plan_summary = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.effective_input {
            out.push_str(&format!(
                "effective_input = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        out.push_str(&format!(
            "text_handle_rewrite_helper_hits = {}\n",
            project.text_handle_rewrite_helper_hits
        ));
        out.push_str(&format!(
            "text_handle_rewrite_local_hits = {}\n",
            project.text_handle_rewrite_local_hits
        ));
        let mut abi_entries = project
            .abi_entries
            .iter()
            .map(|(domain, abi)| format!("{domain}={abi}"))
            .collect::<Vec<_>>();
        abi_entries.sort();
        out.push_str(&format!("abi = {}\n", render_string_array(&abi_entries)));
        if let Some(value) = &project.manifest_copy_path {
            out.push_str(&format!(
                "manifest_copy = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.plan_index_path {
            out.push_str(&format!("plan_index = \"{}\"\n", escape_toml_string(value)));
        }
        if let Some(value) = &project.organization_index_path {
            out.push_str(&format!(
                "organization_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.exchange_index_path {
            out.push_str(&format!(
                "exchange_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.modules_index_path {
            out.push_str(&format!(
                "modules_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.docs_index_path {
            out.push_str(&format!("docs_index = \"{}\"\n", escape_toml_string(value)));
        }
        out.push_str(&format!(
            "docs_module_count = {}\n",
            project.docs_module_count
        ));
        out.push_str(&format!(
            "docs_documented_module_count = {}\n",
            project.docs_documented_module_count
        ));
        out.push_str(&format!(
            "docs_documented_item_count = {}\n",
            project.docs_documented_item_count
        ));
        if let Some(value) = &project.imports_index_path {
            out.push_str(&format!(
                "imports_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        out.push_str(&format!(
            "imports_library_count = {}\n",
            project.imports_library_count
        ));
        out.push_str(&format!(
            "imports_visible_library_count = {}\n",
            project.imports_visible_library_count
        ));
        out.push_str(&format!(
            "imports_visible_module_count = {}\n",
            project.imports_visible_module_count
        ));
        out.push_str(&format!(
            "imports_documented_visible_module_count = {}\n",
            project.imports_documented_visible_module_count
        ));
        out.push_str(&format!(
            "imports_documented_visible_item_count = {}\n",
            project.imports_documented_visible_item_count
        ));
        if let Some(value) = &project.galaxy_index_path {
            out.push_str(&format!(
                "galaxy_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        out.push_str(&format!("galaxy_count = {}\n", project.galaxy_count));
        out.push_str(&format!(
            "documented_galaxy_count = {}\n",
            project.galaxy_documented_count
        ));
        out.push_str(&format!(
            "documented_galaxy_library_module_count = {}\n",
            project.galaxy_documented_library_module_count
        ));
        out.push_str(&format!(
            "documented_galaxy_item_count = {}\n",
            project.galaxy_documented_item_count
        ));
        if let Some(value) = &project.links_index_path {
            out.push_str(&format!(
                "links_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.packet_index_path {
            out.push_str(&format!(
                "packet_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.host_ffi_index_path {
            out.push_str(&format!(
                "host_ffi_index = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &project.abi_index_path {
            out.push_str(&format!("abi_index = \"{}\"\n", escape_toml_string(value)));
        }
    }

    let compiled_artifact =
        build_nuis_compiled_artifact(written, context, &envelope, &lifecycle, &out)?;
    write_nuis_compiled_artifact(&artifact_path, &compiled_artifact)?;
    fs::write(&path, out)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(path.display().to_string())
}

fn build_nuis_compiled_artifact(
    written: &CompileArtifacts,
    context: &BuildManifestContext,
    envelope: &NuisExecutableEnvelope,
    lifecycle: &NuisLifecycleContract,
    build_manifest_source: &str,
) -> Result<NuisCompiledArtifact, String> {
    let binary_blob = fs::read(&written.binary_path).map_err(|error| {
        format!(
            "failed to read compiled binary `{}`: {error}",
            written.binary_path
        )
    })?;
    let binary_name = Path::new(&written.binary_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-binary")
        .to_owned();
    Ok(NuisCompiledArtifact {
        schema: "nuis-compiled-artifact-v1".to_owned(),
        packaging_mode: written.packaging_mode.clone(),
        cpu_target_abi: context.cpu_target.abi.clone(),
        cpu_target_machine_arch: context.cpu_target.machine_arch.clone(),
        cpu_target_machine_os: context.cpu_target.machine_os.clone(),
        cpu_target_object_format: context.cpu_target.object_format.clone(),
        cpu_target_calling_abi: context.cpu_target.calling_abi.clone(),
        binary_name,
        binary_bytes: binary_blob.len(),
        build_manifest_bytes: build_manifest_source.len(),
        envelope: envelope.clone(),
        lifecycle: lifecycle.clone(),
        build_manifest_source: build_manifest_source.to_owned(),
        binary_blob,
    })
}

pub fn render_relocated_unpacked_build_manifest(
    artifact: &NuisCompiledArtifact,
    output_dir: &Path,
    envelope_path: &Path,
    artifact_path: &Path,
    binary_path: &Path,
) -> Result<String, String> {
    let mut out = String::new();
    let source = &artifact.build_manifest_source;
    let mut domain_build_units = parse_domain_build_unit_blocks(source, Path::new("<artifact>"))?;
    write_domain_build_unit_stubs(output_dir, &mut domain_build_units)?;
    let bridge_registry_path = write_domain_bridge_registry(output_dir, &domain_build_units)?;
    let host_bridge_plan_index_path =
        write_host_bridge_plan_index(output_dir, &domain_build_units)?;
    let lowering_plan_index_path =
        write_domain_lowering_plan_index(output_dir, &domain_build_units)?;
    let mut skip_section = false;
    let strip_project_path_keys = [
        "manifest_copy = ",
        "plan_index = ",
        "organization_index = ",
        "exchange_index = ",
        "modules_index = ",
        "links_index = ",
        "packet_index = ",
        "host_ffi_index = ",
        "abi_index = ",
    ];

    for raw in source.lines() {
        let line = raw.trim();
        if line == "[nuis_envelope]" || line == "[nuis_artifact]" || line == "[artifacts]" {
            skip_section = true;
            continue;
        }
        if line == "[bridge_registry]"
            || line == "[host_bridge_plan_index]"
            || line == "[domain_lowering_plan_index]"
        {
            skip_section = true;
            continue;
        }
        if line == "[[domain_build_unit]]" {
            skip_section = true;
            continue;
        }
        if line == "[[artifact_hash]]" {
            skip_section = true;
            continue;
        }
        if skip_section && line.starts_with('[') {
            skip_section = false;
        }
        if skip_section {
            continue;
        }
        if line.starts_with("output_dir = ") {
            out.push_str(&format!(
                "output_dir = \"{}\"\n",
                escape_toml_string(&output_dir.display().to_string())
            ));
            continue;
        }
        if strip_project_path_keys
            .iter()
            .any(|prefix| line.starts_with(prefix))
        {
            continue;
        }
        out.push_str(raw);
        out.push('\n');
    }

    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }

    for unit in &domain_build_units {
        out.push_str(&render_domain_build_unit_manifest_block(unit));
    }
    append_relocated_bridge_registry_manifest_section(
        &mut out,
        bridge_registry_path.as_deref(),
        &domain_build_units,
    );
    append_relocated_host_bridge_plan_index_manifest_section(
        &mut out,
        host_bridge_plan_index_path.as_deref(),
        &domain_build_units,
    );
    append_relocated_domain_lowering_plan_index_manifest_section(
        &mut out,
        lowering_plan_index_path.as_deref(),
        &domain_build_units,
    );

    out.push_str("[nuis_envelope]\n");
    out.push_str(&format!(
        "path = \"{}\"\n",
        escape_toml_string(&envelope_path.display().to_string())
    ));
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(&artifact.envelope.schema)
    ));
    out.push_str(&format!(
        "executable_kind = \"{}\"\n",
        escape_toml_string(&artifact.envelope.executable_kind)
    ));
    out.push_str(&format!(
        "package_count = {}\n",
        artifact.envelope.package_count
    ));
    out.push_str(&format!(
        "domain_families = {}\n",
        render_string_array(&artifact.envelope.domain_families)
    ));
    out.push_str(&format!(
        "contract_families = {}\n",
        render_string_array(&artifact.envelope.contract_families)
    ));
    out.push_str(&format!(
        "function_kind = \"{}\"\n",
        escape_toml_string(&artifact.envelope.function_kind)
    ));
    out.push_str(&format!(
        "graph_kind = \"{}\"\n",
        escape_toml_string(&artifact.envelope.graph_kind)
    ));
    out.push_str(&format!(
        "default_time_mode = \"{}\"\n",
        escape_toml_string(&artifact.envelope.default_time_mode)
    ));
    out.push('\n');

    out.push_str("[nuis_artifact]\n");
    out.push_str(&format!(
        "artifact_path = \"{}\"\n",
        escape_toml_string(&artifact_path.display().to_string())
    ));
    out.push_str(&format!(
        "artifact_schema = \"{}\"\n",
        escape_toml_string(&artifact.schema)
    ));
    out.push_str(&format!(
        "artifact_binary_name = \"{}\"\n",
        escape_toml_string(&artifact.binary_name)
    ));
    out.push_str(&format!(
        "artifact_binary_bytes = {}\n",
        artifact.binary_bytes
    ));
    out.push('\n');

    out.push_str("[artifacts]\n");
    out.push_str(&format!(
        "binary = \"{}\"\n",
        escape_toml_string(&binary_path.display().to_string())
    ));
    out.push_str(&format!(
        "envelope = \"{}\"\n",
        escape_toml_string(&envelope_path.display().to_string())
    ));
    out.push('\n');

    for (kind, path) in [("binary", binary_path), ("envelope", envelope_path)] {
        let bytes = fs::read(path).map_err(|error| {
            format!(
                "failed to read unpacked artifact `{}`: {error}",
                path.display()
            )
        })?;
        out.push_str("[[artifact_hash]]\n");
        out.push_str(&format!("kind = \"{}\"\n", escape_toml_string(kind)));
        out.push_str(&format!(
            "path = \"{}\"\n",
            escape_toml_string(&path.display().to_string())
        ));
        out.push_str(&format!("bytes = {}\n", bytes.len()));
        out.push_str(&format!("fnv1a64 = \"{}\"\n", fnv1a64_hex(&bytes)));
        out.push('\n');
    }

    Ok(out)
}

fn resolve_execution_contracts(
    loaded_nustar: &[String],
) -> Result<Vec<BuildManifestExecutionContract>, String> {
    let mut contracts = Vec::new();
    for package_id in loaded_nustar {
        let manifest =
            match crate::registry::load_manifest(Path::new("nustar-packages"), package_id) {
                Ok(manifest) => manifest,
                Err(package_error) => {
                    let Some(domain) = package_id.strip_prefix("official.") else {
                        return Err(package_error);
                    };
                    crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), domain)
                        .map_err(|_| package_error)?
                }
            };
        contracts.push(BuildManifestExecutionContract {
            package_id: manifest.package_id.clone(),
            domain_family: manifest.domain_family.clone(),
            execution: crate::registry::execution_summary(&manifest),
        });
    }
    Ok(contracts)
}

fn build_manifest_domain_units(
    context: &BuildManifestContext,
    execution_contracts: &[BuildManifestExecutionContract],
) -> Result<Vec<BuildManifestDomainBuildUnit>, String> {
    let mut abi_by_domain = BTreeMap::<String, String>::new();
    abi_by_domain.insert("cpu".to_owned(), context.cpu_target.abi.clone());
    if let Some(project) = &context.project {
        for (domain, abi) in &project.abi_entries {
            abi_by_domain.insert(domain.clone(), abi.clone());
        }
    }

    let mut units = execution_contracts
        .iter()
        .map(|contract| {
            let abi = abi_by_domain.get(&contract.domain_family).cloned();
            let (
                machine_arch,
                machine_os,
                backend_family,
                vendor,
                device_class,
                selected_lowering_target,
            ) = resolve_domain_build_unit_target(&contract.domain_family, abi.as_deref())?;
            Ok(BuildManifestDomainBuildUnit {
                package_id: contract.package_id.clone(),
                domain_family: contract.domain_family.clone(),
                abi,
                machine_arch,
                machine_os,
                backend_family,
                vendor,
                device_class,
                selected_lowering_target,
                artifact_stub_path: None,
                artifact_stub_inline: None,
                artifact_payload_path: None,
                artifact_bridge_stub_path: None,
                artifact_ir_sidecar_path: None,
                artifact_bridge_stub_inline: None,
                artifact_payload_blob_path: None,
                artifact_payload_blob_bytes: None,
                artifact_payload_format: None,
                artifact_payload_blob_inline: None,
                contract_family: contract.execution.contract_family.clone(),
                packaging_role: if contract.domain_family == "cpu" {
                    "host-binary".to_owned()
                } else {
                    "hetero-contract".to_owned()
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    units.sort_by(|lhs, rhs| {
        lhs.domain_family
            .cmp(&rhs.domain_family)
            .then_with(|| lhs.package_id.cmp(&rhs.package_id))
    });
    Ok(units)
}

fn write_domain_build_unit_stubs(
    output_dir: &Path,
    units: &mut [BuildManifestDomainBuildUnit],
) -> Result<Vec<(String, PathBuf)>, String> {
    let mut artifacts = Vec::new();
    for unit in units {
        if unit.domain_family == "cpu" {
            continue;
        }
        let payload_path =
            output_dir.join(format!("nuis.domain.{}.payload.toml", unit.domain_family));
        let payload_source = render_domain_build_unit_payload(unit)?;
        fs::write(&payload_path, payload_source)
            .map_err(|error| format!("failed to write `{}`: {error}", payload_path.display()))?;
        let payload_blob_path =
            output_dir.join(format!("nuis.domain.{}.payload.bin", unit.domain_family));
        let payload_blob = encode_domain_build_unit_payload_blob(unit, &payload_path)?;
        fs::write(&payload_blob_path, &payload_blob).map_err(|error| {
            format!("failed to write `{}`: {error}", payload_blob_path.display())
        })?;
        let bridge_stub_path = output_dir.join(format!(
            "nuis.domain.{}.bridge.stub.txt",
            unit.domain_family
        ));
        let bridge_stub = render_domain_build_unit_host_bridge_stub(unit);
        fs::write(&bridge_stub_path, &bridge_stub).map_err(|error| {
            format!("failed to write `{}`: {error}", bridge_stub_path.display())
        })?;
        let ir_sidecar_path = if unit.domain_family == "shader"
            || unit.domain_family == "kernel"
            || unit.domain_family == "network"
        {
            let path = output_dir.join(format!(
                "nuis.domain.{}.lowering.ir.txt",
                unit.domain_family
            ));
            let sidecar = match unit.domain_family.as_str() {
                "shader" => render_domain_build_unit_shader_ir_sidecar(unit),
                "kernel" => render_domain_build_unit_kernel_ir_sidecar(unit),
                "network" => render_domain_build_unit_network_ir_sidecar(unit),
                _ => unreachable!(),
            };
            fs::write(&path, sidecar)
                .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
            Some(path)
        } else {
            None
        };
        let path = output_dir.join(format!("nuis.domain.{}.artifact.toml", unit.domain_family));
        unit.artifact_payload_path = Some(payload_path.display().to_string());
        unit.artifact_bridge_stub_path = Some(bridge_stub_path.display().to_string());
        unit.artifact_ir_sidecar_path = ir_sidecar_path
            .as_ref()
            .map(|path| path.display().to_string());
        unit.artifact_bridge_stub_inline = Some(bridge_stub.clone());
        unit.artifact_payload_blob_path = Some(payload_blob_path.display().to_string());
        unit.artifact_payload_blob_bytes = Some(payload_blob.len());
        unit.artifact_payload_format = Some("ndpb-v2".to_owned());
        unit.artifact_payload_blob_inline = Some(hex_encode_bytes(&payload_blob));
        let source = render_domain_build_unit_stub(unit);
        fs::write(&path, &source)
            .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
        unit.artifact_stub_path = Some(path.display().to_string());
        unit.artifact_stub_inline = Some(source);
        artifacts.push((format!("domain_stub_{}", unit.domain_family), path));
        artifacts.push((
            format!("domain_payload_{}", unit.domain_family),
            payload_path,
        ));
        artifacts.push((
            format!("domain_payload_blob_{}", unit.domain_family),
            payload_blob_path,
        ));
        artifacts.push((
            format!("domain_bridge_stub_{}", unit.domain_family),
            bridge_stub_path,
        ));
        if let Some(ir_sidecar_path) = ir_sidecar_path {
            artifacts.push((
                format!("domain_ir_sidecar_{}", unit.domain_family),
                ir_sidecar_path,
            ));
        }
    }
    Ok(artifacts)
}

fn write_domain_bridge_registry(
    output_dir: &Path,
    units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<PathBuf>, String> {
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    if hetero_units.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join("nuis.bridge.registry.toml");
    let source = render_domain_bridge_registry(&hetero_units);
    fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

fn write_domain_lowering_plan_index(
    output_dir: &Path,
    units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<PathBuf>, String> {
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    if hetero_units.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join("nuis.lowering.plan-index.toml");
    let source = render_domain_lowering_plan_index(&hetero_units);
    fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

fn write_host_bridge_plan_index(
    output_dir: &Path,
    units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<PathBuf>, String> {
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    if hetero_units.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join("nuis.host-bridge.plan-index.toml");
    let source = render_host_bridge_plan_index(&hetero_units);
    fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

fn resolve_domain_build_unit_target(
    domain_family: &str,
    abi: Option<&str>,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    String,
> {
    let Some(abi) = abi else {
        return Ok((None, None, None, None, None, None));
    };
    match domain_family {
        "cpu" => {
            let target = resolve_cpu_build_target_from_abi(Path::new("nustar-packages"), abi)?;
            Ok((
                Some(target.machine_arch),
                Some(target.machine_os),
                Some("llvm".to_owned()),
                None,
                None,
                Some("llvm".to_owned()),
            ))
        }
        "shader" | "kernel" | "network" => {
            let manifest = crate::registry::load_manifest_for_domain(
                Path::new("nustar-packages"),
                domain_family,
            )?;
            let target = crate::registry::registered_abi_target(&manifest, abi)?;
            let selected_lowering_target =
                crate::project::selected_lowering_target_for_registered_abi_target(
                    domain_family,
                    &target,
                    &manifest.lowering_targets,
                );
            let backend_family =
                crate::project::backend_family_for_registered_abi_target(domain_family, &target);
            Ok((
                Some(target.machine_arch),
                Some(target.machine_os),
                backend_family,
                target.vendor,
                target.device_class,
                selected_lowering_target,
            ))
        }
        _ => Ok((None, None, None, None, None, None)),
    }
}

fn build_nuis_envelope(
    execution_contracts: &[BuildManifestExecutionContract],
    packaging_mode: &str,
) -> NuisExecutableEnvelope {
    let domains = execution_contracts
        .iter()
        .map(|item| NuisEnvelopeDomainSummary {
            domain_family: item.domain_family.clone(),
            contract_family: item.execution.contract_family.clone(),
            function_kind: item.execution.function_kind.clone(),
            graph_kind: item.execution.graph_kind.clone(),
            default_time_mode: item.execution.default_time_mode.clone(),
        })
        .collect::<Vec<_>>();
    build_nuis_envelope_from_domain_summaries(&domains, packaging_mode)
}

fn detect_vcs_info(input_path: &str, output_dir: &str) -> VcsInfo {
    let candidates = [
        PathBuf::from(input_path),
        PathBuf::from(output_dir),
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    ];
    for candidate in candidates {
        if let Some(root) = git_toplevel(&candidate) {
            let head = git_head(&root).unwrap_or_else(|| "unknown".to_owned());
            let dirty = git_is_dirty(&root).unwrap_or(false);
            return VcsInfo::Git { root, head, dirty };
        }
    }
    VcsInfo::None
}

fn git_toplevel(candidate: &Path) -> Option<String> {
    let directory = if candidate.is_dir() {
        candidate.to_path_buf()
    } else {
        candidate
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    let output = Command::new("git")
        .arg("-C")
        .arg(&directory)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if root.is_empty() {
        None
    } else {
        Some(root)
    }
}

fn git_head(root: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let head = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if head.is_empty() {
        None
    } else {
        Some(head)
    }
}

fn git_is_dirty(root: &str) -> Option<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain", "--untracked-files=normal"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let dirty = !String::from_utf8_lossy(&output.stdout).trim().is_empty();
    Some(dirty)
}

pub fn verify_build_manifest(path: &Path) -> Result<BuildManifestVerifyReport, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let core = verify_manifest_core(&source, path)?;
    let fields = verify_manifest_fields(&source, path)?;

    let domain_unit_report = verify_domain_build_units(&source, path, &core)?;
    let heterogeneous_domain_count = domain_unit_report.heterogeneous_domain_count;
    let domain_build_units = &domain_unit_report.domain_build_units;
    let artifacts_checked = verify_manifest_artifacts(
        &source,
        path,
        &core,
        domain_build_units,
        fields.bridge_registry_inline.as_deref(),
        fields.host_bridge_plan_index_inline.as_deref(),
        fields.lowering_plan_index_inline.as_deref(),
    )?;

    let domain_payload_report = verify_domain_payload_blobs(path, domain_build_units)?;

    let domain_index_report = verify_domain_index_artifacts(
        path,
        fields.bridge_registry_path.as_deref(),
        fields.bridge_registry_schema.as_deref(),
        fields.bridge_registry_units,
        fields.bridge_registry_inline.as_deref(),
        fields.host_bridge_plan_index_path.as_deref(),
        fields.host_bridge_plan_index_schema.as_deref(),
        fields.host_bridge_plan_units,
        fields.host_bridge_plan_index_inline.as_deref(),
        fields.lowering_plan_index_path.as_deref(),
        fields.lowering_plan_index_schema.as_deref(),
        fields.lowering_plan_units,
        fields.lowering_plan_index_inline.as_deref(),
        heterogeneous_domain_count,
        domain_build_units,
    )?;

    let project_metadata_report = verify_project_metadata_artifacts(
        path,
        &core.input,
        &core.output_dir,
        fields.doc_index_path.as_deref(),
        fields.doc_index_module_count,
        fields.doc_index_documented_item_count,
        fields.project_plan_index.as_deref(),
        fields.project_plan_summary.as_deref(),
        fields.project_docs_index.as_deref(),
        fields.project_docs_module_count,
        fields.project_docs_documented_module_count,
        fields.project_docs_documented_item_count,
        fields.project_imports_index.as_deref(),
        fields.project_imports_library_count,
        fields.project_imports_visible_library_count,
        fields.project_imports_visible_module_count,
        fields.project_imports_documented_visible_module_count,
        fields.project_imports_documented_visible_item_count,
        fields.project_galaxy_index.as_deref(),
        fields.project_galaxy_count,
        fields.project_documented_galaxy_count,
        fields.project_documented_galaxy_library_module_count,
        fields.project_documented_galaxy_item_count,
        fields.project_packet_index.as_deref(),
    )?;
    Ok(build_manifest_verify_report(
        core,
        fields,
        domain_unit_report,
        domain_payload_report,
        domain_index_report,
        project_metadata_report,
        artifacts_checked,
    ))
}

pub fn verify_nuis_compiled_artifact(
    path: &Path,
) -> Result<NuisCompiledArtifactVerifyReport, String> {
    verify_nuis_compiled_artifact_impl(path)
}

fn parse_domain_build_unit_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<BuildManifestDomainBuildUnit>, String> {
    shared_parse_domain_build_unit_blocks(source, path).map_err(|error| error.to_string())
}

fn requires_window_bundle(yir: &YirModule) -> bool {
    yir.nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "window")
}

fn host_machine_arch() -> &'static str {
    match canonical_machine_arch(std::env::consts::ARCH) {
        "aarch64" => "arm64",
        other => other,
    }
}

fn host_machine_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    }
}

fn object_format_for_os(os: &str) -> &'static str {
    match os {
        "darwin" => "mach-o",
        "linux" => "elf",
        "windows" => "coff",
        _ => "unknown",
    }
}

fn calling_abi_for_machine(machine_arch: &str, machine_os: &str) -> &'static str {
    match (canonical_machine_arch(machine_arch), machine_os) {
        ("arm64", "darwin") => "aapcs64-darwin",
        ("arm64", _) => "aapcs64",
        ("x86_64", "windows") => "win64",
        ("x86_64", _) => "sysv64",
        _ => "unknown",
    }
}

fn host_object_format() -> &'static str {
    object_format_for_os(host_machine_os())
}

fn host_calling_abi() -> &'static str {
    calling_abi_for_machine(host_machine_arch(), host_machine_os())
}

fn clang_target_triple(machine_arch: &str, machine_os: &str) -> String {
    match (canonical_machine_arch(machine_arch), machine_os) {
        ("arm64", "darwin") => "aarch64-apple-darwin".to_owned(),
        ("arm64", "linux") => "aarch64-unknown-linux-gnu".to_owned(),
        ("x86_64", "darwin") => "x86_64-apple-darwin".to_owned(),
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_owned(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_owned(),
        _ => format!("{machine_arch}-unknown-{machine_os}"),
    }
}

fn build_window_bundle(
    yir_path: &Path,
    output_dir: &Path,
    exe_path: &Path,
    cpu_target: &CpuBuildTarget,
) -> Result<(String, String), String> {
    if cpu_target.cross_compile {
        return Err(format!(
            "window AOT bundle packaging does not support cross-compiling yet; requested `{}` -> {}",
            cpu_target.abi, cpu_target.clang_target
        ));
    }
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("yir-pack-aot")
        .arg("--")
        .arg(yir_path)
        .arg(output_dir)
        .arg("4")
        .output()
        .map_err(|error| format!("failed to invoke cargo for yir-pack-aot: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "yir-pack-aot failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok((
        exe_path.display().to_string(),
        "window-aot-bundle".to_owned(),
    ))
}

fn compile_native_binary(
    ll_path: &Path,
    shim_path: &Path,
    exe_path: &Path,
    cpu_target: &CpuBuildTarget,
) -> Result<(), String> {
    let output = Command::new("/usr/bin/clang")
        .arg("-target")
        .arg(&cpu_target.clang_target)
        .arg(ll_path)
        .arg(shim_path)
        .arg("-O2")
        .arg("-o")
        .arg(exe_path)
        .output()
        .map_err(|error| format!("failed to invoke clang: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn ast_uses_network_lifecycle_surface(ast: &AstModule) -> bool {
    ast.domain == "network"
        || ast
            .externs
            .iter()
            .any(|function| function.name.starts_with("host_network_"))
}

fn ast_uses_hetero_lifecycle_surface(ast: &AstModule) -> bool {
    ast.domain == "shader"
        || ast.domain == "kernel"
        || ast.externs.iter().any(|function| {
            function.name.starts_with("host_shader_") || function.name.starts_with("host_kernel_")
        })
}

fn ast_hetero_lifecycle_surface_slots(ast: &AstModule) -> usize {
    let mut slots = 0usize;
    if ast.domain == "shader" || ast.domain == "kernel" {
        slots += 1;
    }
    slots
        + ast
            .externs
            .iter()
            .filter(|function| {
                function.name.starts_with("host_shader_")
                    || function.name.starts_with("host_kernel_")
            })
            .count()
}

fn c_shim_source(ast: &AstModule) -> String {
    let mut out = String::new();
    let network_lifecycle_enabled = if ast_uses_network_lifecycle_surface(ast) {
        "1"
    } else {
        "0"
    };
    let hetero_lifecycle_enabled = if ast_uses_hetero_lifecycle_surface(ast) {
        "1"
    } else {
        "0"
    };
    let hetero_lifecycle_surface_slots = ast_hetero_lifecycle_surface_slots(ast);
    out.push_str(
        r#"#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>
#include <fcntl.h>
#include <unistd.h>
#include <time.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <dirent.h>
#include <signal.h>
#include <sys/wait.h>

extern int64_t nuis_yir_entry(void);

static int nuis_argc = 0;
static char** nuis_argv = NULL;
static char* nuis_host_text_slots[4096];
static size_t nuis_host_text_slot_lens[4096];
static uint64_t nuis_host_text_slot_hashes[4096];
static int64_t nuis_host_text_intern_table[8192];
static int64_t nuis_host_text_len = 0;
static DIR* nuis_host_dir_slots[256];
static int64_t nuis_host_dir_entry_counts[256];
static int64_t nuis_host_dir_len = 0;
static int nuis_host_network_fds[256];
static int64_t nuis_host_network_fd_kinds[256];
static int64_t nuis_host_network_fd_len = 0;
static pid_t nuis_host_command_pids[256];
static int64_t nuis_host_command_status_slots[256];
static int nuis_host_command_done[256];
static int nuis_host_command_timed_out[256];
static int64_t nuis_host_command_deadline_ns[256];
static int64_t nuis_host_command_len = 0;
static pid_t nuis_host_subprocess_pids[256];
static int64_t nuis_host_subprocess_status_slots[256];
static int nuis_host_subprocess_done[256];
static int nuis_host_subprocess_timed_out[256];
static int64_t nuis_host_subprocess_deadline_ns[256];
static int64_t nuis_host_subprocess_len = 0;
"#,
    );
    out.push_str(&format!(
        "static int64_t nuis_lifecycle_network_enabled = {network_lifecycle_enabled};\n"
    ));
    out.push_str(&format!(
        "static int64_t nuis_lifecycle_hetero_enabled = {hetero_lifecycle_enabled};\n"
    ));
    out.push_str(&format!(
        "static int64_t nuis_lifecycle_hetero_surface_slots = {hetero_lifecycle_surface_slots};\n"
    ));
    out.push_str(
        r#"

typedef struct {
    int64_t phase;
    int64_t tick_count;
    int64_t task_poll_count;
    int64_t summary_flush_count;
    int64_t network_bridge_progress_count;
    int64_t hetero_submission_progress_count;
    int64_t last_status;
    int64_t yalivia_rpc_enabled;
} NuisLifecycleState;

static NuisLifecycleState nuis_lifecycle_state = {0, 0, 0, 0, 0, 0, 0, 1};

static void nuis_lifecycle_state_reset(void) {
    nuis_lifecycle_state.phase = 0;
    nuis_lifecycle_state.tick_count = 0;
    nuis_lifecycle_state.task_poll_count = 0;
    nuis_lifecycle_state.summary_flush_count = 0;
    nuis_lifecycle_state.network_bridge_progress_count = 0;
    nuis_lifecycle_state.hetero_submission_progress_count = 0;
    nuis_lifecycle_state.last_status = 0;
    nuis_lifecycle_state.yalivia_rpc_enabled = 1;
}

static int64_t nuis_lifecycle_on_bridge_bind_v1(void) {
    return 0;
}

static int64_t nuis_lifecycle_on_scheduler_tick_v1(int64_t tick) {
    return tick;
}

static int64_t nuis_lifecycle_on_task_poll_v1(void) {
    nuis_lifecycle_state.task_poll_count += 1;
    return nuis_lifecycle_state.task_poll_count;
}

static int64_t nuis_lifecycle_on_result_commit_v1(int64_t status) {
    nuis_lifecycle_state.last_status = status;
    return status;
}

static int64_t nuis_lifecycle_on_summary_flush_v1(void) {
    nuis_lifecycle_state.summary_flush_count += 1;
    return nuis_lifecycle_state.summary_flush_count;
}

static int64_t nuis_lifecycle_sample_network_bridge_progress_v1(void) {
    return nuis_host_network_fd_len;
}

static int64_t nuis_lifecycle_on_network_bridge_progress_v1(void) {
    if (nuis_lifecycle_network_enabled == 0) return 0;
    int64_t observed = nuis_lifecycle_sample_network_bridge_progress_v1();
    if (observed > nuis_lifecycle_state.network_bridge_progress_count) {
        nuis_lifecycle_state.network_bridge_progress_count = observed;
    } else if (observed > 0) {
        nuis_lifecycle_state.network_bridge_progress_count += 1;
    }
    return nuis_lifecycle_state.network_bridge_progress_count;
}

static int64_t nuis_lifecycle_sample_hetero_submission_progress_v1(void) {
    return nuis_lifecycle_hetero_surface_slots;
}

static int64_t nuis_lifecycle_on_hetero_submission_progress_v1(void) {
    if (nuis_lifecycle_hetero_enabled == 0) return 0;
    int64_t observed = nuis_lifecycle_sample_hetero_submission_progress_v1();
    if (observed > nuis_lifecycle_state.hetero_submission_progress_count) {
        nuis_lifecycle_state.hetero_submission_progress_count = observed;
    } else if (observed > 0) {
        nuis_lifecycle_state.hetero_submission_progress_count += 1;
    }
    return nuis_lifecycle_state.hetero_submission_progress_count;
}

static int64_t nuis_lifecycle_on_managed_rpc_v1(void) {
    return nuis_lifecycle_state.yalivia_rpc_enabled;
}

static int64_t nuis_lifecycle_on_shutdown_prepare_v1(int64_t status) {
    nuis_lifecycle_state.last_status = status;
    return status;
}

static int64_t nuis_lifecycle_bootstrap_entry_v1(void) {
    nuis_lifecycle_state_reset();
    nuis_lifecycle_state.phase = 1;
    (void)nuis_lifecycle_on_bridge_bind_v1();
    (void)nuis_lifecycle_on_managed_rpc_v1();
    return 0;
}

static int64_t nuis_lifecycle_tick_once_v1(void) {
    if (nuis_lifecycle_state.phase == 0) return 0;
    if (nuis_lifecycle_state.phase == 3) return nuis_lifecycle_state.last_status;
    nuis_lifecycle_state.phase = 2;
    nuis_lifecycle_state.tick_count += 1;
    (void)nuis_lifecycle_on_scheduler_tick_v1(nuis_lifecycle_state.tick_count);
    (void)nuis_lifecycle_on_task_poll_v1();
    (void)nuis_lifecycle_on_network_bridge_progress_v1();
    (void)nuis_lifecycle_on_hetero_submission_progress_v1();
    return nuis_lifecycle_state.tick_count;
}

static int64_t nuis_lifecycle_shutdown_v1(int64_t status) {
    (void)nuis_lifecycle_on_result_commit_v1(status);
    (void)nuis_lifecycle_on_summary_flush_v1();
    (void)nuis_lifecycle_on_shutdown_prepare_v1(status);
    nuis_lifecycle_state.phase = 3;
    nuis_lifecycle_state.last_status = status;
    return status;
}

static int64_t nuis_lifecycle_yalivia_rpc_hook_v1(void) {
    return nuis_lifecycle_state.yalivia_rpc_enabled;
}

static uint64_t nuis_host_text_hash_bytes(const char* text, size_t len) {
    uint64_t hash = 1469598103934665603ULL;
    for (size_t index = 0; index < len; ++index) {
        hash ^= (unsigned char)text[index];
        hash *= 1099511628211ULL;
    }
    return hash;
}

static int64_t nuis_host_text_find_interned(const char* text, size_t len, uint64_t hash) {
    if (text == NULL) return 0;
    size_t mask = (sizeof(nuis_host_text_intern_table) / sizeof(nuis_host_text_intern_table[0])) - 1;
    size_t slot = (size_t)hash & mask;
    for (size_t probe = 0; probe <= mask; ++probe) {
        int64_t handle = nuis_host_text_intern_table[slot];
        if (handle == 0) return 0;
        if (handle <= nuis_host_text_len && nuis_host_text_slots[handle - 1] != NULL) {
            if (nuis_host_text_slot_hashes[handle - 1] == hash
                && nuis_host_text_slot_lens[handle - 1] == len
                && memcmp(nuis_host_text_slots[handle - 1], text, len) == 0) {
                return handle;
            }
        }
        slot = (slot + 1) & mask;
    }
    return 0;
}

static void nuis_host_text_intern_insert(int64_t handle, uint64_t hash) {
    if (handle <= 0) return;
    size_t mask = (sizeof(nuis_host_text_intern_table) / sizeof(nuis_host_text_intern_table[0])) - 1;
    size_t slot = (size_t)hash & mask;
    for (size_t probe = 0; probe <= mask; ++probe) {
        if (nuis_host_text_intern_table[slot] == 0) {
            nuis_host_text_intern_table[slot] = handle;
            return;
        }
        slot = (slot + 1) & mask;
    }
}

static int64_t nuis_host_text_register_sized(const char* text, size_t len) {
    if (text == NULL) return 0;
    if (nuis_host_text_len >= 4096) return 0;
    uint64_t hash = nuis_host_text_hash_bytes(text, len);
    int64_t interned = nuis_host_text_find_interned(text, len, hash);
    if (interned != 0) return interned;
    size_t size = len + 1;
    char* copy = (char*)malloc(size);
    if (copy == NULL) return 0;
    memcpy(copy, text, size);
    nuis_host_text_slots[nuis_host_text_len] = copy;
    nuis_host_text_slot_lens[nuis_host_text_len] = len;
    nuis_host_text_slot_hashes[nuis_host_text_len] = hash;
    nuis_host_text_len += 1;
    nuis_host_text_intern_insert(nuis_host_text_len, hash);
    return nuis_host_text_len;
}

static int64_t nuis_host_text_register(const char* text) {
    if (text == NULL) return 0;
    return nuis_host_text_register_sized(text, strlen(text));
}

static int64_t nuis_host_text_register_owned_sized(char* text, size_t len) {
    if (text == NULL) return 0;
    uint64_t hash = nuis_host_text_hash_bytes(text, len);
    int64_t interned = nuis_host_text_find_interned(text, len, hash);
    if (interned != 0) {
        free(text);
        return interned;
    }
    if (nuis_host_text_len >= 4096) {
        free(text);
        return 0;
    }
    nuis_host_text_slots[nuis_host_text_len] = text;
    nuis_host_text_slot_lens[nuis_host_text_len] = len;
    nuis_host_text_slot_hashes[nuis_host_text_len] = hash;
    nuis_host_text_len += 1;
    nuis_host_text_intern_insert(nuis_host_text_len, hash);
    return nuis_host_text_len;
}

static int64_t nuis_host_text_register_owned(char* text) {
    if (text == NULL) return 0;
    return nuis_host_text_register_owned_sized(text, strlen(text));
}

int64_t nuis_host_text_lift(const char* text) {
    return nuis_host_text_register(text);
}

static int64_t nuis_host_text_handle(int64_t text_handle) {
    return text_handle;
}

static const char* nuis_host_text_lookup(int64_t handle) {
    static char fallback[64];
    if (handle > 0 && handle <= nuis_host_text_len && nuis_host_text_slots[handle - 1] != NULL) {
        return nuis_host_text_slots[handle - 1];
    }
    if (handle == 0) return "";
    snprintf(fallback, sizeof(fallback), "%lld", (long long)handle);
    return fallback;
}

const char* nuis_host_text_ptr(int64_t handle) {
    return nuis_host_text_lookup(handle);
}

static size_t nuis_host_text_lookup_len(int64_t handle) {
    if (handle > 0 && handle <= nuis_host_text_len && nuis_host_text_slots[handle - 1] != NULL) {
        return nuis_host_text_slot_lens[handle - 1];
    }
    if (handle == 0) return 0;
    return strlen(nuis_host_text_lookup(handle));
}

static int64_t nuis_host_argv_count(void) {
    return (int64_t)nuis_argc;
}

static int64_t nuis_host_argv_at(int64_t index) {
    if (index < 0 || index >= nuis_argc || nuis_argv == NULL) return 0;
    return nuis_host_text_register(nuis_argv[index]);
}

static int64_t nuis_host_env_has(int64_t key_handle) {
    const char* key = nuis_host_text_lookup(key_handle);
    const char* value = getenv(key);
    return value != NULL ? 1 : 0;
}

static int64_t nuis_host_env_get(int64_t key_handle) {
    const char* key = nuis_host_text_lookup(key_handle);
    const char* value = getenv(key);
    if (value == NULL) return 0;
    return nuis_host_text_register(value);
}

static int64_t nuis_host_text_len_value(int64_t handle) {
    return (int64_t)nuis_host_text_lookup_len(handle);
}

static int64_t nuis_host_text_concat(int64_t lhs_handle, int64_t rhs_handle) {
    const char* lhs = nuis_host_text_lookup(lhs_handle);
    const char* rhs = nuis_host_text_lookup(rhs_handle);
    size_t lhs_len = lhs != NULL ? nuis_host_text_lookup_len(lhs_handle) : 0;
    size_t rhs_len = rhs != NULL ? nuis_host_text_lookup_len(rhs_handle) : 0;
    size_t total = lhs_len + rhs_len + 1;
    char* buffer = (char*)malloc(total);
    if (buffer == NULL) return 0;
    if (lhs_len > 0) {
        memcpy(buffer, lhs, lhs_len);
    }
    if (rhs_len > 0) {
        memcpy(buffer + lhs_len, rhs, rhs_len);
    }
    buffer[lhs_len + rhs_len] = '\0';
    return nuis_host_text_register_owned_sized(buffer, lhs_len + rhs_len);
}

static int64_t nuis_host_serialize_text_into(int64_t text_handle, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    const char* text = nuis_host_text_lookup(text_handle);
    if (text == NULL) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    size_t len = nuis_host_text_lookup_len(text_handle);
    for (size_t index = 0; index < len; ++index) {
        buffer[offset + (int64_t)index] = (unsigned char)text[index];
    }
    return (int64_t)len;
}

static int64_t nuis_host_serialize_i64_into(int64_t value, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    char text[64];
    int written = snprintf(text, sizeof(text), "%lld", (long long)value);
    if (written < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int index = 0; index < written; ++index) {
        buffer[offset + index] = (unsigned char)text[index];
    }
    return (int64_t)written;
}

static int64_t nuis_host_serialize_bool_into(int64_t value, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    const char* text = value != 0 ? "true" : "false";
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    size_t len = strlen(text);
    for (size_t index = 0; index < len; ++index) {
        buffer[offset + (int64_t)index] = (unsigned char)text[index];
    }
    return (int64_t)len;
}

static int64_t nuis_host_serialize_byte_into(int64_t value, int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    if (value < 0 || value > 255) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    buffer[offset] = value;
    return 1;
}

static int64_t nuis_host_deserialize_i64_from(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len <= 0) return 0;
    if (len > 63) len = 63;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    char text[64];
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        text[index] = (char)value;
    }
    text[len] = '\0';
    char* end = NULL;
    long long parsed = strtoll(text, &end, 10);
    if (end == text) return 0;
    return (int64_t)parsed;
}

static int64_t nuis_host_deserialize_bool_from(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len <= 0) return 0;
    if (len > 5) len = 5;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    char text[6];
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        text[index] = (char)value;
    }
    text[len] = '\0';
    if (strcmp(text, "true") == 0 || strcmp(text, "1") == 0) return 1;
    if (strcmp(text, "false") == 0 || strcmp(text, "0") == 0) return 0;
    return 0;
}

static int64_t nuis_host_deserialize_byte_from(int64_t buffer_handle, int64_t offset) {
    if (buffer_handle == 0 || offset < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t value = buffer[offset];
    if (value < 0 || value > 255) return 0;
    return value;
}

static int64_t nuis_host_deserialize_text_from(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    char* text = (char*)malloc((size_t)len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)len);
}

static int64_t nuis_host_parse_header_line(int64_t buffer_handle, int64_t offset, int64_t len, int64_t expected_name_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t trimmed_len = len;
    if (trimmed_len > 0) {
        int64_t end = offset + trimmed_len - 1;
        int64_t last = buffer[end];
        if (last == 10) {
            if (trimmed_len >= 2 && buffer[end - 1] == 13) {
                trimmed_len -= 2;
            } else {
                trimmed_len -= 1;
            }
        } else if (last == 13) {
            trimmed_len -= 1;
        }
    }
    if (trimmed_len <= 0) return 0;
    int64_t colon = -1;
    for (int64_t index = 0; index < trimmed_len; ++index) {
        if (buffer[offset + index] == 58) {
            colon = offset + index;
            break;
        }
    }
    if (colon < offset) return 0;
    int64_t name_len = colon - offset;
    const char* expected_name = nuis_host_text_lookup(expected_name_handle);
    if (expected_name == NULL) return 0;
    size_t expected_len = strlen(expected_name);
    if ((int64_t)expected_len != name_len) return 0;
    for (int64_t index = 0; index < name_len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)expected_name[index]) return 0;
    }
    int64_t value_offset = colon + 1;
    int64_t line_end = offset + trimmed_len;
    while (value_offset < line_end) {
        int64_t value = buffer[value_offset];
        if (value != 32 && value != 9) break;
        value_offset += 1;
    }
    int64_t value_len = line_end - value_offset;
    char* text = (char*)malloc((size_t)value_len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < value_len; ++index) {
        int64_t value = buffer[value_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[value_len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)value_len);
}

static int64_t nuis_host_parse_header_line_named(
    int64_t buffer_handle,
    int64_t offset,
    int64_t len,
    const char* expected_name,
    size_t expected_len
) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || expected_name == NULL) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t trimmed_len = len;
    if (trimmed_len > 0) {
        int64_t end = offset + trimmed_len - 1;
        int64_t last = buffer[end];
        if (last == 10) {
            if (trimmed_len >= 2 && buffer[end - 1] == 13) {
                trimmed_len -= 2;
            } else {
                trimmed_len -= 1;
            }
        } else if (last == 13) {
            trimmed_len -= 1;
        }
    }
    if (trimmed_len <= 0) return 0;
    int64_t colon = -1;
    for (int64_t index = 0; index < trimmed_len; ++index) {
        if (buffer[offset + index] == 58) {
            colon = offset + index;
            break;
        }
    }
    if (colon < offset) return 0;
    int64_t name_len = colon - offset;
    if ((int64_t)expected_len != name_len) return 0;
    for (int64_t index = 0; index < name_len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)expected_name[index]) return 0;
    }
    int64_t value_offset = colon + 1;
    int64_t line_end = offset + trimmed_len;
    while (value_offset < line_end) {
        int64_t value = buffer[value_offset];
        if (value != 32 && value != 9) break;
        value_offset += 1;
    }
    int64_t value_len = line_end - value_offset;
    char* text = (char*)malloc((size_t)value_len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < value_len; ++index) {
        int64_t value = buffer[value_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[value_len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)value_len);
}

static int64_t nuis_host_find_header_value(int64_t buffer_handle, int64_t offset, int64_t len, int64_t expected_name_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t cursor = offset;
    int64_t limit = offset + len;
    while (cursor < limit) {
        int64_t line_end = cursor;
        while (line_end < limit) {
            int64_t value = ((int64_t*)(intptr_t)buffer_handle)[line_end];
            if (value == 13 || value == 10) break;
            line_end += 1;
        }
        int64_t line_len = line_end - cursor;
        if (line_len == 0) return 0;
        int64_t parsed = nuis_host_parse_header_line(
            buffer_handle,
            cursor,
            line_end < limit ? (line_end - cursor + 1) : line_len,
            expected_name_handle
        );
        if (parsed != 0) return parsed;
        if (line_end >= limit) break;
        if (((int64_t*)(intptr_t)buffer_handle)[line_end] == 13
            && line_end + 1 < limit
            && ((int64_t*)(intptr_t)buffer_handle)[line_end + 1] == 10) {
            cursor = line_end + 2;
        } else {
            cursor = line_end + 1;
        }
    }
    return 0;
}

static int64_t nuis_host_find_header_value_named(
    int64_t buffer_handle,
    int64_t offset,
    int64_t len,
    const char* expected_name,
    size_t expected_len
) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || expected_name == NULL) return 0;
    int64_t cursor = offset;
    int64_t limit = offset + len;
    while (cursor < limit) {
        int64_t line_end = cursor;
        while (line_end < limit) {
            int64_t value = ((int64_t*)(intptr_t)buffer_handle)[line_end];
            if (value == 13 || value == 10) break;
            line_end += 1;
        }
        int64_t line_len = line_end - cursor;
        if (line_len == 0) return 0;
        int64_t parsed = nuis_host_parse_header_line_named(
            buffer_handle,
            cursor,
            line_end < limit ? (line_end - cursor + 1) : line_len,
            expected_name,
            expected_len
        );
        if (parsed != 0) return parsed;
        if (line_end >= limit) break;
        if (((int64_t*)(intptr_t)buffer_handle)[line_end] == 13
            && line_end + 1 < limit
            && ((int64_t*)(intptr_t)buffer_handle)[line_end + 1] == 10) {
            cursor = line_end + 2;
        } else {
            cursor = line_end + 1;
        }
    }
    return 0;
}

static int64_t nuis_host_find_status_line_reason(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t limit = offset + len;
    int64_t line_end = offset;
    while (line_end < limit) {
        int64_t value = buffer[line_end];
        if (value == 13 || value == 10) break;
        line_end += 1;
    }
    if (line_end <= offset) return 0;
    int64_t first_space = -1;
    for (int64_t index = offset; index < line_end; ++index) {
        if (buffer[index] == 32) {
            first_space = index;
            break;
        }
    }
    if (first_space < offset) return 0;
    int64_t second_space = -1;
    for (int64_t index = first_space + 1; index < line_end; ++index) {
        if (buffer[index] == 32) {
            second_space = index;
            break;
        }
    }
    if (second_space < first_space + 1) return 0;
    int64_t reason_offset = second_space + 1;
    while (reason_offset < line_end) {
        int64_t value = buffer[reason_offset];
        if (value != 32 && value != 9) break;
        reason_offset += 1;
    }
    int64_t reason_len = line_end - reason_offset;
    char* text = (char*)malloc((size_t)reason_len + 1);
    if (text == NULL) return 0;
    for (int64_t index = 0; index < reason_len; ++index) {
        int64_t value = buffer[reason_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[index] = (char)value;
    }
    text[reason_len] = '\0';
    return nuis_host_text_register_owned_sized(text, (size_t)reason_len);
}

static int64_t nuis_host_parse_http_response_summary(int64_t buffer_handle, int64_t offset, int64_t len) {
    static const char content_type_name[] = "Content-Type";
    static const char content_length_name[] = "Content-Length";
    static const char content_type_prefix[] = " | content-type=";
    static const char content_length_prefix[] = " | content-length=";
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t limit = offset + len;
    int64_t line_end = offset;
    while (line_end < limit) {
        int64_t value = buffer[line_end];
        if (value == 13 || value == 10) break;
        line_end += 1;
    }
    if (line_end <= offset) return 0;
    int64_t first_space = -1;
    for (int64_t index = offset; index < line_end; ++index) {
        if (buffer[index] == 32) {
            first_space = index;
            break;
        }
    }
    if (first_space < offset) return 0;
    int64_t second_space = -1;
    for (int64_t index = first_space + 1; index < line_end; ++index) {
        if (buffer[index] == 32) {
            second_space = index;
            break;
        }
    }
    if (second_space < first_space + 1) return 0;
    int64_t status_offset = first_space + 1;
    int64_t status_len = second_space - status_offset;

    int64_t reason_handle = nuis_host_find_status_line_reason(buffer_handle, offset, len);
    const char* reason = nuis_host_text_lookup(reason_handle);
    int64_t content_type_handle =
        nuis_host_find_header_value_named(
            buffer_handle,
            offset,
            len,
            content_type_name,
            sizeof(content_type_name) - 1
        );
    int64_t content_length_handle =
        nuis_host_find_header_value_named(
            buffer_handle,
            offset,
            len,
            content_length_name,
            sizeof(content_length_name) - 1
        );
    const char* content_type = nuis_host_text_lookup(content_type_handle);
    const char* content_length = nuis_host_text_lookup(content_length_handle);

    int has_reason = reason != NULL && reason[0] != '\0';
    int has_content_type = content_type != NULL && content_type[0] != '\0';
    int has_content_length = content_length != NULL && content_length[0] != '\0';
    size_t reason_len = has_reason ? nuis_host_text_lookup_len(reason_handle) : 0;
    size_t content_type_len = has_content_type ? nuis_host_text_lookup_len(content_type_handle) : 0;
    size_t content_length_len =
        has_content_length ? nuis_host_text_lookup_len(content_length_handle) : 0;
    size_t total = (size_t)status_len + 1;
    if (has_reason) total += 1 + reason_len;
    if (has_content_type) total += sizeof(content_type_prefix) - 1 + content_type_len;
    if (has_content_length) total += sizeof(content_length_prefix) - 1 + content_length_len;

    char* text = (char*)malloc(total);
    if (text == NULL) return 0;
    size_t cursor = 0;
    for (int64_t index = 0; index < status_len; ++index) {
        int64_t value = buffer[status_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[cursor++] = (char)value;
    }
    if (has_reason) {
        text[cursor++] = ' ';
        memcpy(text + cursor, reason, reason_len);
        cursor += reason_len;
    }
    if (has_content_type) {
        memcpy(text + cursor, content_type_prefix, sizeof(content_type_prefix) - 1);
        cursor += sizeof(content_type_prefix) - 1;
        memcpy(text + cursor, content_type, content_type_len);
        cursor += content_type_len;
    }
    if (has_content_length) {
        memcpy(text + cursor, content_length_prefix, sizeof(content_length_prefix) - 1);
        cursor += sizeof(content_length_prefix) - 1;
        memcpy(text + cursor, content_length, content_length_len);
        cursor += content_length_len;
    }
    text[cursor] = '\0';
    return nuis_host_text_register_owned_sized(text, cursor);
}

static int64_t nuis_host_parse_http_request_summary(int64_t buffer_handle, int64_t offset, int64_t len) {
    static const char host_name[] = "Host";
    static const char connection_name[] = "Connection";
    static const char host_prefix[] = " | host=";
    static const char connection_prefix[] = " | connection=";
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t limit = offset + len;
    int64_t line_end = offset;
    while (line_end < limit) {
        int64_t value = buffer[line_end];
        if (value == 13 || value == 10) break;
        line_end += 1;
    }
    if (line_end <= offset) return 0;
    int64_t first_space = -1;
    for (int64_t index = offset; index < line_end; ++index) {
        if (buffer[index] == 32) {
            first_space = index;
            break;
        }
    }
    if (first_space < offset) return 0;
    int64_t second_space = -1;
    for (int64_t index = first_space + 1; index < line_end; ++index) {
        if (buffer[index] == 32) {
            second_space = index;
            break;
        }
    }
    if (second_space < first_space + 1) return 0;
    int64_t method_len = first_space - offset;
    int64_t path_offset = first_space + 1;
    int64_t path_len = second_space - path_offset;

    int64_t host_handle = nuis_host_find_header_value_named(
        buffer_handle,
        offset,
        len,
        host_name,
        sizeof(host_name) - 1
    );
    int64_t connection_handle =
        nuis_host_find_header_value_named(
            buffer_handle,
            offset,
            len,
            connection_name,
            sizeof(connection_name) - 1
        );
    const char* host = nuis_host_text_lookup(host_handle);
    const char* connection = nuis_host_text_lookup(connection_handle);
    int has_host = host != NULL && host[0] != '\0';
    int has_connection = connection != NULL && connection[0] != '\0';
    size_t host_len = has_host ? nuis_host_text_lookup_len(host_handle) : 0;
    size_t connection_len = has_connection ? nuis_host_text_lookup_len(connection_handle) : 0;
    size_t total = (size_t)method_len + 1 + (size_t)path_len + 1;
    if (has_host) total += sizeof(host_prefix) - 1 + host_len;
    if (has_connection) total += sizeof(connection_prefix) - 1 + connection_len;

    char* text = (char*)malloc(total);
    if (text == NULL) return 0;
    size_t cursor = 0;
    for (int64_t index = 0; index < method_len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[cursor++] = (char)value;
    }
    text[cursor++] = ' ';
    for (int64_t index = 0; index < path_len; ++index) {
        int64_t value = buffer[path_offset + index];
        if (value < 0 || value > 255) {
            free(text);
            return 0;
        }
        text[cursor++] = (char)value;
    }
    if (has_host) {
        memcpy(text + cursor, host_prefix, sizeof(host_prefix) - 1);
        cursor += sizeof(host_prefix) - 1;
        memcpy(text + cursor, host, host_len);
        cursor += host_len;
    }
    if (has_connection) {
        memcpy(text + cursor, connection_prefix, sizeof(connection_prefix) - 1);
        cursor += sizeof(connection_prefix) - 1;
        memcpy(text + cursor, connection, connection_len);
        cursor += connection_len;
    }
    text[cursor] = '\0';
    return nuis_host_text_register_owned_sized(text, cursor);
}

static int64_t nuis_host_parse_http_roundtrip_summary(
    int64_t request_buffer_handle,
    int64_t request_offset,
    int64_t request_len,
    int64_t response_buffer_handle,
    int64_t response_offset,
    int64_t response_len
) {
    static const char arrow_separator[] = " -> ";
    int64_t request_handle =
        nuis_host_parse_http_request_summary(request_buffer_handle, request_offset, request_len);
    int64_t response_handle =
        nuis_host_parse_http_response_summary(response_buffer_handle, response_offset, response_len);
    const char* request = nuis_host_text_lookup(request_handle);
    const char* response = nuis_host_text_lookup(response_handle);
    if (request == NULL) request = "";
    if (response == NULL) response = "";
    size_t request_len_text = nuis_host_text_lookup_len(request_handle);
    size_t response_len_text = nuis_host_text_lookup_len(response_handle);
    size_t total = request_len_text + (sizeof(arrow_separator) - 1) + response_len_text + 1;
    char* text = (char*)malloc(total);
    if (text == NULL) return 0;
    memcpy(text, request, request_len_text);
    memcpy(text + request_len_text, arrow_separator, sizeof(arrow_separator) - 1);
    memcpy(
        text + request_len_text + (sizeof(arrow_separator) - 1),
        response,
        response_len_text
    );
    text[total - 1] = '\0';
    return nuis_host_text_register_owned_sized(text, total - 1);
}

static int64_t nuis_host_deserialize_text_equals(int64_t buffer_handle, int64_t offset, int64_t len, int64_t expected_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* expected = nuis_host_text_lookup(expected_handle);
    if (expected == NULL) return 0;
    size_t expected_len = strlen(expected);
    if ((int64_t)expected_len != len) return 0;
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)expected[index]) return 0;
    }
    return 1;
}

static int64_t nuis_host_deserialize_text_starts_with(int64_t buffer_handle, int64_t offset, int64_t len, int64_t prefix_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* prefix = nuis_host_text_lookup(prefix_handle);
    if (prefix == NULL) return 0;
    size_t prefix_len = strlen(prefix);
    if ((int64_t)prefix_len > len) return 0;
    for (size_t index = 0; index < prefix_len; ++index) {
        int64_t value = buffer[offset + (int64_t)index];
        if (value < 0 || value > 255) return 0;
        if ((unsigned char)value != (unsigned char)prefix[index]) return 0;
    }
    return 1;
}

static int64_t nuis_host_deserialize_text_contains(int64_t buffer_handle, int64_t offset, int64_t len, int64_t needle_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* needle = nuis_host_text_lookup(needle_handle);
    if (needle == NULL) return 0;
    size_t needle_len = strlen(needle);
    if (needle_len == 0) return 1;
    if ((int64_t)needle_len > len) return 0;
    for (int64_t start = 0; start <= len - (int64_t)needle_len; ++start) {
        int matched = 1;
        for (size_t index = 0; index < needle_len; ++index) {
            int64_t value = buffer[offset + start + (int64_t)index];
            if (value < 0 || value > 255 || (unsigned char)value != (unsigned char)needle[index]) {
                matched = 0;
                break;
            }
        }
        if (matched) return 1;
    }
    return 0;
}

static int64_t nuis_host_deserialize_text_ends_with(int64_t buffer_handle, int64_t offset, int64_t len, int64_t suffix_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* suffix = nuis_host_text_lookup(suffix_handle);
    if (suffix == NULL) return 0;
    size_t suffix_len = strlen(suffix);
    if ((int64_t)suffix_len > len) return 0;
    int64_t start = offset + len - (int64_t)suffix_len;
    for (size_t index = 0; index < suffix_len; ++index) {
        int64_t value = buffer[start + (int64_t)index];
        if (value < 0 || value > 255 || (unsigned char)value != (unsigned char)suffix[index]) {
            return 0;
        }
    }
    return 1;
}

static int64_t nuis_host_buffer_find_byte(int64_t buffer_handle, int64_t offset, int64_t len, int64_t needle) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || needle < 0 || needle > 255) return -1;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int64_t index = 0; index < len; ++index) {
        if (buffer[offset + index] == needle) {
            return offset + index;
        }
    }
    return -1;
}

static int64_t nuis_host_fill_bytes(int64_t buffer_handle, int64_t offset, int64_t len, int64_t value) {
    if (buffer_handle == 0 || offset < 0 || len < 0 || value < 0 || value > 255) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int64_t index = 0; index < len; ++index) {
        buffer[offset + index] = value;
    }
    return len;
}

static int64_t nuis_host_copy_bytes(int64_t dst_handle, int64_t dst_offset, int64_t dst_len, int64_t src_handle, int64_t src_offset, int64_t src_len) {
    if (dst_handle == 0 || src_handle == 0 || dst_offset < 0 || src_offset < 0 || dst_len < 0 || src_len < 0) return 0;
    int64_t copy_len = dst_len < src_len ? dst_len : src_len;
    int64_t* dst = (int64_t*)(intptr_t)dst_handle;
    int64_t* src = (int64_t*)(intptr_t)src_handle;
    if (copy_len <= 0) return 0;
    if (dst == src && dst_offset > src_offset && dst_offset < src_offset + copy_len) {
        for (int64_t index = copy_len; index > 0; --index) {
            int64_t value = src[src_offset + index - 1];
            if (value < 0 || value > 255) return 0;
            dst[dst_offset + index - 1] = value;
        }
    } else {
        for (int64_t index = 0; index < copy_len; ++index) {
            int64_t value = src[src_offset + index];
            if (value < 0 || value > 255) return 0;
            dst[dst_offset + index] = value;
        }
    }
    return copy_len;
}

static int64_t nuis_host_compare_bytes(int64_t lhs_handle, int64_t lhs_offset, int64_t lhs_len, int64_t rhs_handle, int64_t rhs_offset, int64_t rhs_len) {
    if (lhs_handle == 0 || rhs_handle == 0 || lhs_offset < 0 || rhs_offset < 0 || lhs_len < 0 || rhs_len < 0) return 0;
    int64_t* lhs = (int64_t*)(intptr_t)lhs_handle;
    int64_t* rhs = (int64_t*)(intptr_t)rhs_handle;
    int64_t shared_len = lhs_len < rhs_len ? lhs_len : rhs_len;
    for (int64_t index = 0; index < shared_len; ++index) {
        int64_t lhs_value = lhs[lhs_offset + index];
        int64_t rhs_value = rhs[rhs_offset + index];
        if (lhs_value < 0 || lhs_value > 255 || rhs_value < 0 || rhs_value > 255) return 0;
        if (lhs_value < rhs_value) return -1;
        if (lhs_value > rhs_value) return 1;
    }
    if (lhs_len < rhs_len) return -1;
    if (lhs_len > rhs_len) return 1;
    return 0;
}

static int64_t nuis_host_buffer_find_text(int64_t buffer_handle, int64_t offset, int64_t len, int64_t needle_handle) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return -1;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    const char* needle = nuis_host_text_lookup(needle_handle);
    if (needle == NULL) return -1;
    size_t needle_len = strlen(needle);
    if (needle_len == 0) return offset;
    if ((int64_t)needle_len > len) return -1;
    for (int64_t start = 0; start <= len - (int64_t)needle_len; ++start) {
        int matched = 1;
        for (size_t index = 0; index < needle_len; ++index) {
            int64_t value = buffer[offset + start + (int64_t)index];
            if (value < 0 || value > 255 || (unsigned char)value != (unsigned char)needle[index]) {
                matched = 0;
                break;
            }
        }
        if (matched) return offset + start;
    }
    return -1;
}

static int64_t nuis_host_buffer_find_line_end(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return -1;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (int64_t index = 0; index < len; ++index) {
        int64_t value = buffer[offset + index];
        if (value == 13 || value == 10) {
            return offset + index;
        }
    }
    return -1;
}

static int64_t nuis_host_buffer_trim_line_end(int64_t buffer_handle, int64_t offset, int64_t len) {
    if (buffer_handle == 0 || offset < 0 || len < 0) return 0;
    if (len == 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    int64_t end = offset + len - 1;
    int64_t last = buffer[end];
    if (last == 10) {
        if (len >= 2 && buffer[end - 1] == 13) {
            return len - 2;
        }
        return len - 1;
    }
    if (last == 13) {
        return len - 1;
    }
    return len;
}

static int64_t nuis_host_file_open(int64_t path_handle, int64_t flags) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    int fd = open(path, (int)flags, 0644);
    return fd >= 0 ? (int64_t)fd : 0;
}

static int64_t nuis_host_file_read(int64_t file_handle, int64_t buffer_handle, int64_t len) {
    if (file_handle < 0 || buffer_handle == 0 || len <= 0) return 0;
    char scratch[4096];
    size_t read_len = (size_t)len;
    if (read_len > sizeof(scratch)) read_len = sizeof(scratch);
    ssize_t got = read((int)file_handle, scratch, read_len);
    if (got <= 0) return 0;
    int64_t* buffer = (int64_t*)(intptr_t)buffer_handle;
    for (ssize_t i = 0; i < got; ++i) {
        buffer[i] = (unsigned char)scratch[i];
    }
    return (int64_t)got;
}

static int64_t nuis_host_file_write(int64_t file_handle, int64_t text_handle) {
    if (file_handle < 0) return 0;
    const char* text = nuis_host_text_lookup(text_handle);
    size_t len = strlen(text);
    if (len == 0) return 0;
    ssize_t wrote = write((int)file_handle, text, len);
    return wrote > 0 ? (int64_t)wrote : 0;
}

static int64_t nuis_host_file_close(int64_t file_handle) {
    if (file_handle < 0) return 0;
    return close((int)file_handle) == 0 ? 1 : 0;
}

static int64_t nuis_host_network_register_fd(int fd, int64_t kind) {
    if (fd < 0) return 0;
    if (nuis_host_network_fd_len >= 256) {
        close(fd);
        return 0;
    }
    nuis_host_network_fds[nuis_host_network_fd_len] = fd;
    nuis_host_network_fd_kinds[nuis_host_network_fd_len] = kind;
    nuis_host_network_fd_len += 1;
    return nuis_host_network_fd_len;
}

static int nuis_host_network_lookup_fd(int64_t handle) {
    if (handle <= 0 || handle > nuis_host_network_fd_len) return -1;
    return nuis_host_network_fds[handle - 1];
}

static int64_t nuis_host_network_lookup_kind(int64_t handle) {
    if (handle <= 0 || handle > nuis_host_network_fd_len) return 0;
    return nuis_host_network_fd_kinds[handle - 1];
}

static int64_t nuis_host_network_release_fd(int64_t handle, int close_fd) {
    int fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    nuis_host_network_fds[handle - 1] = -1;
    nuis_host_network_fd_kinds[handle - 1] = 0;
    if (close_fd && close(fd) != 0) return 0;
    return 1;
}

static void nuis_network_init_loopback_addr(struct sockaddr_in* addr, int64_t port) {
    memset(addr, 0, sizeof(*addr));
    addr->sin_family = AF_INET;
    addr->sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr->sin_port = htons((uint16_t)port);
}

static int nuis_network_apply_timeout_ms(int fd, int64_t timeout_ms) {
    struct timeval tv;
    if (timeout_ms < 0) return 0;
    tv.tv_sec = (time_t)(timeout_ms / 1000);
    tv.tv_usec = (suseconds_t)((timeout_ms % 1000) * 1000);
    if (setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) != 0) return 0;
    if (setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv)) != 0) return 0;
    return 1;
}

static int nuis_network_try_connect_probe(
    int64_t local_port,
    int64_t remote_port,
    int64_t connect_timeout_ms
) {
    int listener = -1;
    int client = -1;
    int accepted = -1;
    int ok = 0;
    struct sockaddr_in listener_addr;
    struct sockaddr_in client_addr;

    listener = socket(AF_INET, SOCK_STREAM, 0);
    if (listener < 0) goto done;
    {
        int yes = 1;
        setsockopt(listener, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));
    }
    nuis_network_init_loopback_addr(&listener_addr, remote_port);
    if (bind(listener, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    if (listen(listener, 1) != 0) goto done;

    client = socket(AF_INET, SOCK_STREAM, 0);
    if (client < 0) goto done;
    if (!nuis_network_apply_timeout_ms(client, connect_timeout_ms)) goto done;
    if (local_port > 0) {
        nuis_network_init_loopback_addr(&client_addr, local_port);
        if (bind(client, (struct sockaddr*)&client_addr, sizeof(client_addr)) != 0) goto done;
    }
    if (connect(client, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    accepted = accept(listener, NULL, NULL);
    if (accepted < 0) goto done;
    ok = 1;

done:
    if (accepted >= 0) close(accepted);
    if (client >= 0) close(client);
    if (listener >= 0) close(listener);
    return ok;
}

static int nuis_network_try_accept_probe(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int listener = -1;
    int client = -1;
    int accepted = -1;
    int ok = 0;
    struct sockaddr_in listener_addr;

    listener = socket(AF_INET, SOCK_STREAM, 0);
    if (listener < 0) goto done;
    {
        int yes = 1;
        setsockopt(listener, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));
    }
    if (!nuis_network_apply_timeout_ms(listener, read_timeout_ms + write_timeout_ms)) goto done;
    nuis_network_init_loopback_addr(&listener_addr, local_port);
    if (bind(listener, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    if (listen(listener, 1) != 0) goto done;

    client = socket(AF_INET, SOCK_STREAM, 0);
    if (client < 0) goto done;
    if (!nuis_network_apply_timeout_ms(client, write_timeout_ms)) goto done;
    if (connect(client, (struct sockaddr*)&listener_addr, sizeof(listener_addr)) != 0) goto done;
    accepted = accept(listener, NULL, NULL);
    if (accepted < 0) goto done;
    if (!nuis_network_apply_timeout_ms(accepted, read_timeout_ms + write_timeout_ms)) goto done;
    ok = 1;

done:
    if (accepted >= 0) close(accepted);
    if (client >= 0) close(client);
    if (listener >= 0) close(listener);
    return ok;
}

static int nuis_network_try_send_probe(int64_t stream_window, int64_t send_window) {
    int fds[2] = {-1, -1};
    int ok = 0;
    char buffer[64];
    size_t want = (size_t)send_window;
    if (want > sizeof(buffer)) want = sizeof(buffer);
    if (want == 0) want = 1;
    memset(buffer, 'n', want);
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, fds) != 0) goto done;
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    if (send(fds[0], buffer, want, 0) < 0) goto done;
    ok = 1;

done:
    if (fds[0] >= 0) close(fds[0]);
    if (fds[1] >= 0) close(fds[1]);
    return ok;
}

static int nuis_network_try_recv_probe(int64_t stream_window, int64_t recv_window) {
    int fds[2] = {-1, -1};
    int ok = 0;
    char send_buffer[64];
    char recv_buffer[64];
    size_t want = (size_t)recv_window;
    if (want > sizeof(send_buffer)) want = sizeof(send_buffer);
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    memset(send_buffer, 'y', want);
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, fds) != 0) goto done;
    if (send(fds[0], send_buffer, want, 0) < 0) goto done;
    if (recv(fds[1], recv_buffer, want, 0) < 0) goto done;
    ok = 1;

done:
    if (fds[0] >= 0) close(fds[0]);
    if (fds[1] >= 0) close(fds[1]);
    return ok;
}

static int64_t nuis_host_network_open_tcp_stream(
    int64_t remote_port,
    int64_t connect_timeout_ms
) {
    int fd = -1;
    struct sockaddr_in addr;
    if (remote_port <= 0) return 0;
    fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return 0;
    if (connect_timeout_ms >= 0) {
        if (!nuis_network_apply_timeout_ms(fd, connect_timeout_ms)) {
            close(fd);
            return 0;
        }
    }
    nuis_network_init_loopback_addr(&addr, remote_port);
    if (connect(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        close(fd);
        return 0;
    }
    return nuis_host_network_register_fd(fd, 1);
}

static int64_t nuis_host_network_open_tcp_listener(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int fd = -1;
    struct sockaddr_in addr;
    int yes = 1;
    if (local_port <= 0 || read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return 0;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));
    if (!nuis_network_apply_timeout_ms(fd, read_timeout_ms + write_timeout_ms)) {
        close(fd);
        return 0;
    }
    nuis_network_init_loopback_addr(&addr, local_port);
    if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        close(fd);
        return 0;
    }
    if (listen(fd, 1) != 0) {
        close(fd);
        return 0;
    }
    return nuis_host_network_register_fd(fd, 3);
}

static int64_t nuis_host_network_open_udp_datagram(
    int64_t local_port,
    int64_t remote_port
) {
    int fd = -1;
    struct sockaddr_in addr;
    fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0) return 0;
    if (local_port > 0) {
        nuis_network_init_loopback_addr(&addr, local_port);
        if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
            close(fd);
            return 0;
        }
    }
    if (remote_port > 0) {
        nuis_network_init_loopback_addr(&addr, remote_port);
        if (connect(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
            close(fd);
            return 0;
        }
    }
    return nuis_host_network_register_fd(fd, 2);
}

static int64_t nuis_host_network_bind_udp_datagram(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int fd = -1;
    struct sockaddr_in addr;
    if (local_port <= 0 || read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0) return 0;
    if (!nuis_network_apply_timeout_ms(fd, read_timeout_ms + write_timeout_ms)) {
        close(fd);
        return 0;
    }
    nuis_network_init_loopback_addr(&addr, local_port);
    if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        close(fd);
        return 0;
    }
    return nuis_host_network_register_fd(fd, 2);
}

static int64_t nuis_host_network_accept_owned(
    int64_t listener_handle,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    int listener_fd = -1;
    int accepted_fd = -1;
    if (listener_handle <= 0 || read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    if (nuis_host_network_lookup_kind(listener_handle) != 3) return 0;
    listener_fd = nuis_host_network_lookup_fd(listener_handle);
    if (listener_fd < 0) return 0;
    accepted_fd = accept(listener_fd, NULL, NULL);
    if (accepted_fd < 0) return 0;
    if (!nuis_network_apply_timeout_ms(accepted_fd, read_timeout_ms + write_timeout_ms)) {
        close(accepted_fd);
        return 0;
    }
    return nuis_host_network_register_fd(accepted_fd, 1);
}

static int64_t nuis_host_network_close_owned(int64_t handle) {
    return nuis_host_network_release_fd(handle, 1);
}

static int64_t nuis_host_network_send_owned(
    int64_t handle,
    int64_t stream_window,
    int64_t send_window
) {
    int fd = -1;
    int64_t kind = 0;
    ssize_t sent = 0;
    char buffer[64];
    size_t want = (size_t)send_window;
    if (handle <= 0 || stream_window <= 0 || send_window <= 0) return 0;
    fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    kind = nuis_host_network_lookup_kind(handle);
    if (want > sizeof(buffer)) want = sizeof(buffer);
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    if (kind == 1) {
        const char* request = "GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
        size_t request_len = strlen(request);
        if (want > request_len) want = request_len;
        memcpy(buffer, request, want);
    } else {
        memset(buffer, 's', want);
    }
    sent = send(fd, buffer, want, 0);
    if (sent <= 0) return 0;
    if (kind == 1) {
        shutdown(fd, SHUT_WR);
    }
    return handle + (int64_t)sent;
}

static int64_t nuis_host_network_recv_owned(
    int64_t handle,
    int64_t stream_window,
    int64_t recv_window
) {
    int fd = -1;
    ssize_t received = 0;
    char buffer[64];
    size_t want = (size_t)recv_window;
    if (handle <= 0 || stream_window <= 0 || recv_window <= 0) return 0;
    fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    if (want > sizeof(buffer)) want = sizeof(buffer);
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    received = recv(fd, buffer, want, 0);
    if (received <= 0) return 0;
    return handle + (int64_t)received;
}

static int64_t nuis_host_network_recv_http_status_owned(
    int64_t handle,
    int64_t stream_window,
    int64_t recv_window
) {
    int fd = -1;
    ssize_t received = 0;
    char buffer[128];
    size_t want = (size_t)recv_window;
    int status = 0;
    if (handle <= 0 || stream_window <= 0 || recv_window <= 0) return 0;
    fd = nuis_host_network_lookup_fd(handle);
    if (fd < 0) return 0;
    if (want > sizeof(buffer) - 1) want = sizeof(buffer) - 1;
    if ((size_t)stream_window < want) want = (size_t)stream_window;
    if (want == 0) want = 1;
    received = recv(fd, buffer, want, 0);
    if (received <= 0) return 0;
    buffer[received] = '\0';
    if (sscanf(buffer, "HTTP/%*d.%*d %d", &status) == 1 && status > 0) {
        return (int64_t)status;
    }
    return handle + (int64_t)received;
}

static int64_t nuis_host_network_connect_probe(
    int64_t local_port,
    int64_t remote_port,
    int64_t connect_timeout_ms
) {
    if (local_port <= 0 || remote_port <= 0) return 0;
    if (connect_timeout_ms < 0) return 0;
    return nuis_network_try_connect_probe(local_port, remote_port, connect_timeout_ms) ? 1 : 0;
}

static int64_t nuis_host_network_accept_probe(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    if (local_port <= 0) return 0;
    if (read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    return nuis_network_try_accept_probe(local_port, read_timeout_ms, write_timeout_ms) ? 1 : 0;
}

static int64_t nuis_host_network_close(int64_t handle) {
    if (handle <= 0) return 0;
    if (nuis_host_network_close_owned(handle)) return 1;
    return 0;
}

static int64_t nuis_host_network_send_probe(
    int64_t stream_window,
    int64_t send_window,
    int64_t remote_port
) {
    if (stream_window <= 0 || send_window <= 0 || remote_port <= 0) return 0;
    (void)remote_port;
    return nuis_network_try_send_probe(stream_window, send_window) ? 1 : 0;
}

static int64_t nuis_host_network_recv_probe(
    int64_t stream_window,
    int64_t recv_window,
    int64_t local_port
) {
    if (stream_window <= 0 || recv_window <= 0 || local_port <= 0) return 0;
    (void)local_port;
    return nuis_network_try_recv_probe(stream_window, recv_window) ? 1 : 0;
}

static int64_t nuis_host_dir_open(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    if (nuis_host_dir_len >= 256) return 0;
    DIR* dir = opendir(path);
    if (dir == NULL) return 0;
    int64_t count = 0;
    struct dirent* entry = NULL;
    while ((entry = readdir(dir)) != NULL) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) continue;
        count += 1;
    }
    rewinddir(dir);
    nuis_host_dir_slots[nuis_host_dir_len] = dir;
    nuis_host_dir_entry_counts[nuis_host_dir_len] = count;
    nuis_host_dir_len += 1;
    return nuis_host_dir_len;
}

static int64_t nuis_host_dir_entry_count(int64_t dir_handle) {
    if (dir_handle <= 0 || dir_handle > nuis_host_dir_len) return 0;
    return nuis_host_dir_entry_counts[dir_handle - 1];
}

static int64_t nuis_host_dir_close(int64_t dir_handle) {
    if (dir_handle <= 0 || dir_handle > nuis_host_dir_len) return 0;
    DIR* dir = nuis_host_dir_slots[dir_handle - 1];
    if (dir == NULL) return 0;
    nuis_host_dir_slots[dir_handle - 1] = NULL;
    return closedir(dir) == 0 ? 1 : 0;
}

static int64_t nuis_host_dir_create(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return mkdir(path, 0755) == 0 ? 1 : 0;
}

static int64_t nuis_host_dir_remove(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return rmdir(path) == 0 ? 1 : 0;
}

static int64_t nuis_host_stdin_read(int64_t buffer_handle, int64_t len) {
    (void)buffer_handle;
    if (len <= 0) return 0;
    char scratch[4096];
    size_t read_len = (size_t)len;
    if (read_len > sizeof(scratch)) read_len = sizeof(scratch);
    ssize_t got = read(STDIN_FILENO, scratch, read_len);
    return got > 0 ? (int64_t)got : 0;
}

static int64_t nuis_host_stdout_write(int64_t handle) {
    const char* text = nuis_host_text_lookup(handle);
    size_t len = strlen(text);
    if (len == 0) return 0;
    return (int64_t)fwrite(text, 1, len, stdout);
}

static int64_t nuis_host_stderr_write(int64_t handle) {
    const char* text = nuis_host_text_lookup(handle);
    size_t len = strlen(text);
    if (len == 0) return 0;
    return (int64_t)fwrite(text, 1, len, stderr);
}

static int64_t nuis_host_stdout_flush(void) {
    return fflush(stdout) == 0 ? 1 : 0;
}

static int64_t nuis_host_stderr_flush(void) {
    return fflush(stderr) == 0 ? 1 : 0;
}

static int64_t nuis_host_tty_isatty(int64_t fd) {
    return isatty((int)fd) ? 1 : 0;
}

static int64_t nuis_host_tty_width(int64_t fd) {
    struct winsize ws;
    if (ioctl((int)fd, TIOCGWINSZ, &ws) != 0) return 0;
    return (int64_t)ws.ws_col;
}

static int64_t nuis_host_tty_height(int64_t fd) {
    struct winsize ws;
    if (ioctl((int)fd, TIOCGWINSZ, &ws) != 0) return 0;
    return (int64_t)ws.ws_row;
}

static int64_t nuis_host_cwd_handle(void) {
    char buffer[PATH_MAX];
    if (getcwd(buffer, sizeof(buffer)) == NULL) return 0;
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_cwd_len_value(void) {
    return nuis_host_text_len_value(nuis_host_cwd_handle());
}

static int64_t nuis_host_temp_dir_handle(void) {
    const char* tmp = getenv("TMPDIR");
    if (tmp == NULL || tmp[0] == '\0') tmp = "/tmp";
    return nuis_host_text_register(tmp);
}

static int64_t nuis_host_temp_path_len(void) {
    return nuis_host_text_len_value(nuis_host_temp_dir_handle());
}

static int64_t nuis_host_temp_file_handle(int64_t prefix_handle) {
    const char* prefix = nuis_host_text_lookup(prefix_handle);
    const char* tmp = getenv("TMPDIR");
    if (tmp == NULL || tmp[0] == '\0') tmp = "/tmp";
    char buffer[PATH_MAX];
    snprintf(buffer, sizeof(buffer), "%s/%sXXXXXX", tmp, (prefix != NULL && prefix[0] != '\0') ? prefix : "nuis");
    int fd = mkstemp(buffer);
    if (fd < 0) return 0;
    close(fd);
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_chdir_value(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return chdir(path) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_join_len(int64_t lhs_handle, int64_t rhs_handle) {
    const char* lhs = nuis_host_text_lookup(lhs_handle);
    const char* rhs = nuis_host_text_lookup(rhs_handle);
    size_t lhs_len = strlen(lhs);
    size_t rhs_len = strlen(rhs);
    size_t needs_sep = (lhs_len > 0 && rhs_len > 0 && lhs[lhs_len - 1] != '/') ? 1 : 0;
    return (int64_t)(lhs_len + needs_sep + rhs_len);
}

static int64_t nuis_host_path_is_absolute(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path != NULL && path[0] == '/') ? 1 : 0;
}

static int64_t nuis_host_path_is_empty(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path == NULL || path[0] == '\0') ? 1 : 0;
}

static int64_t nuis_host_path_is_dot(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    return (len == 1 && path[0] == '.') ? 1 : 0;
}

static int64_t nuis_host_path_is_dotdot(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 2 && path[len - 1] == '/') {
        len -= 1;
    }
    return (len == 2 && path[0] == '.' && path[1] == '.') ? 1 : 0;
}

static int64_t nuis_host_path_is_relative(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path != NULL && path[0] != '/') ? 1 : 0;
}

static int64_t nuis_host_path_is_root(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] != '/') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    return len == 1 ? 1 : 0;
}

static int64_t nuis_host_path_basename(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t slice_len = len - start;
    char buffer[PATH_MAX];
    if (slice_len >= sizeof(buffer)) slice_len = sizeof(buffer) - 1;
    memcpy(buffer, path + start, slice_len);
    buffer[slice_len] = '\0';
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_filename(int64_t path_handle) {
    return nuis_host_path_basename(path_handle);
}

static int64_t nuis_host_path_basename_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* name = nuis_host_text_lookup(name_handle);
    if (path == NULL || name == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t slice_len = len - start;
    size_t name_len = strlen(name);
    if (slice_len != name_len) return 0;
    return memcmp(path + start, name, slice_len) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_filename_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    return nuis_host_path_basename_matches(path_handle, name_handle);
}

static int64_t nuis_host_path_parent_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* name = nuis_host_text_lookup(name_handle);
    if (path == NULL || name == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] != '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t name_len = strlen(name);
    if (len != name_len) return 0;
    return memcmp(path, name, len) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_stem_matches(
    int64_t path_handle,
    int64_t name_handle
) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* name = nuis_host_text_lookup(name_handle);
    if (path == NULL || name == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t end = len;
    size_t dot = end;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    if (dot > start + 1 && dot < end) {
        end = dot - 1;
    }
    size_t stem_len = end - start;
    size_t name_len = strlen(name);
    if (stem_len != name_len) return 0;
    return memcmp(path + start, name, stem_len) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_parent(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] != '/') {
        len -= 1;
    }
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    char buffer[PATH_MAX];
    if (len >= sizeof(buffer)) len = sizeof(buffer) - 1;
    memcpy(buffer, path, len);
    buffer[len] = '\0';
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_has_parent(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    if (len == 1 && (path[0] == '.' || path[0] == '/')) return 0;
    if (len == 2 && path[0] == '.' && path[1] == '.') return 0;
    size_t i = len;
    while (i > 0) {
        if (path[i - 1] == '/') return 1;
        i -= 1;
    }
    return 0;
}

static int64_t nuis_host_path_is_basename_only(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    if (len == 1 && (path[0] == '.' || path[0] == '/')) return 0;
    if (len == 2 && path[0] == '.' && path[1] == '.') return 0;
    size_t i = 0;
    while (i < len) {
        if (path[i] == '/') return 0;
        i += 1;
    }
    return 1;
}

static int64_t nuis_host_path_depth(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    int64_t depth = 0;
    size_t i = 0;
    while (i < len) {
        while (i < len && path[i] == '/') {
            i += 1;
        }
        if (i >= len) break;
        depth += 1;
        while (i < len && path[i] != '/') {
            i += 1;
        }
    }
    return depth;
}

static int64_t nuis_host_path_stem(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t end = len;
    size_t dot = end;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    if (dot > start + 1 && dot < end) {
        end = dot - 1;
    }
    size_t slice_len = end - start;
    char buffer[PATH_MAX];
    if (slice_len >= sizeof(buffer)) slice_len = sizeof(buffer) - 1;
    memcpy(buffer, path + start, slice_len);
    buffer[slice_len] = '\0';
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_extension(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t dot = len;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    char buffer[PATH_MAX];
    if (dot > start + 1 && dot < len) {
        size_t slice_len = len - dot;
        if (slice_len >= sizeof(buffer)) slice_len = sizeof(buffer) - 1;
        memcpy(buffer, path + dot, slice_len);
        buffer[slice_len] = '\0';
    } else {
        buffer[0] = '\0';
    }
    return nuis_host_text_register(buffer);
}

static int64_t nuis_host_path_has_extension(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t dot = len;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    return (dot > start + 1 && dot < len) ? 1 : 0;
}

static int64_t nuis_host_path_matches_extension(int64_t path_handle, int64_t ext_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    const char* ext = nuis_host_text_lookup(ext_handle);
    if (path == NULL || ext == NULL) return 0;
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t dot = len;
    while (dot > start && path[dot - 1] != '.') {
        dot -= 1;
    }
    if (!(dot > start + 1 && dot < len)) return 0;
    const char* actual = path + dot;
    if (ext[0] == '.') ext += 1;
    return strcmp(actual, ext) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_extension_is(int64_t path_handle, int64_t ext_handle) {
    return nuis_host_path_matches_extension(path_handle, ext_handle);
}

static int64_t nuis_host_path_starts_with_dot(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    return (path != NULL && path[0] == '.') ? 1 : 0;
}

static int64_t nuis_host_path_ends_with_slash(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    size_t len = strlen(path);
    return (len > 1 && path[len - 1] == '/') ? 1 : 0;
}

static int64_t nuis_host_path_is_hidden(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    size_t len = strlen(path);
    while (len > 1 && path[len - 1] == '/') {
        len -= 1;
    }
    size_t start = len;
    while (start > 0 && path[start - 1] != '/') {
        start -= 1;
    }
    size_t slice_len = len - start;
    return (slice_len > 1 && path[start] == '.') ? 1 : 0;
}

static int64_t nuis_host_path_rename(int64_t src_handle, int64_t dst_handle) {
    const char* src = nuis_host_text_lookup(src_handle);
    const char* dst = nuis_host_text_lookup(dst_handle);
    if (src == NULL || src[0] == '\0' || dst == NULL || dst[0] == '\0') return 0;
    return rename(src, dst) == 0 ? 1 : 0;
}

static int64_t nuis_host_path_copy(int64_t src_handle, int64_t dst_handle) {
    const char* src = nuis_host_text_lookup(src_handle);
    const char* dst = nuis_host_text_lookup(dst_handle);
    if (src == NULL || src[0] == '\0' || dst == NULL || dst[0] == '\0') return 0;
    FILE* in = fopen(src, "rb");
    if (in == NULL) return 0;
    FILE* out = fopen(dst, "wb");
    if (out == NULL) {
        fclose(in);
        return 0;
    }
    char buffer[4096];
    int ok = 1;
    while (!feof(in)) {
        size_t got = fread(buffer, 1, sizeof(buffer), in);
        if (got > 0 && fwrite(buffer, 1, got, out) != got) {
            ok = 0;
            break;
        }
        if (ferror(in)) {
            ok = 0;
            break;
        }
    }
    fclose(in);
    if (fclose(out) != 0) ok = 0;
    return ok ? 1 : 0;
}

static int64_t nuis_host_path_remove(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    return unlink(path) == 0 ? 1 : 0;
}

static int64_t nuis_host_fs_exists(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    return stat(path, &st) == 0 ? 1 : 0;
}

static int64_t nuis_host_fs_kind(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
    if (S_ISREG(st.st_mode)) return 1;
    if (S_ISDIR(st.st_mode)) return 2;
    return 3;
}

static int64_t nuis_host_fs_size(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
    return (int64_t)st.st_size;
}

static int64_t nuis_host_stat_mode(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
    return (int64_t)st.st_mode;
}

static int64_t nuis_host_stat_mtime_ns(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
#if defined(__APPLE__)
    return (int64_t)st.st_mtimespec.tv_sec * 1000000000LL + (int64_t)st.st_mtimespec.tv_nsec;
#else
    return (int64_t)st.st_mtim.tv_sec * 1000000000LL + (int64_t)st.st_mtim.tv_nsec;
#endif
}

static int64_t nuis_host_stat_ctime_ns(int64_t path_handle) {
    const char* path = nuis_host_text_lookup(path_handle);
    struct stat st;
    if (stat(path, &st) != 0) return 0;
#if defined(__APPLE__)
    return (int64_t)st.st_ctimespec.tv_sec * 1000000000LL + (int64_t)st.st_ctimespec.tv_nsec;
#else
    return (int64_t)st.st_ctim.tv_sec * 1000000000LL + (int64_t)st.st_ctim.tv_nsec;
#endif
}

static int64_t nuis_host_process_id(void) {
    return (int64_t)getpid();
}

static int64_t nuis_host_process_status(void) {
    return 0;
}

static int64_t nuis_host_process_exit_code(int64_t status) {
    int raw = (int)status;
    if (WIFEXITED(raw)) return (int64_t)WEXITSTATUS(raw);
    if (WIFSIGNALED(raw)) return (int64_t)(128 + WTERMSIG(raw));
    return status;
}

static char* nuis_host_build_shell_command(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t env_handle
) {
    const char* program = nuis_host_text_lookup(program_handle);
    const char* argv_text = nuis_host_text_lookup(argv_handle);
    const char* env_text = nuis_host_text_lookup(env_handle);
    if (program == NULL || program[0] == '\0') return NULL;
    int has_argv = argv_text != NULL && argv_text[0] != '\0';
    int has_env = env_text != NULL && env_text[0] != '\0';
    size_t program_len = strlen(program);
    size_t argv_len = has_argv ? strlen(argv_text) : 0;
    size_t env_len = has_env ? strlen(env_text) : 0;
    size_t total = program_len + 1;
    if (has_argv) total += 1 + argv_len;
    if (has_env) total += 4 + env_len + 1;
    char* command = (char*)malloc(total);
    if (command == NULL) return NULL;
    if (has_env) {
        if (has_argv) {
            snprintf(command, total, "env %s %s %s", env_text, program, argv_text);
        } else {
            snprintf(command, total, "env %s %s", env_text, program);
        }
    } else if (has_argv) {
        snprintf(command, total, "%s %s", program, argv_text);
    } else {
        snprintf(command, total, "%s", program);
    }
    return command;
}

static int64_t nuis_host_now_monotonic_ns_raw(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) return 0;
    return (int64_t)ts.tv_sec * 1000000000LL + (int64_t)ts.tv_nsec;
}

static int64_t nuis_host_deadline_ns_from_timeout_ms(int64_t timeout_ms) {
    if (timeout_ms <= 0) return 0;
    int64_t now = nuis_host_now_monotonic_ns_raw();
    if (now <= 0) return 0;
    return now + (timeout_ms * 1000000LL);
}

static int nuis_host_timeout_expired(int64_t deadline_ns) {
    if (deadline_ns <= 0) return 0;
    int64_t now = nuis_host_now_monotonic_ns_raw();
    if (now <= 0) return 0;
    return now >= deadline_ns;
}

static int nuis_host_apply_timeout_to_pid(
    pid_t pid,
    int* done_slot,
    int64_t* status_slot,
    int* timed_out_slot,
    int64_t deadline_ns
) {
    if (*done_slot) return 0;
    if (!nuis_host_timeout_expired(deadline_ns)) return 0;
    kill(pid, SIGKILL);
    int status = 0;
    pid_t result = waitpid(pid, &status, 0);
    if (result < 0) return 0;
    *done_slot = 1;
    *status_slot = (int64_t)status;
    *timed_out_slot = 1;
    return 1;
}

static int64_t nuis_host_command_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
);

static int64_t nuis_host_subprocess_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t env_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
);

static pid_t nuis_host_spawn_shell(char* program, int64_t cwd_handle) {
    if (program == NULL || program[0] == '\0') return -1;
    pid_t pid = fork();
    if (pid < 0) return -1;
    if (pid == 0) {
        const char* cwd = nuis_host_text_lookup(cwd_handle);
        if (cwd != NULL && cwd[0] != '\0') {
            if (chdir(cwd) != 0) _exit(127);
        }
        execl("/bin/sh", "sh", "-c", program, (char*)NULL);
        _exit(127);
    }
    return pid;
}

static int64_t nuis_host_command_spawn(int64_t program_handle, int64_t argv_handle) {
    return nuis_host_command_spawn_in(program_handle, argv_handle, 0, 0);
}

static int64_t nuis_host_command_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
) {
    if (nuis_host_command_len >= 256) return 0;
    char* command = nuis_host_build_shell_command(program_handle, argv_handle, 0);
    pid_t pid = nuis_host_spawn_shell(command, cwd_handle);
    free(command);
    if (pid < 0) return 0;
    nuis_host_command_pids[nuis_host_command_len] = pid;
    nuis_host_command_status_slots[nuis_host_command_len] = 0;
    nuis_host_command_done[nuis_host_command_len] = 0;
    nuis_host_command_timed_out[nuis_host_command_len] = 0;
    nuis_host_command_deadline_ns[nuis_host_command_len] =
        nuis_host_deadline_ns_from_timeout_ms(timeout_ms);
    nuis_host_command_len += 1;
    return nuis_host_command_len;
}

static int64_t nuis_host_command_status(int64_t command_handle) {
    if (command_handle <= 0 || command_handle > nuis_host_command_len) return 0;
    int64_t idx = command_handle - 1;
    if (nuis_host_command_done[idx]) return nuis_host_command_status_slots[idx];
    if (nuis_host_apply_timeout_to_pid(
            nuis_host_command_pids[idx],
            &nuis_host_command_done[idx],
            &nuis_host_command_status_slots[idx],
            &nuis_host_command_timed_out[idx],
            nuis_host_command_deadline_ns[idx]
        )) {
        return nuis_host_command_status_slots[idx];
    }
    int status = 0;
    pid_t result = waitpid(nuis_host_command_pids[idx], &status, WNOHANG);
    if (result == nuis_host_command_pids[idx]) {
        nuis_host_command_done[idx] = 1;
        nuis_host_command_status_slots[idx] = (int64_t)status;
    }
    return nuis_host_command_status_slots[idx];
}

static int64_t nuis_host_command_wait(int64_t command_handle) {
    if (command_handle <= 0 || command_handle > nuis_host_command_len) return 0;
    int64_t idx = command_handle - 1;
    if (nuis_host_command_done[idx]) return nuis_host_command_status_slots[idx];
    if (nuis_host_apply_timeout_to_pid(
            nuis_host_command_pids[idx],
            &nuis_host_command_done[idx],
            &nuis_host_command_status_slots[idx],
            &nuis_host_command_timed_out[idx],
            nuis_host_command_deadline_ns[idx]
        )) {
        return nuis_host_command_status_slots[idx];
    }
    int status = 0;
    pid_t result = waitpid(nuis_host_command_pids[idx], &status, 0);
    if (result < 0) return 0;
    nuis_host_command_done[idx] = 1;
    nuis_host_command_status_slots[idx] = (int64_t)status;
    return nuis_host_command_status_slots[idx];
}

static int64_t nuis_host_command_wait_exit(int64_t command_handle) {
    if (command_handle > 0 && command_handle <= nuis_host_command_len) {
        int64_t idx = command_handle - 1;
        if (nuis_host_command_timed_out[idx]) return 124;
    }
    int64_t raw = nuis_host_command_wait(command_handle);
    if (command_handle > 0 && command_handle <= nuis_host_command_len) {
        int64_t idx = command_handle - 1;
        if (nuis_host_command_timed_out[idx]) return 124;
    }
    return nuis_host_process_exit_code(raw);
}

static int64_t nuis_host_subprocess_spawn(int64_t program_handle, int64_t argv_handle, int64_t env_handle) {
    return nuis_host_subprocess_spawn_in(program_handle, argv_handle, env_handle, 0, 0);
}

static int64_t nuis_host_subprocess_spawn_in(
    int64_t program_handle,
    int64_t argv_handle,
    int64_t env_handle,
    int64_t cwd_handle,
    int64_t timeout_ms
) {
    if (nuis_host_subprocess_len >= 256) return 0;
    char* command = nuis_host_build_shell_command(program_handle, argv_handle, env_handle);
    pid_t pid = nuis_host_spawn_shell(command, cwd_handle);
    free(command);
    if (pid < 0) return 0;
    nuis_host_subprocess_pids[nuis_host_subprocess_len] = pid;
    nuis_host_subprocess_status_slots[nuis_host_subprocess_len] = 0;
    nuis_host_subprocess_done[nuis_host_subprocess_len] = 0;
    nuis_host_subprocess_timed_out[nuis_host_subprocess_len] = 0;
    nuis_host_subprocess_deadline_ns[nuis_host_subprocess_len] =
        nuis_host_deadline_ns_from_timeout_ms(timeout_ms);
    nuis_host_subprocess_len += 1;
    return nuis_host_subprocess_len;
}

static int64_t nuis_host_subprocess_signal(int64_t process_handle, int64_t signal) {
    if (process_handle <= 0 || process_handle > nuis_host_subprocess_len) return 0;
    int64_t idx = process_handle - 1;
    if (nuis_host_subprocess_done[idx]) return 0;
    return kill(nuis_host_subprocess_pids[idx], (int)signal) == 0 ? 1 : 0;
}

static int64_t nuis_host_subprocess_join(int64_t process_handle) {
    if (process_handle <= 0 || process_handle > nuis_host_subprocess_len) return 0;
    int64_t idx = process_handle - 1;
    if (nuis_host_subprocess_done[idx]) return nuis_host_subprocess_status_slots[idx];
    if (nuis_host_apply_timeout_to_pid(
            nuis_host_subprocess_pids[idx],
            &nuis_host_subprocess_done[idx],
            &nuis_host_subprocess_status_slots[idx],
            &nuis_host_subprocess_timed_out[idx],
            nuis_host_subprocess_deadline_ns[idx]
        )) {
        return nuis_host_subprocess_status_slots[idx];
    }
    int status = 0;
    pid_t result = waitpid(nuis_host_subprocess_pids[idx], &status, 0);
    if (result < 0) return 0;
    nuis_host_subprocess_done[idx] = 1;
    nuis_host_subprocess_status_slots[idx] = (int64_t)status;
    return nuis_host_subprocess_status_slots[idx];
}

static int64_t nuis_host_subprocess_join_exit(int64_t process_handle) {
    if (process_handle > 0 && process_handle <= nuis_host_subprocess_len) {
        int64_t idx = process_handle - 1;
        if (nuis_host_subprocess_timed_out[idx]) return 124;
    }
    int64_t raw = nuis_host_subprocess_join(process_handle);
    if (process_handle > 0 && process_handle <= nuis_host_subprocess_len) {
        int64_t idx = process_handle - 1;
        if (nuis_host_subprocess_timed_out[idx]) return 124;
    }
    return nuis_host_process_exit_code(raw);
}

static int64_t nuis_host_wall_time_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) return 0;
    return (int64_t)ts.tv_sec * 1000000000LL + (int64_t)ts.tv_nsec;
}

static int64_t nuis_host_monotonic_time_ns(void) {
    return nuis_host_now_monotonic_ns_raw();
}

static int64_t nuis_host_sleep_ns(int64_t duration_ns) {
    if (duration_ns <= 0) return 0;
    struct timespec req;
    req.tv_sec = duration_ns / 1000000000LL;
    req.tv_nsec = duration_ns % 1000000000LL;
    nanosleep(&req, NULL);
    return duration_ns;
}

void nuis_debug_print_i64(int64_t value) {
    printf("%lld\n", (long long)value);
}

void nuis_debug_print_bool(int32_t value) {
    printf("%s\n", value ? "true" : "false");
}

void nuis_debug_print_i32(int32_t value) {
    printf("%d\n", value);
}

void nuis_debug_print_f32(float value) {
    printf("%g\n", value);
}

void nuis_debug_print_f64(double value) {
    printf("%g\n", value);
}

int64_t host_color_bias(int64_t value) {
    int64_t biased = value + 12;
    if (biased < 0) return 0;
    if (biased > 255) return 255;
    return biased;
}

int64_t host_speed_curve(int64_t value) {
    return value * 2 + 3;
}

int64_t host_radius_curve(int64_t value) {
    return (value * 3) / 2 + 8;
}

int64_t host_mix_tick(int64_t base, int64_t tick) {
    return base + tick;
}
"#,
    );
    for (symbol, function) in collect_host_ffi_symbols(ast) {
        out.push('\n');
        out.push_str(&render_host_ffi_stub(&symbol, function));
    }
    for export_name in collect_exported_entry_symbols(ast) {
        out.push('\n');
        out.push_str(&render_exported_entry_wrapper(&export_name));
    }
    out.push('\n');
    out.push_str(&render_lifecycle_export_wrappers());
    out.push_str(
        r#"

int main(int argc, char** argv) {
    nuis_argc = argc;
    nuis_argv = argv;
    if (nuis_lifecycle_bootstrap_entry_v1() != 0) {
        return 1;
    }
    int64_t entry_status = nuis_yir_entry();
    (void)nuis_lifecycle_tick_once_v1();
    return (int)nuis_lifecycle_shutdown_v1(entry_status);
}
"#,
    );
    out
}

fn collect_exported_entry_symbols(ast: &AstModule) -> Vec<String> {
    ast.functions
        .iter()
        .filter(|function| function.name == "main")
        .filter_map(|function| {
            function
                .attributes
                .iter()
                .find(|attribute| attribute.name == "export")
                .and_then(|attribute| attribute.args.first())
                .and_then(|arg| match &arg.value {
                    nuis_semantics::model::AstAttributeValue::String(value) => Some(value.clone()),
                    _ => None,
                })
        })
        .collect()
}

fn render_exported_entry_wrapper(symbol: &str) -> String {
    format!("int64_t {symbol}(void) {{\n    return nuis_yir_entry();\n}}\n")
}

fn render_lifecycle_export_wrappers() -> String {
    r#"int64_t nuis_lifecycle_bootstrap_export_v1(void) {
    return nuis_lifecycle_bootstrap_entry_v1();
}

int64_t nuis_lifecycle_tick_export_v1(void) {
    return nuis_lifecycle_tick_once_v1();
}

int64_t nuis_lifecycle_shutdown_export_v1(int64_t status) {
    return nuis_lifecycle_shutdown_v1(status);
}

int64_t nuis_lifecycle_yalivia_rpc_export_v1(void) {
    return nuis_lifecycle_yalivia_rpc_hook_v1();
}

int64_t nuis_lifecycle_network_bridge_progress_export_v1(void) {
    return nuis_lifecycle_state.network_bridge_progress_count;
}

int64_t nuis_lifecycle_hetero_submission_progress_export_v1(void) {
    return nuis_lifecycle_state.hetero_submission_progress_count;
}
"#
    .to_owned()
}

fn collect_host_ffi_symbols(ast: &AstModule) -> BTreeMap<String, AstExternFunction> {
    let mut out = BTreeMap::new();
    out.insert(
        "host_text_handle".to_owned(),
        AstExternFunction {
            visibility: nuis_semantics::model::AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: "host_text_handle".to_owned(),
            params: vec![nuis_semantics::model::AstParam {
                name: "text".to_owned(),
                ty: AstTypeRef {
                    name: "String".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: false,
                },
            }],
            return_type: AstTypeRef {
                name: "i64".to_owned(),
                generic_args: vec![],
                is_optional: false,
                is_ref: false,
            },
            host_symbol: None,
        },
    );
    for function in &ast.externs {
        if function.name.starts_with("host_") {
            out.insert(function.name.clone(), function.clone());
        }
    }
    for interface in &ast.extern_interfaces {
        for method in &interface.methods {
            out.insert(
                format!("{}__{}", interface.name, method.name),
                method.clone(),
            );
        }
    }
    out
}

fn render_host_ffi_stub(symbol: &str, function: AstExternFunction) -> String {
    let mut signature = String::new();
    if function.params.is_empty() {
        signature.push_str("void");
    } else {
        let mut first = true;
        for param in &function.params {
            if !first {
                signature.push_str(", ");
            }
            first = false;
            signature.push_str(&format!(
                "{} {}",
                c_type_for_ast_type(&param.ty),
                param.name
            ));
        }
    }
    let body = if symbol.ends_with("color_bias") {
        format!("    return host_color_bias({});", arg_name(0, &function))
    } else if symbol.ends_with("speed_curve") {
        format!("    return host_speed_curve({});", arg_name(0, &function))
    } else if symbol.ends_with("radius_curve") {
        format!("    return host_radius_curve({});", arg_name(0, &function))
    } else if symbol.ends_with("mix_tick") {
        format!(
            "    return host_mix_tick({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_argv_count" {
        "    return nuis_host_argv_count();".to_owned()
    } else if symbol == "host_argv_at" {
        format!("    return nuis_host_argv_at({});", arg_name(0, &function))
    } else if symbol == "host_env_has" {
        format!("    return nuis_host_env_has({});", arg_name(0, &function))
    } else if symbol == "host_env_get" {
        format!("    return nuis_host_env_get({});", arg_name(0, &function))
    } else if symbol == "host_text_handle" {
        format!(
            "    return nuis_host_text_handle({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_text_len" {
        format!(
            "    return nuis_host_text_len_value({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_text_concat" {
        format!(
            "    return nuis_host_text_concat({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_serialize_text_into" {
        format!(
            "    return nuis_host_serialize_text_into({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_serialize_i64_into" {
        format!(
            "    return nuis_host_serialize_i64_into({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_serialize_bool_into" {
        format!(
            "    return nuis_host_serialize_bool_into({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_serialize_byte_into" {
        format!(
            "    return nuis_host_serialize_byte_into({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_deserialize_i64_from" {
        format!(
            "    return nuis_host_deserialize_i64_from({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_deserialize_bool_from" {
        format!(
            "    return nuis_host_deserialize_bool_from({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_deserialize_byte_from" {
        format!(
            "    return nuis_host_deserialize_byte_from({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_deserialize_text_from" {
        format!(
            "    return nuis_host_deserialize_text_from({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_parse_header_line" {
        format!(
            "    return nuis_host_parse_header_line({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_find_header_value" {
        format!(
            "    return nuis_host_find_header_value({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_find_status_line_reason" {
        format!(
            "    return nuis_host_find_status_line_reason({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_parse_http_response_summary" {
        format!(
            "    return nuis_host_parse_http_response_summary({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_parse_http_request_summary" {
        format!(
            "    return nuis_host_parse_http_request_summary({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_parse_http_roundtrip_summary" {
        format!(
            "    return nuis_host_parse_http_roundtrip_summary({}, {}, {}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function),
            arg_name(4, &function),
            arg_name(5, &function)
        )
    } else if symbol == "host_deserialize_text_equals" {
        format!(
            "    return nuis_host_deserialize_text_equals({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_deserialize_text_starts_with" {
        format!(
            "    return nuis_host_deserialize_text_starts_with({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_deserialize_text_contains" {
        format!(
            "    return nuis_host_deserialize_text_contains({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_deserialize_text_ends_with" {
        format!(
            "    return nuis_host_deserialize_text_ends_with({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_buffer_find_byte" {
        format!(
            "    return nuis_host_buffer_find_byte({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_fill_bytes" {
        format!(
            "    return nuis_host_fill_bytes({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_copy_bytes" {
        format!(
            "    return nuis_host_copy_bytes({}, {}, {}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function),
            arg_name(4, &function),
            arg_name(5, &function)
        )
    } else if symbol == "host_compare_bytes" {
        format!(
            "    return nuis_host_compare_bytes({}, {}, {}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function),
            arg_name(4, &function),
            arg_name(5, &function)
        )
    } else if symbol == "host_buffer_find_text" {
        format!(
            "    return nuis_host_buffer_find_text({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_buffer_find_line_end" {
        format!(
            "    return nuis_host_buffer_find_line_end({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_buffer_trim_line_end" {
        format!(
            "    return nuis_host_buffer_trim_line_end({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_file_open" {
        format!(
            "    return nuis_host_file_open({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_file_read" {
        format!(
            "    return nuis_host_file_read({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_file_write" {
        format!(
            "    return nuis_host_file_write({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_file_close" {
        format!(
            "    return nuis_host_file_close({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_network_connect_probe" {
        format!(
            "    return nuis_host_network_connect_probe({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_open_tcp_stream" {
        format!(
            "    return nuis_host_network_open_tcp_stream({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_network_open_tcp_listener" {
        format!(
            "    return nuis_host_network_open_tcp_listener({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_open_udp_datagram" {
        format!(
            "    return nuis_host_network_open_udp_datagram({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_network_bind_udp_datagram" {
        format!(
            "    return nuis_host_network_bind_udp_datagram({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_accept_owned" {
        format!(
            "    return nuis_host_network_accept_owned({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_close_owned" {
        format!(
            "    return nuis_host_network_close_owned({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_network_send_owned" {
        format!(
            "    return nuis_host_network_send_owned({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_recv_owned" {
        format!(
            "    return nuis_host_network_recv_owned({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_recv_http_status_owned" {
        format!(
            "    return nuis_host_network_recv_http_status_owned({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_accept_probe" {
        format!(
            "    return nuis_host_network_accept_probe({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_close" {
        format!(
            "    return nuis_host_network_close({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_network_send_probe" {
        format!(
            "    return nuis_host_network_send_probe({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_network_recv_probe" {
        format!(
            "    return nuis_host_network_recv_probe({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_dir_open" {
        format!("    return nuis_host_dir_open({});", arg_name(0, &function))
    } else if symbol == "host_dir_entry_count" {
        format!(
            "    return nuis_host_dir_entry_count({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_dir_close" {
        format!(
            "    return nuis_host_dir_close({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_dir_create" {
        format!(
            "    return nuis_host_dir_create({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_dir_remove" {
        format!(
            "    return nuis_host_dir_remove({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_stdin_read" {
        format!(
            "    return nuis_host_stdin_read({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_stdout_write" {
        format!(
            "    return nuis_host_stdout_write({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_stderr_write" {
        format!(
            "    return nuis_host_stderr_write({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_stdout_flush" {
        "    return nuis_host_stdout_flush();".to_owned()
    } else if symbol == "host_stderr_flush" {
        "    return nuis_host_stderr_flush();".to_owned()
    } else if symbol == "host_tty_isatty" {
        format!(
            "    return nuis_host_tty_isatty({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_tty_width" {
        format!(
            "    return nuis_host_tty_width({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_tty_height" {
        format!(
            "    return nuis_host_tty_height({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_cwd_handle" {
        "    return nuis_host_cwd_handle();".to_owned()
    } else if symbol == "host_cwd_len" {
        "    return nuis_host_cwd_len_value();".to_owned()
    } else if symbol == "host_temp_dir_handle" {
        "    return nuis_host_temp_dir_handle();".to_owned()
    } else if symbol == "host_temp_path_len" {
        "    return nuis_host_temp_path_len();".to_owned()
    } else if symbol == "host_temp_file_handle" {
        format!(
            "    return nuis_host_temp_file_handle({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_chdir" {
        format!(
            "    return nuis_host_chdir_value({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_join_len" {
        format!(
            "    return nuis_host_path_join_len({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_is_absolute" {
        format!(
            "    return nuis_host_path_is_absolute({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_is_empty" {
        format!(
            "    return nuis_host_path_is_empty({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_is_dot" {
        format!(
            "    return nuis_host_path_is_dot({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_is_dotdot" {
        format!(
            "    return nuis_host_path_is_dotdot({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_is_relative" {
        format!(
            "    return nuis_host_path_is_relative({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_is_root" {
        format!(
            "    return nuis_host_path_is_root({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_basename" {
        format!(
            "    return nuis_host_path_basename({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_filename" {
        format!(
            "    return nuis_host_path_filename({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_basename_matches" {
        format!(
            "    return nuis_host_path_basename_matches({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_filename_matches" {
        format!(
            "    return nuis_host_path_filename_matches({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_parent_matches" {
        format!(
            "    return nuis_host_path_parent_matches({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_stem_matches" {
        format!(
            "    return nuis_host_path_stem_matches({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_parent" {
        format!(
            "    return nuis_host_path_parent({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_has_parent" {
        format!(
            "    return nuis_host_path_has_parent({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_is_basename_only" {
        format!(
            "    return nuis_host_path_is_basename_only({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_depth" {
        format!(
            "    return nuis_host_path_depth({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_stem" {
        format!(
            "    return nuis_host_path_stem({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_extension" {
        format!(
            "    return nuis_host_path_extension({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_has_extension" {
        format!(
            "    return nuis_host_path_has_extension({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_matches_extension" {
        format!(
            "    return nuis_host_path_matches_extension({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_extension_is" {
        format!(
            "    return nuis_host_path_extension_is({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_starts_with_dot" {
        format!(
            "    return nuis_host_path_starts_with_dot({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_ends_with_slash" {
        format!(
            "    return nuis_host_path_ends_with_slash({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_is_hidden" {
        format!(
            "    return nuis_host_path_is_hidden({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_path_rename" {
        format!(
            "    return nuis_host_path_rename({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_copy" {
        format!(
            "    return nuis_host_path_copy({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_path_remove" {
        format!(
            "    return nuis_host_path_remove({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_fs_exists" {
        format!(
            "    return nuis_host_fs_exists({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_fs_kind" {
        format!("    return nuis_host_fs_kind({});", arg_name(0, &function))
    } else if symbol == "host_fs_size" {
        format!("    return nuis_host_fs_size({});", arg_name(0, &function))
    } else if symbol == "host_stat_mode" {
        format!(
            "    return nuis_host_stat_mode({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_stat_mtime_ns" {
        format!(
            "    return nuis_host_stat_mtime_ns({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_stat_ctime_ns" {
        format!(
            "    return nuis_host_stat_ctime_ns({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_process_id" {
        "    return nuis_host_process_id();".to_owned()
    } else if symbol == "host_process_status" {
        "    return nuis_host_process_status();".to_owned()
    } else if symbol == "host_process_exit_code" {
        format!(
            "    return nuis_host_process_exit_code({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_command_spawn" {
        format!(
            "    return nuis_host_command_spawn({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_command_spawn_in" {
        format!(
            "    return nuis_host_command_spawn_in({}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function)
        )
    } else if symbol == "host_command_status" {
        format!(
            "    return nuis_host_command_status({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_command_wait" {
        format!(
            "    return nuis_host_command_wait({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_command_wait_exit" {
        format!(
            "    return nuis_host_command_wait_exit({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_subprocess_spawn" {
        format!(
            "    return nuis_host_subprocess_spawn({}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function)
        )
    } else if symbol == "host_subprocess_spawn_in" {
        format!(
            "    return nuis_host_subprocess_spawn_in({}, {}, {}, {}, {});",
            arg_name(0, &function),
            arg_name(1, &function),
            arg_name(2, &function),
            arg_name(3, &function),
            arg_name(4, &function)
        )
    } else if symbol == "host_subprocess_signal" {
        format!(
            "    return nuis_host_subprocess_signal({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else if symbol == "host_subprocess_join" {
        format!(
            "    return nuis_host_subprocess_join({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_subprocess_join_exit" {
        format!(
            "    return nuis_host_subprocess_join_exit({});",
            arg_name(0, &function)
        )
    } else if symbol == "host_wall_time_ns" {
        "    return nuis_host_wall_time_ns();".to_owned()
    } else if symbol == "host_monotonic_time_ns" {
        "    return nuis_host_monotonic_time_ns();".to_owned()
    } else if symbol == "host_sleep_ns" {
        format!("    return nuis_host_sleep_ns({});", arg_name(0, &function))
    } else {
        render_generic_host_ffi_body(&function)
    };
    format!(
        "{} {}({}) {{\n{}\n}}\n",
        c_type_for_ast_type(&function.return_type),
        symbol,
        signature,
        body
    )
}

fn arg_name(index: usize, function: &AstExternFunction) -> String {
    function
        .params
        .get(index)
        .map(|param| param.name.clone())
        .unwrap_or_else(|| "0".to_owned())
}

fn render_generic_host_ffi_body(function: &AstExternFunction) -> String {
    if function.params.is_empty() {
        return "    return 0;".to_owned();
    }
    if function.params.len() == 1 {
        return format!("    return {};", function.params[0].name);
    }
    let mut expr = String::new();
    for (idx, param) in function.params.iter().enumerate() {
        if idx > 0 {
            expr.push_str(" + ");
        }
        expr.push_str(&param.name);
    }
    format!("    return {};", expr)
}

fn c_type_for_ast_type(ty: &AstTypeRef) -> &'static str {
    match ty.name.as_str() {
        "i32" => "int32_t",
        "i64" => "int64_t",
        "f32" => "float",
        "f64" => "double",
        "bool" => "int32_t",
        _ => "int64_t",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_nuis_lifecycle_contract, c_shim_source, decode_nuis_compiled_artifact_binary,
        decode_nuis_executable_envelope_binary, encode_nuis_compiled_artifact_binary,
        encode_nuis_compiled_artifact_section_table_binary, encode_nuis_executable_envelope_binary,
        inspect_nuis_compiled_artifact_container, parse_nuis_compiled_artifact,
        parse_nuis_executable_envelope, render_nuis_executable_envelope,
        resolve_cpu_build_target_from_abi, verify_build_manifest, verify_nuis_compiled_artifact,
        BuildManifestCacheInfo, BuildManifestContext, BuildManifestDomainBuildUnit,
        BuildManifestProjectInfo, CompileArtifacts, CpuBuildTarget, NuisExecutableEnvelope,
    };
    use nuis_artifact::{
        decode_nuis_compiled_artifact_section_table_binary,
        encode_nuis_compiled_artifact_section_table,
        protocol::COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML,
    };
    use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef, AstVisibility};
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
    fn verify_compiled_artifact_rejects_binary_name_with_path_traversal() {
        let (dir, _manifest) = write_minimal_cpu_artifact("artifact_binary_name_traversal");
        let artifact_path = dir.join("nuis.compiled.artifact");
        let mut artifact = parse_nuis_compiled_artifact(&artifact_path).unwrap();
        artifact.binary_name = "../evil".to_owned();
        super::write_nuis_compiled_artifact(&artifact_path, &artifact).unwrap();

        let error = match verify_nuis_compiled_artifact(&artifact_path) {
            Ok(_) => panic!("artifact with traversal binary_name should fail verification"),
            Err(error) => error,
        };
        assert!(error.contains("unsafe binary_name"));
        assert!(error.contains("single file name"));
    }

    #[test]
    fn inspect_compiled_artifact_container_rejects_lowering_index_manifest_drift() {
        let (dir, _manifest) = write_minimal_cpu_artifact("artifact_lowering_index_drift");
        let artifact_path = dir.join("nuis.compiled.artifact");
        let artifact = parse_nuis_compiled_artifact(&artifact_path).unwrap();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        let lowering_section = table
            .sections
            .iter_mut()
            .find(|section| section.name == COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .unwrap();
        let drifted = std::str::from_utf8(&lowering_section.bytes)
            .unwrap()
            .replace(
                "selected_lowering_target = \"llvm\"",
                "selected_lowering_target = \"shader-msl\"",
            );
        lowering_section.bytes = drifted.into_bytes();
        let drifted_path = dir.join("nuis.compiled.drifted.v2.artifact");
        fs::write(
            &drifted_path,
            encode_nuis_compiled_artifact_section_table(&table).unwrap(),
        )
        .unwrap();

        let error = inspect_nuis_compiled_artifact_container(&drifted_path).unwrap_err();

        assert!(error.contains("inconsistent nuis artifact section payloads"));
        assert!(error.contains("selected_lowering_target"));
        assert!(error.contains("shader-msl"));
    }

    #[test]
    fn verify_build_manifest_rejects_artifact_path_outside_output_dir() {
        let (dir, manifest) = write_minimal_cpu_artifact("manifest_artifact_path_traversal");
        let mut source = fs::read_to_string(&manifest).unwrap();
        source = source.replace(
            &format!(
                "artifact_path = \"{}\"",
                dir.join("nuis.compiled.artifact").display()
            ),
            &format!(
                "artifact_path = \"{}\"",
                dir.join("..")
                    .join("evil")
                    .join("nuis.compiled.artifact")
                    .display()
            ),
        );
        fs::write(&manifest, source).unwrap();

        let error = match verify_build_manifest(&manifest) {
            Ok(_) => panic!("manifest with traversal artifact_path should fail verification"),
            Err(error) => error,
        };
        assert!(error.contains("unsafe nuis_artifact.artifact_path"));
        assert!(error.contains("parent-directory traversal"));
    }

    #[test]
    fn verify_build_manifest_rejects_artifact_hash_path_outside_output_dir() {
        let (dir, manifest) = write_minimal_cpu_artifact("manifest_artifact_hash_traversal");
        let mut source = fs::read_to_string(&manifest).unwrap();
        source = source.replace(
            &format!("path = \"{}\"", dir.join("demo.ast.txt").display()),
            &format!(
                "path = \"{}\"",
                dir.join("..").join("evil").join("demo.ast.txt").display()
            ),
        );
        fs::write(&manifest, source).unwrap();

        let error = match verify_build_manifest(&manifest) {
            Ok(_) => panic!("manifest with traversal artifact_hash path should fail verification"),
            Err(error) => error,
        };
        assert!(error.contains("unsafe artifact_hash.path"));
        assert!(error.contains("parent-directory traversal"));
    }

    #[test]
    fn project_metadata_summary_mismatch_error_suggests_rebuild_for_legacy_outputs() {
        let source_root = temp_dir("metadata_mismatch_source_exists");
        let message = super::project_metadata_summary_mismatch_error(
            "galaxy",
            "build/nuis.project.galaxy.txt",
            "summary\tgalaxies=1",
            "summary\tgalaxies=0\ncore\tpackage=nuis.core",
            &source_root.display().to_string(),
            "build",
        );
        assert!(message.contains("project galaxy index `build/nuis.project.galaxy.txt`"));
        assert!(message.contains("expected `summary\tgalaxies=1`"));
        assert!(message.contains("found `summary\tgalaxies=0`"));
        assert!(message.contains("older nuisc metadata format"));
        assert!(message.contains("Rebuild the project with the current nuisc"));
        assert!(message.contains(&format!(
            "nuisc compile \"{}\" \"build\"",
            source_root.display()
        )));
        assert!(message.contains(&format!(
            "nuisc inspect-project-metadata \"{}\"",
            source_root.display()
        )));
    }

    #[test]
    fn project_metadata_summary_mismatch_error_falls_back_to_manifest_commands_when_source_missing()
    {
        let message = super::project_metadata_summary_mismatch_error(
            "galaxy",
            "build/nuis.project.galaxy.txt",
            "summary\tgalaxies=1",
            "summary\tgalaxies=0\ncore\tpackage=nuis.core",
            "/tmp/definitely-missing-nuis-project-input",
            "build/out",
        );
        assert!(message.contains("older nuisc metadata format"));
        assert!(message
            .contains("nuisc inspect-project-metadata \"build/out/nuis.build.manifest.toml\""));
        assert!(
            message.contains("nuisc verify-build-manifest \"build/out/nuis.build.manifest.toml\"")
        );
    }

    fn sample_domain_unit(
        domain_family: &str,
        package_id: &str,
        backend_family: &str,
        vendor: &str,
        device_class: &str,
        selected_lowering_target: &str,
    ) -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: package_id.to_owned(),
            domain_family: domain_family.to_owned(),
            abi: None,
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some(backend_family.to_owned()),
            vendor: Some(vendor.to_owned()),
            device_class: Some(device_class.to_owned()),
            selected_lowering_target: Some(selected_lowering_target.to_owned()),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            contract_family: format!("nustar.{domain_family}"),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    #[test]
    fn resolve_cpu_build_target_for_known_abis() {
        let registry_root = registry_root();
        let apple =
            resolve_cpu_build_target_from_abi(&registry_root, "cpu.arm64.apple_aapcs64").unwrap();
        assert_eq!(apple.machine_arch, "arm64");
        assert_eq!(apple.machine_os, "darwin");
        assert_eq!(apple.clang_target, "aarch64-apple-darwin");
        assert_eq!(apple.isa_family, "aarch64");
        assert!(apple.isa_features.contains(&"neon".to_owned()));
        assert!(apple.isa_features.contains(&"lse".to_owned()));

        let apple_amd64 =
            resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.apple_sysv64").unwrap();
        assert_eq!(apple_amd64.machine_arch, "x86_64");
        assert_eq!(apple_amd64.machine_os, "darwin");
        assert_eq!(apple_amd64.object_format, "mach-o");
        assert_eq!(apple_amd64.calling_abi, "sysv64");
        assert_eq!(apple_amd64.clang_target, "x86_64-apple-darwin");
        assert_eq!(apple_amd64.isa_family, "x86_64");
        assert!(apple_amd64.isa_features.contains(&"sse2".to_owned()));
        assert!(apple_amd64.isa_features.contains(&"avx2".to_owned()));

        let linux = resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.sysv64").unwrap();
        assert_eq!(linux.machine_arch, "x86_64");
        assert_eq!(linux.machine_os, "linux");
        assert_eq!(linux.object_format, "elf");
        assert_eq!(linux.calling_abi, "sysv64");
        assert_eq!(linux.isa_family, "x86_64");
        assert!(linux.isa_features.contains(&"bmi2".to_owned()));

        let windows =
            resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.win64").unwrap();
        assert_eq!(windows.machine_os, "windows");
        assert_eq!(windows.clang_target, "x86_64-pc-windows-msvc");
        assert_eq!(windows.isa_family, "x86_64");
        assert!(windows.isa_features.contains(&"sse4.2".to_owned()));
        assert!(!windows.isa_features.contains(&"avx2".to_owned()));
    }

    #[test]
    fn shader_lowering_and_stub_include_profile_aware_fields() {
        let shader_unit = sample_domain_unit(
            "shader",
            "official.shader",
            "metal",
            "apple",
            "apple-silicon-gpu",
            "metal.apple-silicon-gpu",
        );
        let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
        let backend_stub = super::render_domain_build_unit_backend_stub(&shader_unit);
        let host_bridge_stub = super::render_domain_build_unit_host_bridge_stub(&shader_unit);

        assert!(lowering_plan.contains("lowering_profile = \"metal.apple-silicon-gpu\""));
        assert!(lowering_plan.contains("execution_route = \"unified-render-graph\""));
        assert!(lowering_plan.contains("submission_adapter = \"metal-command-encoder\""));
        assert!(lowering_plan.contains("wake_adapter = \"metal-shared-event\""));
        assert!(
            lowering_plan.contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]")
        );
        assert!(lowering_plan.contains("lowering_ir = \"msl2.4\""));
        assert!(lowering_plan.contains("shader_stage_model = \"metal-render-pipeline\""));
        assert!(lowering_plan.contains("stage_binding_model = \"argument-buffer-specialized\""));
        assert!(lowering_plan.contains("dispatch_encoding_model = \"tile-and-threadgroup\""));

        assert!(backend_stub.contains("backend_profile = \"metal.apple-silicon-gpu\""));
        assert!(backend_stub.contains("execution_route = \"unified-render-graph\""));
        assert!(backend_stub.contains("submission_adapter = \"metal-command-encoder\""));
        assert!(backend_stub.contains("wake_adapter = \"metal-shared-event\""));
        assert!(backend_stub.contains("shader_ir = \"msl2.4\""));
        assert!(
            backend_stub.contains("shader_entry_model = \"metal-function-constant-specialized\"")
        );
        assert!(backend_stub.contains("queue_binding_model = \"unified-command-queue\""));
        assert!(backend_stub.contains("resource_binding_model = \"argument-buffer-table\""));

        assert!(host_bridge_stub.contains("bridge_profile = \"metal.apple-silicon-gpu\""));
        assert!(host_bridge_stub.contains("execution_route = \"unified-render-graph\""));
        assert!(host_bridge_stub.contains("submission_adapter = \"metal-command-encoder\""));
        assert!(host_bridge_stub.contains("wake_adapter = \"metal-shared-event\""));
        let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);
        assert!(sidecar.contains("ir_container = \"text.msl\""));
        assert!(sidecar.contains("entry_symbol = \"main0\""));
        assert!(sidecar.contains("stage_kind = \"fragment\""));
        assert!(sidecar.contains("resource_layout = \"argument-buffer\""));
        assert!(sidecar.contains("[pipeline_layout]"));
        assert!(sidecar.contains("color_targets = [\"rgba8unorm\"]"));
        assert!(sidecar.contains("threadgroup_topology = \"tile\""));
        assert!(sidecar.contains("[resource_bindings]"));
        assert!(sidecar.contains("binding_table = \"material.uniforms, frame.texture0\""));
        assert!(sidecar.contains("[entry_points]"));
        assert!(sidecar.contains("vertex = \"vs_main\""));
        assert!(sidecar.contains("fragment = \"main0\""));
        assert!(sidecar.contains("compute = \"cs_main\""));
        assert!(sidecar.contains("#include <metal_stdlib>"));
        assert!(sidecar.contains("vertex float4 vs_main"));
        assert!(sidecar.contains("fragment float4 main0"));
        assert!(sidecar.contains("kernel void cs_main"));
    }

    #[test]
    fn shader_vulkan_lowering_plan_switches_to_spirv_pipeline_profile() {
        let shader_unit = sample_domain_unit(
            "shader",
            "official.shader",
            "vulkan",
            "cross-vendor",
            "discrete-or-integrated-gpu",
            "vulkan.discrete-or-integrated-gpu",
        );
        let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
        let backend_stub = super::render_domain_build_unit_backend_stub(&shader_unit);

        assert!(lowering_plan.contains("lowering_profile = \"vulkan.discrete-or-integrated-gpu\""));
        assert!(lowering_plan.contains("execution_route = \"spirv-render-queue\""));
        assert!(lowering_plan.contains("submission_adapter = \"vulkan-command-buffer\""));
        assert!(lowering_plan.contains("wake_adapter = \"vulkan-timeline-semaphore\""));
        assert!(
            lowering_plan.contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]")
        );
        assert!(lowering_plan.contains("lowering_ir = \"spirv1.6\""));
        assert!(lowering_plan.contains("shader_stage_model = \"spirv-graphics-pipeline\""));
        assert!(lowering_plan.contains("stage_binding_model = \"descriptor-set-layout\""));
        assert!(lowering_plan.contains("dispatch_encoding_model = \"renderpass-command-buffer\""));

        assert!(backend_stub.contains("backend_profile = \"vulkan.discrete-or-integrated-gpu\""));
        assert!(backend_stub.contains("shader_ir = \"spirv1.6\""));
        assert!(backend_stub.contains("shader_entry_model = \"vulkan-pipeline\""));
        assert!(backend_stub.contains("queue_binding_model = \"explicit-device-queue\""));
        assert!(backend_stub.contains("resource_binding_model = \"descriptor-set-layout\""));
        let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);
        assert!(sidecar.contains("ir_container = \"text.spirv\""));
        assert!(sidecar.contains("entry_symbol = \"main\""));
        assert!(sidecar.contains("stage_kind = \"fragment\""));
        assert!(sidecar.contains("resource_layout = \"descriptor-set\""));
        assert!(sidecar.contains("[pipeline_layout]"));
        assert!(sidecar.contains("threadgroup_topology = \"quad-fragment\""));
        assert!(sidecar.contains("[resource_bindings]"));
        assert!(
            sidecar.contains("binding_table = \"set0.binding0.texture, set0.binding1.sampler\"")
        );
        assert!(sidecar.contains("[entry_points]"));
        assert!(sidecar.contains("vertex = \"vs_main\""));
        assert!(sidecar.contains("fragment = \"main\""));
        assert!(sidecar.contains("compute = \"cs_main\""));
        assert!(sidecar.contains("OpCapability Shader"));
        assert!(sidecar.contains("OpEntryPoint Vertex %vs_main"));
        assert!(sidecar.contains("OpEntryPoint Fragment %main"));
        assert!(sidecar.contains("OpEntryPoint GLCompute %cs_main"));
    }

    #[test]
    fn shader_unknown_profile_falls_back_to_fragment_only_stage_set() {
        let shader_unit = sample_domain_unit(
            "shader",
            "official.shader",
            "experimental",
            "generic",
            "fragment-only-lab",
            "experimental.fragment-only-lab",
        );
        let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
        let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);

        assert!(lowering_plan.contains("supported_stages = [\"fragment\"]"));
        assert!(sidecar.contains("supported_stages = [\"fragment\"]"));
        assert!(sidecar.contains("entry_symbol = \"unimplemented\""));
        assert!(sidecar.contains("fragment = \"unimplemented\""));
        assert!(!sidecar.contains("vertex = "));
        assert!(!sidecar.contains("compute = "));
    }

    #[test]
    fn kernel_coreml_profile_reports_dispatch_kinds() {
        let kernel_unit = sample_domain_unit(
            "kernel",
            "official.kernel",
            "coreml",
            "apple",
            "apple-ane",
            "coreml.apple-ane",
        );
        let lowering_plan = super::render_domain_build_unit_lowering_plan(&kernel_unit);
        let backend_stub = super::render_domain_build_unit_backend_stub(&kernel_unit);

        assert!(
            lowering_plan.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]")
        );
        assert!(
            backend_stub.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]")
        );
    }

    #[test]
    fn kernel_coreml_sidecar_emits_dispatch_templates() {
        let kernel_unit = sample_domain_unit(
            "kernel",
            "official.kernel",
            "coreml",
            "apple",
            "apple-ane",
            "coreml.apple-ane",
        );
        let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

        assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
        assert!(sidecar.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
        assert!(sidecar.contains("[dispatch_shapes]"));
        assert!(sidecar.contains("primary = \"graph\""));
        assert!(sidecar.contains("[entry_points]"));
        assert!(sidecar.contains("graph = \"infer_main\""));
        assert!(sidecar.contains("batch = \"infer_batch\""));
        assert!(sidecar.contains("graph_body = \"program infer_main"));
    }

    #[test]
    fn kernel_vulkan_sidecar_emits_grid_and_indirect_dispatch_templates() {
        let kernel_unit = sample_domain_unit(
            "kernel",
            "official.kernel",
            "vulkan",
            "cross-vendor",
            "discrete-or-integrated-gpu",
            "vulkan.discrete-or-integrated-gpu",
        );
        let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

        assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
        assert!(sidecar.contains("supported_dispatch_kinds = [\"grid\", \"indirect\", \"batch\"]"));
        assert!(sidecar.contains("primary = \"grid\""));
        assert!(sidecar.contains("fallback = \"indirect\""));
        assert!(sidecar.contains("binding_table = \"set0.buffer0, set0.buffer1\""));
        assert!(sidecar.contains("grid = \"main\""));
        assert!(sidecar.contains("indirect = \"main_indirect\""));
        assert!(sidecar.contains("OpEntryPoint GLCompute %main"));
    }

    #[test]
    fn kernel_cpu_fallback_sidecar_emits_range_and_tile_dispatch_templates() {
        let kernel_unit = sample_domain_unit(
            "kernel",
            "official.kernel",
            "cpu-fallback",
            "generic",
            "cpu-host",
            "cpu-fallback.cpu-host",
        );
        let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

        assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
        assert!(sidecar.contains("supported_dispatch_kinds = [\"range\", \"tile\", \"batch\"]"));
        assert!(sidecar.contains("primary = \"range\""));
        assert!(sidecar.contains("fallback = \"tile\""));
        assert!(sidecar.contains("binding_table = \"slice.input, slice.output\""));
        assert!(sidecar.contains("range = \"run_range\""));
        assert!(sidecar.contains("tile = \"run_tile\""));
        assert!(sidecar.contains("range_body = \"fn run_range"));
    }

    #[test]
    fn network_urlsession_sidecar_emits_foundation_session_templates() {
        let network_unit = sample_domain_unit(
            "network",
            "official.network",
            "urlsession",
            "apple",
            "socket-io",
            "urlsession.socket-io",
        );
        let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

        assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
        assert!(sidecar.contains("transport_ir = \"foundation-url-request\""));
        assert!(sidecar.contains("transport_binding_model = \"session-task-packet\""));
        assert!(sidecar.contains("[session_shapes]"));
        assert!(sidecar.contains("request = \"http-client-session\""));
        assert!(sidecar.contains("response = \"completion-callback\""));
        assert!(sidecar.contains("streaming = \"delegate-push-stream\""));
        assert!(
            sidecar.contains("binding_table = \"session.handle, request.packet, response.slot\"")
        );
        assert!(sidecar.contains("connect = \"open_session\""));
        assert!(sidecar.contains("send = \"submit_request\""));
        assert!(sidecar.contains("recv = \"on_response\""));
        assert!(sidecar.contains("finalize = \"finish_exchange\""));
    }

    #[test]
    fn network_socket_abi_sidecar_emits_poll_reactor_templates() {
        let network_unit = sample_domain_unit(
            "network",
            "official.network",
            "socket-abi",
            "cross-vendor",
            "socket-io",
            "socket-abi.socket-io",
        );
        let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

        assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
        assert!(sidecar.contains("transport_ir = \"posix-socket\""));
        assert!(sidecar.contains("transport_binding_model = \"packet-poll-reactor\""));
        assert!(sidecar.contains("request = \"socket-reactor-session\""));
        assert!(sidecar.contains("response = \"poll-ready-response\""));
        assert!(sidecar.contains("streaming = \"fd-edge-stream\""));
        assert!(sidecar.contains("binding_table = \"fd.handle, packet.buffer, ready.token\""));
        assert!(sidecar.contains("connect = \"open_fd_session\""));
        assert!(sidecar.contains("recv = \"poll_ready_response\""));
        assert!(sidecar.contains("finalize = \"finish_poll_exchange\""));
    }

    #[test]
    fn network_winsock_sidecar_emits_iocp_templates() {
        let network_unit = sample_domain_unit(
            "network",
            "official.network",
            "winsock",
            "microsoft",
            "socket-io",
            "winsock.socket-io",
        );
        let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

        assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
        assert!(sidecar.contains("transport_ir = \"winsock-overlapped\""));
        assert!(sidecar.contains("transport_binding_model = \"overlapped-packet-reactor\""));
        assert!(sidecar.contains("request = \"overlapped-client-session\""));
        assert!(sidecar.contains("response = \"iocp-completion\""));
        assert!(sidecar.contains("streaming = \"completion-port-stream\""));
        assert!(sidecar
            .contains("binding_table = \"socket.handle, overlapped.packet, completion.port\""));
        assert!(sidecar.contains("connect = \"connect_overlapped\""));
        assert!(sidecar.contains("recv = \"await_iocp_completion\""));
        assert!(sidecar.contains("finalize = \"finish_iocp_exchange\""));
    }

    #[test]
    fn build_manifest_emits_shader_ir_sidecar() {
        let dir = temp_dir("build_manifest_shader_sidecar");
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
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
            machine_arch: "arm64".to_owned(),
            machine_os: "darwin".to_owned(),
            object_format: "macho".to_owned(),
            calling_abi: "apple_aapcs64".to_owned(),
            clang_target: "arm64-apple-darwin".to_owned(),
            isa_family: "aarch64".to_owned(),
            isa_features: vec!["a64".to_owned(), "neon".to_owned()],
            cross_compile: false,
        };
        let manifest = super::write_build_manifest(
            &dir,
            &written,
            &BuildManifestContext {
                input_path: "/tmp/shader.ns".to_owned(),
                output_dir: dir.display().to_string(),
                loaded_nustar: vec!["official.cpu".to_owned(), "official.shader".to_owned()],
                compile_cache: None,
                project: Some(BuildManifestProjectInfo {
                    name: "shader".to_owned(),
                    abi_mode: "explicit".to_owned(),
                    abi_graph_summary: None,
                    abi_entries: vec![
                        ("cpu".to_owned(), cpu_target.abi.clone()),
                        ("shader".to_owned(), "shader.metal.msl2_4".to_owned()),
                    ],
                    plan_summary: None,
                    effective_input: None,
                    text_handle_rewrite_helper_hits: 0,
                    text_handle_rewrite_local_hits: 0,
                    manifest_copy_path: None,
                    plan_index_path: None,
                    organization_index_path: None,
                    exchange_index_path: None,
                    modules_index_path: None,
                    docs_index_path: None,
                    docs_module_count: 0,
                    docs_documented_module_count: 0,
                    docs_documented_item_count: 0,
                    imports_index_path: None,
                    imports_library_count: 0,
                    imports_visible_library_count: 0,
                    imports_visible_module_count: 0,
                    imports_documented_visible_module_count: 0,
                    imports_documented_visible_item_count: 0,
                    galaxy_index_path: None,
                    galaxy_count: 0,
                    galaxy_documented_count: 0,
                    galaxy_documented_library_module_count: 0,
                    galaxy_documented_item_count: 0,
                    links_index_path: None,
                    packet_index_path: None,
                    host_ffi_index_path: None,
                    abi_index_path: None,
                }),
                doc_index: None,
                cpu_target,
            },
        )
        .unwrap();

        let report = verify_build_manifest(PathBuf::from(&manifest).as_path()).unwrap();
        let shader_unit = report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "shader")
            .unwrap();
        let shader_sidecar_path = dir.join("nuis.domain.shader.lowering.ir.txt");
        let shader_sidecar_path_text = shader_sidecar_path.display().to_string();
        let shader_payload_blob = dir.join("nuis.domain.shader.payload.bin");
        assert!(shader_sidecar_path.exists());
        assert_eq!(
            shader_unit.artifact_ir_sidecar_path.as_deref(),
            Some(shader_sidecar_path_text.as_str())
        );
        let shader_sidecar_text = fs::read_to_string(&shader_sidecar_path).unwrap();
        assert!(shader_sidecar_text.contains("schema = \"nuis-shader-ir-sidecar-v1\""));
        assert!(shader_sidecar_text.contains("lowering_profile = \"metal.apple-silicon-gpu\""));
        assert!(shader_sidecar_text.contains("lowering_ir = \"msl2.4\""));
        assert!(shader_sidecar_text
            .contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]"));
        assert!(shader_sidecar_text.contains("ir_container = \"text.msl\""));
        assert!(shader_sidecar_text.contains("entry_symbol = \"main0\""));
        assert!(shader_sidecar_text.contains("[pipeline_layout]"));
        assert!(shader_sidecar_text.contains("[resource_bindings]"));
        assert!(shader_sidecar_text.contains("[entry_points]"));
        assert!(shader_sidecar_text.contains("vertex = \"vs_main\""));
        assert!(shader_sidecar_text.contains("compute = \"cs_main\""));
        assert!(shader_sidecar_text.contains("fragment float4 main0"));

        let shader_blob =
            super::decode_domain_build_unit_payload_blob(&fs::read(&shader_payload_blob).unwrap())
                .unwrap();
        assert_eq!(shader_blob.sections.len(), 5);
        assert_eq!(shader_blob.sections[4].name, "shader_ir_sidecar");
        assert_eq!(
            shader_blob.sections[4].bytes,
            shader_sidecar_text.as_bytes()
        );
    }

    #[test]
    fn resolve_cpu_build_target_from_target_triple() {
        let registry_root = registry_root();
        let target = super::resolve_cpu_build_target_from_target(
            registry_root.as_path(),
            "x86_64-unknown-linux-gnu",
        )
        .unwrap();
        assert_eq!(target.machine_arch, "x86_64");
        assert_eq!(target.machine_os, "linux");
        assert_eq!(target.object_format, "elf");
        assert_eq!(target.calling_abi, "sysv64");
    }

    #[test]
    fn resolve_cpu_build_target_from_darwin_amd64_alias_triple() {
        let registry_root = registry_root();
        let target = super::resolve_cpu_build_target_from_target(
            registry_root.as_path(),
            "amd64-apple-darwin",
        )
        .unwrap();
        assert_eq!(target.abi, "cpu.x86_64.apple_sysv64");
        assert_eq!(target.machine_arch, "x86_64");
        assert_eq!(target.machine_os, "darwin");
        assert_eq!(target.object_format, "mach-o");
        assert_eq!(target.calling_abi, "sysv64");
        assert_eq!(target.clang_target, "x86_64-apple-darwin");
    }

    #[test]
    fn reject_conflicting_cpu_abi_and_target_override() {
        let registry_root = registry_root();
        let error = super::resolve_cpu_build_target(
            registry_root.as_path(),
            None,
            Some("cpu.arm64.apple_aapcs64"),
            Some("x86_64-unknown-linux-gnu"),
        )
        .unwrap_err();
        assert!(error.contains("--cpu-abi"));
        assert!(error.contains("--target"));
    }

    #[test]
    fn build_manifest_round_trips_cpu_target_metadata() {
        let dir = temp_dir("build_manifest_cpu_target");
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
                compile_cache: Some(BuildManifestCacheInfo {
                    status: "miss".to_owned(),
                    key: "abc".to_owned(),
                    root: "/tmp/cache".to_owned(),
                }),
                project: Some(BuildManifestProjectInfo {
                    name: "demo".to_owned(),
                    abi_mode: "explicit".to_owned(),
                    abi_graph_summary: Some(
                        "graph\tmode=explicit\tdomains=cpu\tcpu_summary=present\tdata_summary=absent\tkernel_target=absent\tshader_target=absent\tnetwork_target=absent"
                            .to_owned(),
                    ),
                    abi_entries: vec![("cpu".to_owned(), cpu_target.abi.clone())],
                    plan_summary: None,
                    effective_input: None,
                    text_handle_rewrite_helper_hits: 0,
                    text_handle_rewrite_local_hits: 0,
                    manifest_copy_path: None,
                    plan_index_path: None,
                    organization_index_path: None,
                    exchange_index_path: None,
                    modules_index_path: None,
                    docs_index_path: None,
                    docs_module_count: 0,
                    docs_documented_module_count: 0,
                    docs_documented_item_count: 0,
                    imports_index_path: None,
                    imports_library_count: 0,
                    imports_visible_library_count: 0,
                    imports_visible_module_count: 0,
                    imports_documented_visible_module_count: 0,
                    imports_documented_visible_item_count: 0,
                    galaxy_index_path: None,
                    galaxy_count: 0,
                    galaxy_documented_count: 0,
                    galaxy_documented_library_module_count: 0,
                    galaxy_documented_item_count: 0,
                    links_index_path: None,
                    packet_index_path: None,
                    host_ffi_index_path: None,
                    abi_index_path: None,
                }),
                doc_index: None,
                cpu_target: cpu_target.clone(),
            },
        )
        .unwrap();
        let manifest_text = std::fs::read_to_string(&manifest).unwrap();
        assert!(manifest_text.contains("[nuis_envelope]"));
        assert!(manifest_text.contains("path = "));
        assert!(manifest_text.contains("schema = \"nuis-executable-envelope-v1\""));
        assert!(manifest_text.contains("[nuis_artifact]"));
        assert!(manifest_text.contains("artifact_schema = \"nuis-compiled-artifact-v1\""));
        assert!(manifest_text.contains("domain_families = [\"cpu\"]"));
        assert!(manifest_text.contains("abi_graph = "));
        assert!(manifest_text.contains("graph\tmode=explicit"));
        assert!(manifest_text.contains("[[execution_contract]]"));
        assert!(manifest_text.contains("[[domain_build_unit]]"));
        assert!(manifest_text.contains("package_id = \"official.cpu\""));
        assert!(manifest_text.contains("contract_family = \"nustar.cpu\""));
        assert!(manifest_text.contains("packaging_role = \"host-binary\""));
        let envelope = parse_nuis_executable_envelope(PathBuf::from(&manifest).as_path()).unwrap();
        assert_eq!(envelope.schema, "nuis-executable-envelope-v1");
        assert_eq!(envelope.executable_kind, "native-cpu-llvm");
        assert_eq!(envelope.package_count, 1);
        assert_eq!(envelope.domain_families, vec!["cpu".to_owned()]);
        assert_eq!(envelope.contract_families, vec!["nustar.cpu".to_owned()]);
        let rendered_envelope = render_nuis_executable_envelope(&envelope);
        assert!(rendered_envelope.contains("envelope_schema = \"nuis-executable-envelope-v1\""));
        assert!(rendered_envelope.contains("executable_kind = \"native-cpu-llvm\""));
        let encoded_envelope = encode_nuis_executable_envelope_binary(&envelope).unwrap();
        let decoded_envelope = decode_nuis_executable_envelope_binary(&encoded_envelope).unwrap();
        assert_eq!(decoded_envelope, envelope);
        let compiled_artifact = parse_nuis_compiled_artifact(
            PathBuf::from(&dir).join("nuis.compiled.artifact").as_path(),
        )
        .unwrap();
        assert_eq!(compiled_artifact.schema, "nuis-compiled-artifact-v1");
        assert_eq!(compiled_artifact.packaging_mode, "native-cpu-llvm");
        assert_eq!(compiled_artifact.binary_name, "demo.bin");
        assert_eq!(compiled_artifact.binary_blob, b"bin".to_vec());
        assert_eq!(compiled_artifact.build_manifest_source, manifest_text);
        assert_eq!(compiled_artifact.build_manifest_bytes, manifest_text.len());
        assert_eq!(compiled_artifact.envelope, envelope);
        assert_eq!(
            compiled_artifact.lifecycle.schema,
            "nuis-lifecycle-contract-v1"
        );
        assert_eq!(
            compiled_artifact.lifecycle.bootstrap_entry,
            "nuis.bootstrap.lifecycle.v1"
        );
        assert_eq!(compiled_artifact.lifecycle.export_surface.len(), 4);
        assert_eq!(
            compiled_artifact.lifecycle.runtime_capability_flags.len(),
            4
        );
        assert!(compiled_artifact
            .lifecycle
            .export_surface
            .contains(&"nuis_lifecycle_tick_export_v1".to_owned()));
        assert!(compiled_artifact
            .lifecycle
            .runtime_capability_flags
            .contains(&"runtime.tick".to_owned()));
        assert!(manifest_text.contains("[nuis_lifecycle]"));
        assert!(manifest_text.contains("lifecycle_schema = \"nuis-lifecycle-contract-v1\""));
        assert!(manifest_text.contains("lifecycle_export_surface = ["));
        let unpacked_dir = dir.join("unpacked");
        fs::create_dir_all(&unpacked_dir).unwrap();
        let unpacked_envelope = unpacked_dir.join("nuis.executable.envelope.toml");
        let unpacked_artifact = unpacked_dir.join("nuis.compiled.artifact");
        let unpacked_binary = unpacked_dir.join("demo.bin");
        fs::write(&unpacked_binary, &compiled_artifact.binary_blob).unwrap();
        super::write_nuis_executable_envelope(&unpacked_envelope, &compiled_artifact.envelope)
            .unwrap();
        let relocated_manifest = super::render_relocated_unpacked_build_manifest(
            &compiled_artifact,
            &unpacked_dir,
            &unpacked_envelope,
            &unpacked_artifact,
            &unpacked_binary,
        )
        .unwrap();
        assert!(
            relocated_manifest.contains(&format!("output_dir = \"{}\"", unpacked_dir.display()))
        );
        assert!(relocated_manifest.contains(&format!(
            "artifact_path = \"{}\"",
            unpacked_artifact.display()
        )));
        assert!(!relocated_manifest.contains("plan_index = "));
        let encoded_artifact = encode_nuis_compiled_artifact_binary(&compiled_artifact).unwrap();
        let decoded_artifact = decode_nuis_compiled_artifact_binary(&encoded_artifact).unwrap();
        assert_eq!(decoded_artifact, compiled_artifact);
        let artifact_verify_report = verify_nuis_compiled_artifact(
            PathBuf::from(&dir).join("nuis.compiled.artifact").as_path(),
        )
        .unwrap();
        assert!(artifact_verify_report.lifecycle_contract_consistent);
        assert!(artifact_verify_report.lifecycle_runtime_capability_flags_consistent);
        let report = verify_build_manifest(PathBuf::from(manifest).as_path()).unwrap();
        assert!(std::path::Path::new(&report.envelope_path).exists());
        assert!(std::path::Path::new(&report.artifact_path).exists());
        assert_eq!(report.envelope_schema, "nuis-executable-envelope-v1");
        assert_eq!(report.envelope_package_count, 1);
        assert_eq!(report.artifact_schema, "nuis-compiled-artifact-v1");
        assert_eq!(report.artifact_binary_name, "demo.bin");
        assert_eq!(report.artifact_binary_bytes, 3);
        assert_eq!(report.lifecycle_schema, "nuis-lifecycle-contract-v1");
        assert_eq!(
            report.lifecycle_bootstrap_entry,
            "nuis.bootstrap.lifecycle.v1"
        );
        assert!(report.lifecycle_hook_count >= 7);
        assert!(report
            .lifecycle_hook_surface
            .contains(&"on_scheduler_tick".to_owned()));
        assert_eq!(report.lifecycle_export_count, 4);
        assert!(report
            .lifecycle_export_surface
            .contains(&"nuis_lifecycle_shutdown_export_v1".to_owned()));
        assert!(report
            .lifecycle_runtime_capability_flags
            .contains(&"runtime.shutdown".to_owned()));
        assert_eq!(report.execution_contracts_checked, 1);
        assert_eq!(report.domain_build_unit_count, 1);
        assert_eq!(report.heterogeneous_domain_count, 0);
        assert_eq!(report.domain_payload_blobs_checked, 0);
        assert_eq!(report.bridge_registry_path, None);
        assert_eq!(report.bridge_registry_units, 0);
        assert_eq!(report.bridge_registry_checked, 0);
        assert_eq!(report.host_bridge_plan_index_path, None);
        assert_eq!(report.host_bridge_plan_units, 0);
        assert_eq!(report.host_bridge_plan_checked, 0);
        assert_eq!(report.lowering_plan_index_path, None);
        assert_eq!(report.lowering_plan_units, 0);
        assert_eq!(report.lowering_plan_index_checked, 0);
        assert_eq!(report.doc_index_path, None);
        assert_eq!(report.doc_index_module_count, 0);
        assert_eq!(report.doc_index_documented_item_count, 0);
        assert_eq!(report.doc_index_checked, 0);
        assert_eq!(report.domain_build_units.len(), 1);
        assert_eq!(report.domain_build_units[0].domain_family, "cpu");
        assert_eq!(report.domain_build_units[0].artifact_stub_path, None);
        assert_eq!(report.domain_build_units[0].artifact_payload_path, None);
        assert_eq!(report.domain_build_units[0].artifact_bridge_stub_path, None);
        assert_eq!(
            report.domain_build_units[0].artifact_payload_blob_path,
            None
        );
        assert_eq!(
            report.domain_build_units[0].artifact_payload_blob_bytes,
            None
        );
        assert_eq!(report.domain_build_units[0].artifact_payload_format, None);
        assert_eq!(
            report.domain_build_units[0]
                .selected_lowering_target
                .as_deref(),
            Some("llvm")
        );
        assert_eq!(report.cpu_target_abi, cpu_target.abi);
        assert_eq!(report.cpu_target_machine_arch, cpu_target.machine_arch);
        assert_eq!(report.cpu_target_machine_os, cpu_target.machine_os);
        assert_eq!(report.cpu_target_object_format, cpu_target.object_format);
        assert_eq!(report.cpu_target_calling_abi, cpu_target.calling_abi);
        assert_eq!(report.cpu_target_clang, cpu_target.clang_target);
        assert!(report.cpu_target_cross);
        assert_eq!(report.project_metadata_checked, 0);
    }

    #[test]
    fn build_manifest_tracks_heterogeneous_domain_build_units() {
        let dir = temp_dir("build_manifest_heterogeneous_units");
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
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
            machine_arch: "arm64".to_owned(),
            machine_os: "darwin".to_owned(),
            object_format: "macho".to_owned(),
            calling_abi: "apple_aapcs64".to_owned(),
            clang_target: "arm64-apple-darwin".to_owned(),
            isa_family: "aarch64".to_owned(),
            isa_features: vec!["a64".to_owned(), "neon".to_owned()],
            cross_compile: false,
        };
        let manifest = super::write_build_manifest(
            &dir,
            &written,
            &BuildManifestContext {
                input_path: "/tmp/hetero.ns".to_owned(),
                output_dir: dir.display().to_string(),
                loaded_nustar: vec![
                    "official.cpu".to_owned(),
                    "official.kernel".to_owned(),
                    "official.network".to_owned(),
                ],
                compile_cache: None,
                project: Some(BuildManifestProjectInfo {
                    name: "hetero".to_owned(),
                    abi_mode: "explicit".to_owned(),
                    abi_graph_summary: None,
                    abi_entries: vec![
                        ("cpu".to_owned(), cpu_target.abi.clone()),
                        ("kernel".to_owned(), "kernel.apple_ane.coreml.v1".to_owned()),
                        (
                            "network".to_owned(),
                            "network.socket.macos.arm64.v1".to_owned(),
                        ),
                    ],
                    plan_summary: None,
                    effective_input: None,
                    text_handle_rewrite_helper_hits: 0,
                    text_handle_rewrite_local_hits: 0,
                    manifest_copy_path: None,
                    plan_index_path: None,
                    organization_index_path: None,
                    exchange_index_path: None,
                    modules_index_path: None,
                    docs_index_path: None,
                    docs_module_count: 0,
                    docs_documented_module_count: 0,
                    docs_documented_item_count: 0,
                    imports_index_path: None,
                    imports_library_count: 0,
                    imports_visible_library_count: 0,
                    imports_visible_module_count: 0,
                    imports_documented_visible_module_count: 0,
                    imports_documented_visible_item_count: 0,
                    galaxy_index_path: None,
                    galaxy_count: 0,
                    galaxy_documented_count: 0,
                    galaxy_documented_library_module_count: 0,
                    galaxy_documented_item_count: 0,
                    links_index_path: None,
                    packet_index_path: None,
                    host_ffi_index_path: None,
                    abi_index_path: None,
                }),
                doc_index: None,
                cpu_target,
            },
        )
        .unwrap();

        let report = verify_build_manifest(PathBuf::from(manifest).as_path()).unwrap();
        assert_eq!(report.envelope_package_count, 3);
        assert_eq!(report.execution_contracts_checked, 3);
        assert_eq!(report.domain_build_unit_count, 3);
        assert_eq!(report.heterogeneous_domain_count, 2);
        assert_eq!(report.domain_payload_blobs_checked, 2);
        assert_eq!(report.domain_payload_blob_sections_checked, 10);
        assert_eq!(report.domain_payload_contract_sections_checked, 2);
        assert_eq!(report.domain_payload_lowering_plans_checked, 2);
        assert_eq!(report.domain_payload_backend_stubs_checked, 2);
        assert_eq!(report.domain_payload_bridge_plans_checked, 2);
        assert_eq!(report.domain_bridge_stubs_checked, 2);
        assert_eq!(report.bridge_registry_units, 2);
        assert_eq!(report.bridge_registry_checked, 1);
        assert_eq!(report.bridge_registry_entries_checked, 2);
        assert_eq!(report.host_bridge_plan_units, 2);
        assert_eq!(report.host_bridge_plan_checked, 1);
        assert_eq!(report.host_bridge_plan_entries_checked, 2);
        assert_eq!(report.lowering_plan_units, 2);
        assert_eq!(report.lowering_plan_index_checked, 1);
        assert_eq!(report.lowering_plan_entries_checked, 2);
        let kernel_payload = dir.join("nuis.domain.kernel.payload.toml");
        let kernel_bridge_stub = dir.join("nuis.domain.kernel.bridge.stub.txt");
        let kernel_payload_blob = dir.join("nuis.domain.kernel.payload.bin");
        let network_payload = dir.join("nuis.domain.network.payload.toml");
        let network_bridge_stub = dir.join("nuis.domain.network.bridge.stub.txt");
        let network_payload_blob = dir.join("nuis.domain.network.payload.bin");
        let bridge_registry = dir.join("nuis.bridge.registry.toml");
        let host_bridge_plan_index = dir.join("nuis.host-bridge.plan-index.toml");
        let lowering_plan_index = dir.join("nuis.lowering.plan-index.toml");
        assert!(kernel_payload.exists());
        assert!(kernel_bridge_stub.exists());
        assert!(kernel_payload_blob.exists());
        assert!(network_payload.exists());
        assert!(network_bridge_stub.exists());
        assert!(network_payload_blob.exists());
        assert!(bridge_registry.exists());
        assert!(host_bridge_plan_index.exists());
        assert!(lowering_plan_index.exists());
        let kernel_payload_text = fs::read_to_string(&kernel_payload).unwrap();
        let kernel_bridge_stub_text = fs::read_to_string(&kernel_bridge_stub).unwrap();
        let network_payload_text = fs::read_to_string(&network_payload).unwrap();
        let network_bridge_stub_text = fs::read_to_string(&network_bridge_stub).unwrap();
        let bridge_registry_text = fs::read_to_string(&bridge_registry).unwrap();
        let host_bridge_plan_index_text = fs::read_to_string(&host_bridge_plan_index).unwrap();
        let lowering_plan_index_text = fs::read_to_string(&lowering_plan_index).unwrap();
        let bridge_registry_path_text = bridge_registry.display().to_string();
        let host_bridge_plan_index_path_text = host_bridge_plan_index.display().to_string();
        let lowering_plan_index_path_text = lowering_plan_index.display().to_string();
        assert_eq!(
            report.bridge_registry_path.as_deref(),
            Some(bridge_registry_path_text.as_str())
        );
        assert_eq!(
            report.host_bridge_plan_index_path.as_deref(),
            Some(host_bridge_plan_index_path_text.as_str())
        );
        assert_eq!(
            report.lowering_plan_index_path.as_deref(),
            Some(lowering_plan_index_path_text.as_str())
        );
        assert!(bridge_registry_text.contains("schema = \"nuis-bridge-registry-v1\""));
        assert!(bridge_registry_text.contains("bridge_count = 2"));
        assert!(bridge_registry_text.contains("[[bridge]]"));
        assert!(bridge_registry_text.contains("domain_family = \"kernel\""));
        assert!(bridge_registry_text.contains("domain_family = \"network\""));
        assert!(bridge_registry_text.contains("backend_family = \"coreml\""));
        assert!(bridge_registry_text.contains("vendor = \"apple\""));
        assert!(bridge_registry_text.contains("device_class = \"apple-ane\""));
        assert!(bridge_registry_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
        assert!(bridge_registry_text.contains("backend_family = \"urlsession\""));
        assert!(bridge_registry_text.contains("device_class = \"socket-io\""));
        assert!(
            bridge_registry_text.contains("selected_lowering_target = \"urlsession.socket-io\"")
        );
        assert!(bridge_registry_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
        assert!(bridge_registry_text.contains("host_ffi_bridge = \"cffi.network.dispatch.v1\""));
        assert!(bridge_registry_text.contains("host_ffi_policy = \"signature-whitelist-required\""));
        assert!(bridge_registry_text
            .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
        assert!(bridge_registry_text
            .contains("host_ffi_symbol = \"nuis_network_urlsession_socket_io_dispatch_v1\""));
        assert!(bridge_registry_text.contains("host_ffi_signature_hash = \"0x"));
        assert!(bridge_registry_text.contains("bridge_stub_path = "));
        assert!(host_bridge_plan_index_text.contains("schema = \"nuis-host-bridge-plan-index-v1\""));
        assert!(host_bridge_plan_index_text.contains("plan_count = 2"));
        assert!(host_bridge_plan_index_text.contains("[[plan]]"));
        assert!(host_bridge_plan_index_text.contains("domain_family = \"kernel\""));
        assert!(host_bridge_plan_index_text.contains("domain_family = \"network\""));
        assert!(host_bridge_plan_index_text.contains("backend_family = \"coreml\""));
        assert!(host_bridge_plan_index_text.contains("vendor = \"apple\""));
        assert!(host_bridge_plan_index_text.contains("device_class = \"apple-ane\""));
        assert!(
            host_bridge_plan_index_text.contains("selected_lowering_target = \"coreml.apple-ane\"")
        );
        assert!(host_bridge_plan_index_text.contains("backend_family = \"urlsession\""));
        assert!(host_bridge_plan_index_text.contains("device_class = \"socket-io\""));
        assert!(host_bridge_plan_index_text
            .contains("selected_lowering_target = \"urlsession.socket-io\""));
        assert!(
            host_bridge_plan_index_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\"")
        );
        assert!(
            host_bridge_plan_index_text.contains("host_ffi_bridge = \"cffi.network.dispatch.v1\"")
        );
        assert!(host_bridge_plan_index_text
            .contains("host_ffi_policy = \"signature-whitelist-required\""));
        assert!(host_bridge_plan_index_text
            .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
        assert!(host_bridge_plan_index_text
            .contains("host_ffi_symbol = \"nuis_network_urlsession_socket_io_dispatch_v1\""));
        assert!(host_bridge_plan_index_text.contains("host_ffi_signature_hash = \"0x"));
        assert!(host_bridge_plan_index_text
            .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
        assert!(
            lowering_plan_index_text.contains("schema = \"nuis-domain-lowering-plan-index-v1\"")
        );
        assert!(lowering_plan_index_text.contains("plan_count = 2"));
        assert!(lowering_plan_index_text.contains("[[lowering_plan]]"));
        assert!(lowering_plan_index_text.contains("domain_family = \"kernel\""));
        assert!(lowering_plan_index_text.contains("domain_family = \"network\""));
        assert!(
            lowering_plan_index_text.contains("selected_lowering_target = \"coreml.apple-ane\"")
        );
        assert!(lowering_plan_index_text
            .contains("selected_lowering_target = \"urlsession.socket-io\""));
        assert!(lowering_plan_index_text.contains("execution_route = \"ane-graph-execution\""));
        assert!(
            lowering_plan_index_text.contains("execution_route = \"foundation-session-reactor\"")
        );
        assert!(lowering_plan_index_text
            .contains("symbol_namespace = \"nuis::domain::kernel::coreml_apple_ane\""));
        assert!(lowering_plan_index_text
            .contains("debug_anchor = \"nuis.debug.kernel.coreml_apple_ane\""));
        assert!(lowering_plan_index_text
            .contains("linkage_anchor = \"nuis.link.kernel.coreml_apple_ane\""));
        assert!(lowering_plan_index_text.contains(
            "source_map_scope = \"domain:kernel/package:official.kernel/target:coreml.apple-ane\""
        ));
        assert!(lowering_plan_index_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
        assert!(
            lowering_plan_index_text.contains("host_ffi_policy = \"signature-whitelist-required\"")
        );
        assert!(lowering_plan_index_text
            .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
        assert!(lowering_plan_index_text.contains(
            "host_ffi_signature = \"fn(payload: ptr, payload_len: usize, bridge_state: ptr) -> i64\""
        ));
        assert!(lowering_plan_index_text.contains("host_ffi_signature_hash = \"0x"));
        assert!(lowering_plan_index_text
            .contains("symbol_namespace = \"nuis::domain::network::urlsession_socket_io\""));
        assert!(lowering_plan_index_text
            .contains("debug_anchor = \"nuis.debug.network.urlsession_socket_io\""));
        assert!(lowering_plan_index_text.contains("ir_sidecar_path = "));
        assert!(lowering_plan_index_text.contains("payload_blob_path = "));
        assert!(lowering_plan_index_text.contains("bridge_stub_path = "));
        let kernel_blob =
            super::decode_domain_build_unit_payload_blob(&fs::read(&kernel_payload_blob).unwrap())
                .unwrap();
        let network_blob =
            super::decode_domain_build_unit_payload_blob(&fs::read(&network_payload_blob).unwrap())
                .unwrap();
        let kernel_lowering_plan = super::render_domain_build_unit_lowering_plan(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "kernel")
                .unwrap(),
        );
        let kernel_backend_stub = super::render_domain_build_unit_backend_stub(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "kernel")
                .unwrap(),
        );
        let kernel_ir_sidecar = super::render_domain_build_unit_kernel_ir_sidecar(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "kernel")
                .unwrap(),
        );
        let kernel_bridge_plan = super::render_domain_build_unit_bridge_plan(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "kernel")
                .unwrap(),
        );
        let network_lowering_plan = super::render_domain_build_unit_lowering_plan(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "network")
                .unwrap(),
        );
        let network_backend_stub = super::render_domain_build_unit_backend_stub(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "network")
                .unwrap(),
        );
        let network_ir_sidecar = super::render_domain_build_unit_network_ir_sidecar(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "network")
                .unwrap(),
        );
        let network_bridge_plan = super::render_domain_build_unit_bridge_plan(
            report
                .domain_build_units
                .iter()
                .find(|unit| unit.domain_family == "network")
                .unwrap(),
        );
        assert!(kernel_payload_text.contains("schema = \"nuis-domain-build-payload-v1\""));
        assert!(kernel_payload_text.contains("support_surface = ["));
        assert!(kernel_payload_text.contains("default_lanes = ["));
        assert!(kernel_payload_text.contains("resource_families = ["));
        assert!(kernel_payload_text.contains("lowering_targets = ["));
        assert!(kernel_payload_text.contains("ops = ["));
        assert!(network_payload_text.contains("schema = \"nuis-domain-build-payload-v1\""));
        assert!(network_payload_text.contains("host_ffi_surface = ["));
        assert!(network_payload_text.contains("clock_bridge_default = "));
        assert_eq!(kernel_blob.domain_family, "kernel");
        assert_eq!(kernel_blob.package_id, "official.kernel");
        assert_eq!(kernel_blob.backend_family.as_deref(), Some("coreml"));
        assert_eq!(
            kernel_blob.selected_lowering_target.as_deref(),
            Some("coreml.apple-ane")
        );
        assert_eq!(kernel_blob.contract_family, "nustar.kernel");
        assert_eq!(kernel_blob.packaging_role, "hetero-contract");
        assert_eq!(kernel_blob.payload_kind, "contract-sidecar");
        assert_eq!(kernel_blob.payload_format, "toml");
        assert_eq!(kernel_blob.sections.len(), 5);
        assert_eq!(kernel_blob.sections[0].name, "contract_toml");
        assert_eq!(
            kernel_blob.sections[0].bytes,
            kernel_payload_text.as_bytes()
        );
        assert_eq!(kernel_blob.sections[1].name, "lowering_plan");
        assert_eq!(
            kernel_blob.sections[1].bytes,
            kernel_lowering_plan.as_bytes()
        );
        assert_eq!(kernel_blob.sections[2].name, "backend_stub");
        assert_eq!(
            kernel_blob.sections[2].bytes,
            kernel_backend_stub.as_bytes()
        );
        assert_eq!(kernel_blob.sections[3].name, "bridge_plan");
        assert_eq!(kernel_blob.sections[3].bytes, kernel_bridge_plan.as_bytes());
        assert_eq!(kernel_blob.sections[4].name, "kernel_ir_sidecar");
        assert_eq!(kernel_blob.sections[4].bytes, kernel_ir_sidecar.as_bytes());
        let kernel_backend_text = std::str::from_utf8(&kernel_blob.sections[2].bytes).unwrap();
        let kernel_bridge_text = std::str::from_utf8(&kernel_blob.sections[3].bytes).unwrap();
        let kernel_sidecar_text = std::str::from_utf8(&kernel_blob.sections[4].bytes).unwrap();
        assert!(kernel_bridge_stub_text.contains("schema = \"nuis-host-bridge-spec-v1\""));
        assert!(kernel_bridge_stub_text.contains("vendor = \"apple\""));
        assert!(kernel_bridge_stub_text.contains("device_class = \"apple-ane\""));
        assert!(kernel_bridge_stub_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
        assert!(kernel_bridge_stub_text
            .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
        assert!(kernel_bridge_stub_text.contains("host_ffi_surface = \"buffer,queue,fence\""));
        assert!(
            kernel_bridge_stub_text.contains("handle_family = \"kernel.buffer,kernel.dispatch\"")
        );
        assert!(kernel_bridge_stub_text.contains(
            "phase_submit_inputs = [\"dispatch.handle\", \"bound.buffer.table\", \"queue.slot\"]"
        ));
        assert!(kernel_bridge_stub_text.contains("phase_wait_wake = \"completion-fence\""));
        assert!(kernel_bridge_stub_text.contains("bridge_plan_begin = true"));
        assert!(kernel_bridge_stub_text.contains("bridge_plan_end = true"));
        assert!(kernel_bridge_stub_text.contains("phase_submit = \"queue-dispatch-submit\""));
        assert!(kernel_backend_text.contains("stub_kind = \"kernel-dispatch\""));
        assert!(kernel_backend_text.contains("dispatch_shape = \"grid-launch\""));
        assert!(kernel_backend_text.contains("memory_binding = \"buffer-table\""));
        assert!(kernel_backend_text.contains("completion_model = \"device-fence\""));
        assert!(kernel_backend_text.contains("scheduler_binding = \"hetero-submit-bridge\""));
        assert!(kernel_backend_text.contains("backend_profile = \"coreml.apple-ane\""));
        assert!(kernel_backend_text.contains("execution_route = \"ane-graph-execution\""));
        assert!(kernel_backend_text.contains("submission_adapter = \"coreml-graph-submit\""));
        assert!(kernel_backend_text.contains("wake_adapter = \"coreml-completion-callback\""));
        assert!(kernel_backend_text
            .contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
        assert!(kernel_backend_text.contains("kernel_ir = \"coreml-program\""));
        assert!(kernel_backend_text.contains("kernel_entry_model = \"mlmodelc-function\""));
        assert!(kernel_backend_text.contains("queue_binding_model = \"ane-submission-service\""));
        assert!(kernel_backend_text.contains("resource_binding_model = \"tensor-argument-table\""));
        assert!(kernel_sidecar_text.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
        assert!(kernel_sidecar_text
            .contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
        assert!(kernel_sidecar_text.contains("graph = \"infer_main\""));
        assert!(kernel_backend_text.contains("bind_phase = \"buffer-and-argument-bind\""));
        assert!(kernel_backend_text.contains("launch_phase = \"queue-dispatch-submit\""));
        assert!(kernel_backend_text.contains("wait_phase = \"fence-await-or-poll\""));
        assert!(kernel_backend_text.contains("finalize_phase = \"result-commit-and-release\""));
        assert!(kernel_bridge_text.contains("bridge_kind = \"managed-lifecycle-bridge\""));
        assert!(kernel_bridge_text.contains("phase_bind = \"buffer-and-argument-bind\""));
        assert!(kernel_bridge_text.contains("phase_submit = \"queue-dispatch-submit\""));
        assert!(kernel_bridge_text.contains("phase_wait = \"fence-await-or-poll\""));
        assert!(kernel_bridge_text.contains("phase_finalize = \"result-commit-and-release\""));
        assert_eq!(network_blob.domain_family, "network");
        assert_eq!(network_blob.package_id, "official.network");
        assert_eq!(network_blob.backend_family.as_deref(), Some("urlsession"));
        assert_eq!(
            network_blob.selected_lowering_target.as_deref(),
            Some("urlsession.socket-io")
        );
        assert_eq!(network_blob.contract_family, "nustar.network");
        assert_eq!(network_blob.packaging_role, "hetero-contract");
        assert_eq!(network_blob.payload_kind, "contract-sidecar");
        assert_eq!(network_blob.payload_format, "toml");
        assert_eq!(network_blob.sections.len(), 5);
        assert_eq!(network_blob.sections[0].name, "contract_toml");
        assert_eq!(
            network_blob.sections[0].bytes,
            network_payload_text.as_bytes()
        );
        assert_eq!(network_blob.sections[1].name, "lowering_plan");
        assert_eq!(
            network_blob.sections[1].bytes,
            network_lowering_plan.as_bytes()
        );
        assert_eq!(network_blob.sections[2].name, "backend_stub");
        assert_eq!(
            network_blob.sections[2].bytes,
            network_backend_stub.as_bytes()
        );
        assert_eq!(network_blob.sections[3].name, "bridge_plan");
        assert_eq!(
            network_blob.sections[3].bytes,
            network_bridge_plan.as_bytes()
        );
        assert_eq!(network_blob.sections[4].name, "network_ir_sidecar");
        assert_eq!(
            network_blob.sections[4].bytes,
            network_ir_sidecar.as_bytes()
        );
        let network_backend_text = std::str::from_utf8(&network_blob.sections[2].bytes).unwrap();
        let network_bridge_text = std::str::from_utf8(&network_blob.sections[3].bytes).unwrap();
        let network_sidecar_text = std::str::from_utf8(&network_blob.sections[4].bytes).unwrap();
        assert!(network_bridge_stub_text.contains("schema = \"nuis-host-bridge-spec-v1\""));
        assert!(network_bridge_stub_text.contains("vendor = \"apple\""));
        assert!(network_bridge_stub_text.contains("device_class = \"socket-io\""));
        assert!(network_bridge_stub_text
            .contains("selected_lowering_target = \"urlsession.socket-io\""));
        assert!(network_bridge_stub_text
            .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
        assert!(network_bridge_stub_text.contains("host_ffi_surface = \"socket,urlsession\""));
        assert!(network_bridge_stub_text
            .contains("handle_family = \"network.request,network.response\""));
        assert!(network_bridge_stub_text.contains(
            "phase_submit_inputs = [\"session.handle\", \"request.handle\", \"request.packet\"]"
        ));
        assert!(network_bridge_stub_text.contains("phase_wait_wake = \"io-ready\""));
        assert!(network_bridge_stub_text.contains("bridge_plan_begin = true"));
        assert!(network_bridge_stub_text.contains("bridge_plan_end = true"));
        assert!(network_bridge_stub_text.contains("phase_submit = \"packet-write-dispatch\""));
        assert!(network_backend_text.contains("stub_kind = \"network-host-bridge\""));
        assert!(network_backend_text.contains("transport_model = \"client-session\""));
        assert!(network_backend_text.contains("request_shape = \"packetized-exchange\""));
        assert!(network_backend_text.contains("response_shape = \"completion-callback\""));
        assert!(network_backend_text.contains("scheduler_binding = \"network-poll-bridge\""));
        assert!(network_backend_text.contains("backend_profile = \"urlsession.socket-io\""));
        assert!(network_backend_text.contains("execution_route = \"foundation-session-reactor\""));
        assert!(network_backend_text.contains("submission_adapter = \"urlsession-task-submit\""));
        assert!(network_backend_text.contains("wake_adapter = \"urlsession-callback\""));
        assert!(network_backend_text.contains("transport_ir = \"foundation-url-request\""));
        assert!(network_backend_text.contains("transport_entry_model = \"urlsession-task\""));
        assert!(network_backend_text.contains("socket_binding_model = \"session-owned-socket\""));
        assert!(network_backend_text.contains("completion_binding_model = \"delegate-callback\""));
        assert!(network_backend_text.contains("connect_phase = \"socket-bind-or-session-open\""));
        assert!(network_backend_text.contains("send_phase = \"packet-write-dispatch\""));
        assert!(network_backend_text.contains("recv_phase = \"callback-or-read-ready\""));
        assert!(network_backend_text.contains("finalize_phase = \"response-commit-and-wake\""));
        assert!(network_bridge_text.contains("bridge_kind = \"managed-lifecycle-bridge\""));
        assert!(network_bridge_text.contains("phase_bind = \"socket-bind-or-session-open\""));
        assert!(network_bridge_text.contains("phase_submit = \"packet-write-dispatch\""));
        assert!(network_bridge_text.contains("phase_wait = \"callback-or-read-ready\""));
        assert!(network_bridge_text.contains("phase_finalize = \"response-commit-and-wake\""));
        assert!(network_sidecar_text.contains("schema = \"nuis-network-ir-sidecar-v1\""));
        assert!(network_sidecar_text.contains("request = \"http-client-session\""));
        assert!(network_sidecar_text.contains("response = \"completion-callback\""));
        assert!(network_sidecar_text.contains("streaming = \"delegate-push-stream\""));
        assert!(network_sidecar_text.contains("connect = \"open_session\""));
        assert!(network_sidecar_text.contains("finalize = \"finish_exchange\""));
        assert!(report
            .domain_build_units
            .iter()
            .any(|unit| unit.domain_family == "cpu"
                && unit.packaging_role == "host-binary"
                && unit.artifact_stub_path.is_none()
                && unit.selected_lowering_target.as_deref() == Some("llvm")));
        assert!(report
            .domain_build_units
            .iter()
            .any(|unit| unit.domain_family == "kernel"
                && unit
                    .artifact_stub_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.kernel.artifact.toml"))
                && unit
                    .artifact_payload_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.kernel.payload.toml"))
                && unit
                    .artifact_bridge_stub_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.kernel.bridge.stub.txt"))
                && unit
                    .artifact_ir_sidecar_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.kernel.lowering.ir.txt"))
                && unit
                    .artifact_payload_blob_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.kernel.payload.bin"))
                && unit
                    .artifact_payload_blob_bytes
                    .is_some_and(|bytes| bytes > 0)
                && unit.artifact_payload_format.as_deref() == Some("ndpb-v2")
                && unit.backend_family.as_deref() == Some("coreml")
                && unit.selected_lowering_target.as_deref() == Some("coreml.apple-ane")));
        assert!(report
            .domain_build_units
            .iter()
            .any(|unit| unit.domain_family == "network"
                && unit
                    .artifact_stub_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.network.artifact.toml"))
                && unit
                    .artifact_payload_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.network.payload.toml"))
                && unit
                    .artifact_bridge_stub_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.network.bridge.stub.txt"))
                && unit
                    .artifact_ir_sidecar_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.network.lowering.ir.txt"))
                && unit
                    .artifact_payload_blob_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("nuis.domain.network.payload.bin"))
                && unit
                    .artifact_payload_blob_bytes
                    .is_some_and(|bytes| bytes > 0)
                && unit.artifact_payload_format.as_deref() == Some("ndpb-v2")
                && unit.backend_family.as_deref() == Some("urlsession")
                && unit.selected_lowering_target.as_deref() == Some("urlsession.socket-io")));
    }

    #[test]
    fn verify_compiled_artifact_preserves_heterogeneous_domain_unit_paths() {
        let dir = temp_dir("verify_compiled_artifact_heterogeneous_units");
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
            packaging_mode: "window-aot-bundle".to_owned(),
        };
        let cpu_target = CpuBuildTarget {
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
            machine_arch: "arm64".to_owned(),
            machine_os: "darwin".to_owned(),
            object_format: "macho".to_owned(),
            calling_abi: "apple_aapcs64".to_owned(),
            clang_target: "arm64-apple-darwin".to_owned(),
            isa_family: "aarch64".to_owned(),
            isa_features: vec!["a64".to_owned(), "neon".to_owned()],
            cross_compile: false,
        };
        super::write_build_manifest(
            &dir,
            &written,
            &BuildManifestContext {
                input_path: "/tmp/hetero_artifact.ns".to_owned(),
                output_dir: dir.display().to_string(),
                loaded_nustar: vec![
                    "official.cpu".to_owned(),
                    "official.kernel".to_owned(),
                    "official.network".to_owned(),
                ],
                compile_cache: None,
                project: Some(BuildManifestProjectInfo {
                    name: "hetero_artifact".to_owned(),
                    abi_mode: "explicit".to_owned(),
                    abi_graph_summary: None,
                    abi_entries: vec![
                        ("cpu".to_owned(), cpu_target.abi.clone()),
                        ("kernel".to_owned(), "kernel.apple_ane.coreml.v1".to_owned()),
                        (
                            "network".to_owned(),
                            "network.socket.macos.arm64.v1".to_owned(),
                        ),
                    ],
                    plan_summary: None,
                    effective_input: None,
                    text_handle_rewrite_helper_hits: 0,
                    text_handle_rewrite_local_hits: 0,
                    manifest_copy_path: None,
                    plan_index_path: None,
                    organization_index_path: None,
                    exchange_index_path: None,
                    modules_index_path: None,
                    docs_index_path: None,
                    docs_module_count: 0,
                    docs_documented_module_count: 0,
                    docs_documented_item_count: 0,
                    imports_index_path: None,
                    imports_library_count: 0,
                    imports_visible_library_count: 0,
                    imports_visible_module_count: 0,
                    imports_documented_visible_module_count: 0,
                    imports_documented_visible_item_count: 0,
                    galaxy_index_path: None,
                    galaxy_count: 0,
                    galaxy_documented_count: 0,
                    galaxy_documented_library_module_count: 0,
                    galaxy_documented_item_count: 0,
                    links_index_path: None,
                    packet_index_path: None,
                    host_ffi_index_path: None,
                    abi_index_path: None,
                }),
                doc_index: None,
                cpu_target,
            },
        )
        .unwrap();

        fs::remove_file(dir.join("nuis.bridge.registry.toml")).unwrap();
        fs::remove_file(dir.join("nuis.host-bridge.plan-index.toml")).unwrap();
        fs::remove_file(dir.join("nuis.domain.kernel.payload.toml")).unwrap();
        fs::remove_file(dir.join("nuis.domain.kernel.payload.bin")).unwrap();
        fs::remove_file(dir.join("nuis.domain.kernel.bridge.stub.txt")).unwrap();
        fs::remove_file(dir.join("nuis.domain.network.payload.toml")).unwrap();
        fs::remove_file(dir.join("nuis.domain.network.payload.bin")).unwrap();
        fs::remove_file(dir.join("nuis.domain.network.bridge.stub.txt")).unwrap();

        let artifact_report = verify_nuis_compiled_artifact(
            PathBuf::from(&dir).join("nuis.compiled.artifact").as_path(),
        )
        .unwrap();
        assert!(artifact_report.lifecycle_contract_consistent);
        assert!(artifact_report.lifecycle_runtime_capability_flags_consistent);
        assert!(artifact_report.artifact_roundtrip_verified);
    }

    #[test]
    fn c_shim_source_includes_native_cli_runtime_hooks() {
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
                    name: "host_argv_count".to_owned(),
                    params: Vec::new(),
                    return_type: i64_ty(),
                    host_symbol: None,
                },
                AstExternFunction {
                    visibility: AstVisibility::Private,
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_deserialize_text_ends_with".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "suffix_handle".to_owned(),
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
                    name: "host_cwd_handle".to_owned(),
                    params: Vec::new(),
                    return_type: i64_ty(),
                    host_symbol: None,
                },
                AstExternFunction {
                    visibility: AstVisibility::Private,
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_monotonic_time_ns".to_owned(),
                    params: Vec::new(),
                    return_type: i64_ty(),
                    host_symbol: None,
                },
                AstExternFunction {
                    visibility: AstVisibility::Private,
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_deserialize_bool_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_parse_header_line".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "expected_name_handle".to_owned(),
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
                    name: "host_find_header_value".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "expected_name_handle".to_owned(),
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
                    name: "host_find_status_line_reason".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_parse_http_response_summary".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_parse_http_request_summary".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_parse_http_roundtrip_summary".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "request_buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "request_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "request_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "response_buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "response_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "response_len".to_owned(),
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
                    name: "host_deserialize_text_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_deserialize_bool_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_deserialize_text_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_deserialize_bool_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_deserialize_text_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_deserialize_bool_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_deserialize_text_from".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_deserialize_text_equals".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "expected_handle".to_owned(),
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
                    name: "host_deserialize_text_starts_with".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "prefix_handle".to_owned(),
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
                    name: "host_deserialize_text_equals".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "expected_handle".to_owned(),
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
                    name: "host_deserialize_text_starts_with".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "prefix_handle".to_owned(),
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
                    name: "host_deserialize_text_equals".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "expected_handle".to_owned(),
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
                    name: "host_deserialize_text_starts_with".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "prefix_handle".to_owned(),
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
                    name: "host_deserialize_text_contains".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "needle_handle".to_owned(),
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
                    name: "host_buffer_find_byte".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "needle".to_owned(),
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
                    name: "host_fill_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "value".to_owned(),
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
                    name: "host_copy_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "dst_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_len".to_owned(),
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
                    name: "host_compare_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "lhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_len".to_owned(),
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
                    name: "host_buffer_find_text".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "needle_handle".to_owned(),
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
                    name: "host_buffer_find_text".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "needle_handle".to_owned(),
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
                    name: "host_buffer_find_text".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "needle_handle".to_owned(),
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
                    name: "host_buffer_find_line_end".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_buffer_trim_line_end".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
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
                    name: "host_fill_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "value".to_owned(),
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
                    name: "host_copy_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "dst_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_len".to_owned(),
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
                    name: "host_compare_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "lhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_len".to_owned(),
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
                    name: "host_fill_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "value".to_owned(),
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
                    name: "host_copy_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "dst_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_len".to_owned(),
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
                    name: "host_compare_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "lhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_len".to_owned(),
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
                    name: "host_fill_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "value".to_owned(),
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
                    name: "host_copy_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "dst_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_len".to_owned(),
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
                    name: "host_compare_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "lhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_len".to_owned(),
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
                    name: "host_fill_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "buffer_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "value".to_owned(),
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
                    name: "host_copy_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "dst_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "dst_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "src_len".to_owned(),
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
                    name: "host_compare_bytes".to_owned(),
                    params: vec![
                        nuis_semantics::model::AstParam {
                            name: "lhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "lhs_len".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_handle".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_offset".to_owned(),
                            ty: i64_ty(),
                        },
                        nuis_semantics::model::AstParam {
                            name: "rhs_len".to_owned(),
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
        assert!(shim.contains("int main(int argc, char** argv)"));
        assert!(shim.contains("nuis_argc = argc;"));
        assert!(shim.contains("static int64_t nuis_lifecycle_network_enabled = 0;"));
        assert!(shim.contains("static int64_t nuis_lifecycle_hetero_enabled = 0;"));
        assert!(shim.contains("static int64_t nuis_lifecycle_hetero_surface_slots = 0;"));
        assert!(shim.contains("static int64_t nuis_lifecycle_bootstrap_entry_v1(void)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_tick_once_v1(void)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_shutdown_v1(int64_t status)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_yalivia_rpc_hook_v1(void)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_on_bridge_bind_v1(void)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_on_scheduler_tick_v1(int64_t tick)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_on_task_poll_v1(void)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_on_result_commit_v1(int64_t status)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_on_summary_flush_v1(void)"));
        assert!(
            shim.contains("static int64_t nuis_lifecycle_sample_network_bridge_progress_v1(void)")
        );
        assert!(shim
            .contains("static int64_t nuis_lifecycle_sample_hetero_submission_progress_v1(void)"));
        assert!(shim.contains("static int64_t nuis_lifecycle_on_network_bridge_progress_v1(void)"));
        assert!(
            shim.contains("static int64_t nuis_lifecycle_on_hetero_submission_progress_v1(void)")
        );
        assert!(shim.contains("static int64_t nuis_lifecycle_on_managed_rpc_v1(void)"));
        assert!(
            shim.contains("static int64_t nuis_lifecycle_on_shutdown_prepare_v1(int64_t status)")
        );
        assert!(shim.contains("int64_t nuis_lifecycle_bootstrap_export_v1(void) {"));
        assert!(shim.contains("return nuis_lifecycle_bootstrap_entry_v1();"));
        assert!(shim.contains("int64_t nuis_lifecycle_tick_export_v1(void) {"));
        assert!(shim.contains("return nuis_lifecycle_tick_once_v1();"));
        assert!(shim.contains("int64_t nuis_lifecycle_shutdown_export_v1(int64_t status) {"));
        assert!(shim.contains("return nuis_lifecycle_shutdown_v1(status);"));
        assert!(shim.contains("int64_t nuis_lifecycle_yalivia_rpc_export_v1(void) {"));
        assert!(shim.contains("return nuis_lifecycle_yalivia_rpc_hook_v1();"));
        assert!(shim.contains("int64_t nuis_lifecycle_network_bridge_progress_export_v1(void) {"));
        assert!(shim.contains("return nuis_lifecycle_state.network_bridge_progress_count;"));
        assert!(
            shim.contains("int64_t nuis_lifecycle_hetero_submission_progress_export_v1(void) {")
        );
        assert!(shim.contains("return nuis_lifecycle_state.hetero_submission_progress_count;"));
        assert!(shim.contains("if (nuis_lifecycle_bootstrap_entry_v1() != 0) {"));
        assert!(shim.contains("(void)nuis_lifecycle_on_bridge_bind_v1();"));
        assert!(shim.contains("(void)nuis_lifecycle_on_managed_rpc_v1();"));
        assert!(shim.contains("int64_t entry_status = nuis_yir_entry();"));
        assert!(shim.contains("(void)nuis_lifecycle_tick_once_v1();"));
        assert!(shim.contains(
            "(void)nuis_lifecycle_on_scheduler_tick_v1(nuis_lifecycle_state.tick_count);"
        ));
        assert!(shim.contains("(void)nuis_lifecycle_on_task_poll_v1();"));
        assert!(shim.contains("(void)nuis_lifecycle_on_network_bridge_progress_v1();"));
        assert!(shim.contains("(void)nuis_lifecycle_on_hetero_submission_progress_v1();"));
        assert!(shim.contains("(void)nuis_lifecycle_on_result_commit_v1(status);"));
        assert!(shim.contains("(void)nuis_lifecycle_on_summary_flush_v1();"));
        assert!(shim.contains("(void)nuis_lifecycle_on_shutdown_prepare_v1(status);"));
        assert!(shim.contains("return (int)nuis_lifecycle_shutdown_v1(entry_status);"));
        assert!(shim.contains("return nuis_host_argv_count();"));
        assert!(shim.contains("return nuis_host_cwd_handle();"));
        assert!(shim.contains("return nuis_host_monotonic_time_ns();"));
    }

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
        assert!(shim
            .contains("return nuis_host_network_send_owned(handle, stream_window, send_window);"));
        assert!(shim
            .contains("return nuis_host_network_recv_owned(handle, stream_window, recv_window);"));
        assert!(shim.contains(
            "return nuis_host_network_recv_http_status_owned(handle, stream_window, recv_window);"
        ));
        assert!(shim.contains(
            "return nuis_host_network_send_probe(stream_window, send_window, remote_port);"
        ));
        assert!(shim.contains(
            "return nuis_host_network_recv_probe(stream_window, recv_window, local_port);"
        ));
    }

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

    #[test]
    fn c_shim_source_includes_native_serialization_hooks() {
        fn i64_ty() -> AstTypeRef {
            AstTypeRef {
                name: "i64".to_owned(),
                generic_args: Vec::new(),
                is_optional: false,
                is_ref: false,
            }
        }

        fn host_extern(name: &str, params: &[&str]) -> AstExternFunction {
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: name.to_owned(),
                params: params
                    .iter()
                    .map(|param| nuis_semantics::model::AstParam {
                        name: (*param).to_owned(),
                        ty: i64_ty(),
                    })
                    .collect(),
                return_type: i64_ty(),
                host_symbol: None,
            }
        }

        let ast = AstModule {
            attributes: Vec::new(),
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![
                host_extern(
                    "host_serialize_text_into",
                    &["text_handle", "buffer_handle", "offset"],
                ),
                host_extern(
                    "host_serialize_i64_into",
                    &["value", "buffer_handle", "offset"],
                ),
                host_extern(
                    "host_serialize_bool_into",
                    &["value", "buffer_handle", "offset"],
                ),
                host_extern(
                    "host_serialize_byte_into",
                    &["value", "buffer_handle", "offset"],
                ),
                host_extern(
                    "host_deserialize_i64_from",
                    &["buffer_handle", "offset", "len"],
                ),
                host_extern("host_deserialize_byte_from", &["buffer_handle", "offset"]),
                host_extern(
                    "host_deserialize_bool_from",
                    &["buffer_handle", "offset", "len"],
                ),
                host_extern(
                    "host_deserialize_text_from",
                    &["buffer_handle", "offset", "len"],
                ),
                host_extern(
                    "host_fill_bytes",
                    &["buffer_handle", "offset", "len", "value"],
                ),
                host_extern(
                    "host_copy_bytes",
                    &[
                        "dst_handle",
                        "dst_offset",
                        "dst_len",
                        "src_handle",
                        "src_offset",
                        "src_len",
                    ],
                ),
                host_extern(
                    "host_compare_bytes",
                    &[
                        "lhs_handle",
                        "lhs_offset",
                        "lhs_len",
                        "rhs_handle",
                        "rhs_offset",
                        "rhs_len",
                    ],
                ),
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
        assert!(shim.contains("static int64_t nuis_host_serialize_text_into("));
        assert!(shim.contains("static int64_t nuis_host_serialize_i64_into("));
        assert!(shim.contains("static int64_t nuis_host_serialize_bool_into("));
        assert!(shim.contains("static int64_t nuis_host_serialize_byte_into("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_i64_from("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_byte_from("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_bool_from("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_text_from("));
        assert!(shim.contains("static int64_t nuis_host_parse_header_line("));
        assert!(shim.contains("static int64_t nuis_host_find_header_value("));
        assert!(shim.contains("static int64_t nuis_host_find_status_line_reason("));
        assert!(shim.contains("static int64_t nuis_host_parse_http_response_summary("));
        assert!(shim.contains("static int64_t nuis_host_parse_http_request_summary("));
        assert!(shim.contains("static int64_t nuis_host_parse_http_roundtrip_summary("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_text_equals("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_text_starts_with("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_text_contains("));
        assert!(shim.contains("static int64_t nuis_host_deserialize_text_ends_with("));
        assert!(shim.contains("static int64_t nuis_host_buffer_find_byte("));
        assert!(shim.contains("static int64_t nuis_host_fill_bytes("));
        assert!(shim.contains("static int64_t nuis_host_copy_bytes("));
        assert!(shim.contains("static int64_t nuis_host_compare_bytes("));
        assert!(shim.contains("static int64_t nuis_host_buffer_find_text("));
        assert!(shim.contains("static int64_t nuis_host_buffer_find_line_end("));
        assert!(shim.contains("static int64_t nuis_host_buffer_trim_line_end("));
        assert!(shim
            .contains("return nuis_host_serialize_text_into(text_handle, buffer_handle, offset);"));
        assert!(shim.contains("return nuis_host_serialize_i64_into(value, buffer_handle, offset);"));
        assert!(
            shim.contains("return nuis_host_serialize_bool_into(value, buffer_handle, offset);")
        );
        assert!(
            shim.contains("return nuis_host_serialize_byte_into(value, buffer_handle, offset);")
        );
        assert!(shim.contains("return nuis_host_deserialize_i64_from(buffer_handle, offset, len);"));
        assert!(shim.contains("return nuis_host_deserialize_byte_from(buffer_handle, offset);"));
        assert!(shim.contains("return nuis_host_deserialize_bool_from("));
        assert!(shim.contains("return nuis_host_deserialize_text_from("));
        assert!(shim.contains("return nuis_host_fill_bytes("));
        assert!(shim.contains("return nuis_host_copy_bytes("));
        assert!(shim.contains("return nuis_host_compare_bytes("));
    }

    #[test]
    fn c_shim_source_leaves_plain_system_externs_unstubbed() {
        fn ty(name: &str) -> AstTypeRef {
            AstTypeRef {
                name: name.to_owned(),
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
                name: "usleep".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "usec".to_owned(),
                    ty: ty("i64"),
                }],
                return_type: ty("i32"),
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
        assert!(!shim.contains("int32_t usleep("));
    }

    #[test]
    fn c_shim_source_includes_exported_main_wrapper() {
        fn ty(name: &str) -> AstTypeRef {
            AstTypeRef {
                name: name.to_owned(),
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
            externs: Vec::new(),
            extern_interfaces: Vec::new(),
            consts: Vec::new(),
            type_aliases: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            functions: vec![nuis_semantics::model::AstFunction {
                name: "main".to_owned(),
                visibility: nuis_semantics::model::AstVisibility::Private,
                attributes: vec![nuis_semantics::model::AstAttribute {
                    name: "export".to_owned(),
                    args: vec![nuis_semantics::model::AstAttributeArg {
                        name: Some("name".to_owned()),
                        value: nuis_semantics::model::AstAttributeValue::String(
                            "entry_main".to_owned(),
                        ),
                    }],
                }],
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                benchmark_name: None,
                benchmark_warmup_iters: None,
                benchmark_measure_iters: None,
                benchmark_timeout_ms: None,
                benchmark_clock_domain: None,
                benchmark_clock_policy: None,
                is_async: false,
                generic_params: Vec::new(),
                where_bounds: Vec::new(),
                params: Vec::new(),
                return_type: Some(ty("i64")),
                body: Vec::new(),
            }],
        };

        let shim = c_shim_source(&ast);
        assert!(shim.contains("int64_t entry_main(void) {"));
        assert!(shim.contains("return nuis_yir_entry();"));
    }
}
