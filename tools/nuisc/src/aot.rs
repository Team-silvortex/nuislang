use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_artifact::{
    decode_domain_payload_blob as shared_decode_domain_payload_blob,
    decode_nuis_compiled_artifact_binary as shared_decode_nuis_compiled_artifact_binary,
    decode_nuis_executable_envelope_binary as shared_decode_nuis_executable_envelope_binary,
    encode_domain_payload_blob as shared_encode_domain_payload_blob,
    encode_nuis_compiled_artifact_binary as shared_encode_nuis_compiled_artifact_binary,
    encode_nuis_executable_envelope_binary as shared_encode_nuis_executable_envelope_binary,
    parse_domain_build_unit_blocks as shared_parse_domain_build_unit_blocks,
    parse_nuis_compiled_artifact as shared_parse_nuis_compiled_artifact,
    parse_nuis_executable_envelope as shared_parse_nuis_executable_envelope,
    parse_nuis_executable_envelope_from_source as shared_parse_nuis_executable_envelope_from_source,
    render_nuis_executable_envelope as shared_render_nuis_executable_envelope,
    write_nuis_compiled_artifact as shared_write_nuis_compiled_artifact,
    write_nuis_executable_envelope as shared_write_nuis_executable_envelope,
};
use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef, NirModule};
use yir_core::YirModule;

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
    BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob, DomainBuildUnitPayloadBlobSection,
    NuisCompiledArtifact, NuisExecutableEnvelope, NuisLifecycleContract,
};

fn validate_artifact_binary_name(field: &str, value: &str, context: &Path) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty() || path.components().count() != 1 {
        return Err(format!(
            "`{}` has unsafe {field} `{}`; expected a single file name",
            context.display(),
            value
        ));
    }
    match path.components().next() {
        Some(Component::Normal(_)) => Ok(()),
        _ => Err(format!(
            "`{}` has unsafe {field} `{}`; expected a plain file name",
            context.display(),
            value
        )),
    }
}

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

fn validate_manifest_path_in_output_dir(
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

pub fn validate_nuis_compiled_artifact_layout(
    path: &Path,
    artifact: &NuisCompiledArtifact,
) -> Result<(), String> {
    validate_artifact_binary_name("binary_name", &artifact.binary_name, path)
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

pub fn render_nuis_executable_envelope(envelope: &NuisExecutableEnvelope) -> String {
    shared_render_nuis_executable_envelope(envelope)
}

pub fn encode_nuis_executable_envelope_binary(
    envelope: &NuisExecutableEnvelope,
) -> Result<Vec<u8>, String> {
    shared_encode_nuis_executable_envelope_binary(envelope).map_err(|error| error.to_string())
}

pub fn decode_nuis_executable_envelope_binary(
    bytes: &[u8],
) -> Result<NuisExecutableEnvelope, String> {
    shared_decode_nuis_executable_envelope_binary(bytes).map_err(|error| error.to_string())
}

pub fn write_nuis_executable_envelope(
    path: &Path,
    envelope: &NuisExecutableEnvelope,
) -> Result<(), String> {
    shared_write_nuis_executable_envelope(path, envelope).map_err(|error| error.to_string())
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

pub fn encode_nuis_compiled_artifact_binary(
    artifact: &NuisCompiledArtifact,
) -> Result<Vec<u8>, String> {
    shared_encode_nuis_compiled_artifact_binary(artifact).map_err(|error| error.to_string())
}

pub fn decode_nuis_compiled_artifact_binary(bytes: &[u8]) -> Result<NuisCompiledArtifact, String> {
    shared_decode_nuis_compiled_artifact_binary(bytes).map_err(|error| error.to_string())
}

pub fn write_nuis_compiled_artifact(
    path: &Path,
    artifact: &NuisCompiledArtifact,
) -> Result<(), String> {
    shared_write_nuis_compiled_artifact(path, artifact).map_err(|error| error.to_string())
}

pub fn parse_nuis_compiled_artifact(path: &Path) -> Result<NuisCompiledArtifact, String> {
    shared_parse_nuis_compiled_artifact(path).map_err(|error| error.to_string())
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
        if line == "[bridge_registry]" || line == "[host_bridge_plan_index]" {
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

fn render_domain_build_unit_manifest_block(unit: &BuildManifestDomainBuildUnit) -> String {
    let mut out = String::new();
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
    out.push('\n');
    out
}

fn append_relocated_bridge_registry_manifest_section(
    out: &mut String,
    bridge_registry_path: Option<&Path>,
    units: &[BuildManifestDomainBuildUnit],
) {
    let Some(bridge_registry_path) = bridge_registry_path else {
        return;
    };
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let source = render_domain_bridge_registry(&hetero_units);
    out.push_str("[bridge_registry]\n");
    out.push_str(&format!(
        "bridge_registry_path = \"{}\"\n",
        escape_toml_string(&bridge_registry_path.display().to_string())
    ));
    out.push_str("bridge_registry_schema = \"nuis-bridge-registry-v1\"\n");
    out.push_str(&format!("bridge_registry_units = {}\n", hetero_units.len()));
    out.push_str(&format!(
        "bridge_registry_inline = \"{}\"\n",
        escape_toml_string(&source)
    ));
    out.push('\n');
}

fn append_relocated_host_bridge_plan_index_manifest_section(
    out: &mut String,
    host_bridge_plan_index_path: Option<&Path>,
    units: &[BuildManifestDomainBuildUnit],
) {
    let Some(host_bridge_plan_index_path) = host_bridge_plan_index_path else {
        return;
    };
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let source = render_host_bridge_plan_index(&hetero_units);
    out.push_str("[host_bridge_plan_index]\n");
    out.push_str(&format!(
        "host_bridge_plan_index_path = \"{}\"\n",
        escape_toml_string(&host_bridge_plan_index_path.display().to_string())
    ));
    out.push_str("host_bridge_plan_index_schema = \"nuis-host-bridge-plan-index-v1\"\n");
    out.push_str(&format!(
        "host_bridge_plan_units = {}\n",
        hetero_units.len()
    ));
    out.push_str(&format!(
        "host_bridge_plan_index_inline = \"{}\"\n",
        escape_toml_string(&source)
    ));
    out.push('\n');
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

fn render_domain_bridge_registry(units: &[&BuildManifestDomainBuildUnit]) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-bridge-registry-v1\"\n");
    out.push_str(&format!("bridge_count = {}\n", units.len()));
    let domains = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<Vec<_>>();
    out.push_str(&format!("domains = {}\n", render_string_array(&domains)));
    for unit in units {
        out.push('\n');
        out.push_str("[[bridge]]\n");
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "vendor = \"{}\"\n",
            escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "device_class = \"{}\"\n",
            escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "selected_lowering_target = \"{}\"\n",
            escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "bridge_stub_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_bridge_stub_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&format!(
            "payload_blob_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_payload_blob_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&render_domain_build_unit_bridge_plan(unit));
    }
    out
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

fn render_host_bridge_plan_index(units: &[&BuildManifestDomainBuildUnit]) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-host-bridge-plan-index-v1\"\n");
    out.push_str(&format!("plan_count = {}\n", units.len()));
    let domains = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<Vec<_>>();
    out.push_str(&format!("domains = {}\n", render_string_array(&domains)));
    for unit in units {
        let contract = domain_build_contract_summary_for_unit(unit);
        out.push('\n');
        out.push_str("[[plan]]\n");
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "vendor = \"{}\"\n",
            escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "device_class = \"{}\"\n",
            escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "selected_lowering_target = \"{}\"\n",
            escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "bridge_stub_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_bridge_stub_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&format!(
            "bridge_surface = \"{}\"\n",
            escape_toml_string(&contract.bridge.bridge_surface)
        ));
        out.push_str(&format!(
            "scheduler_binding = \"{}\"\n",
            escape_toml_string(&contract.bridge.scheduler_binding)
        ));
        out.push_str(&format!(
            "phase_order = {}\n",
            render_string_array(&contract.host_bridge.phase_order)
        ));
        out.push_str(&format!(
            "plan_inline = \"{}\"\n",
            escape_toml_string(&render_domain_build_unit_bridge_plan(unit).replace('\n', "\\n"))
        ));
    }
    out
}

fn render_domain_build_unit_stub(unit: &BuildManifestDomainBuildUnit) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-build-unit-v1\"\n");
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
    out
}

fn render_domain_build_unit_payload(unit: &BuildManifestDomainBuildUnit) -> Result<String, String> {
    let manifest = crate::registry::load_manifest_for_domain(
        Path::new("nustar-packages"),
        &unit.domain_family,
    )?;
    let capability = crate::registry::capability_summary(&manifest);
    let execution = crate::registry::execution_summary(&manifest);
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-build-payload-v1\"\n");
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
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str("payload_kind = \"contract-sidecar\"\n");
    out.push_str("payload_format = \"toml\"\n");
    out.push_str(&format!(
        "frontend = \"{}\"\n",
        escape_toml_string(&manifest.frontend)
    ));
    out.push_str(&format!(
        "entry_crate = \"{}\"\n",
        escape_toml_string(&manifest.entry_crate)
    ));
    out.push_str(&format!(
        "loader_abi = \"{}\"\n",
        escape_toml_string(&manifest.loader_abi)
    ));
    out.push_str(&format!(
        "loader_entry = \"{}\"\n",
        escape_toml_string(&manifest.loader_entry)
    ));
    out.push_str(&format!(
        "clock_domain_id = \"{}\"\n",
        escape_toml_string(&capability.clock.domain_id)
    ));
    out.push_str(&format!(
        "clock_kind = \"{}\"\n",
        escape_toml_string(&capability.clock.kind)
    ));
    out.push_str(&format!(
        "clock_epoch_kind = \"{}\"\n",
        escape_toml_string(&capability.clock.epoch_kind)
    ));
    out.push_str(&format!(
        "clock_resolution = \"{}\"\n",
        escape_toml_string(&capability.clock.resolution)
    ));
    out.push_str(&format!(
        "clock_bridge_default = \"{}\"\n",
        escape_toml_string(&capability.clock.bridge_default)
    ));
    out.push_str(&format!(
        "execution_skeleton_version = \"{}\"\n",
        escape_toml_string(&execution.skeleton_version)
    ));
    out.push_str(&format!(
        "execution_function_kind = \"{}\"\n",
        escape_toml_string(&execution.function_kind)
    ));
    out.push_str(&format!(
        "execution_graph_kind = \"{}\"\n",
        escape_toml_string(&execution.graph_kind)
    ));
    out.push_str(&format!(
        "execution_default_time_mode = \"{}\"\n",
        escape_toml_string(&execution.default_time_mode)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out.push_str(&format!(
        "support_surface = {}\n",
        render_string_array(&capability.support_surface)
    ));
    out.push_str(&format!(
        "support_profile_slots = {}\n",
        render_string_array(&capability.support_profile_slots)
    ));
    out.push_str(&format!(
        "default_lanes = {}\n",
        render_string_array(&capability.default_lanes)
    ));
    out.push_str(&format!(
        "resource_families = {}\n",
        render_string_array(&manifest.resource_families)
    ));
    out.push_str(&format!(
        "unit_types = {}\n",
        render_string_array(&manifest.unit_types)
    ));
    out.push_str(&format!(
        "lowering_targets = {}\n",
        render_string_array(&execution.lowering_targets)
    ));
    out.push_str(&format!("ops = {}\n", render_string_array(&manifest.ops)));
    out.push_str(&format!(
        "host_ffi_surface = {}\n",
        render_string_array(&manifest.host_ffi_surface)
    ));
    out.push_str(&format!(
        "host_ffi_abis = {}\n",
        render_string_array(&manifest.host_ffi_abis)
    ));
    if !manifest.host_ffi_bridge.is_empty() {
        out.push_str(&format!(
            "host_ffi_bridge = \"{}\"\n",
            escape_toml_string(&manifest.host_ffi_bridge)
        ));
    }
    Ok(out)
}

fn encode_domain_build_unit_payload_blob(
    unit: &BuildManifestDomainBuildUnit,
    payload_path: &Path,
) -> Result<Vec<u8>, String> {
    let payload = fs::read(payload_path)
        .map_err(|error| format!("failed to read `{}`: {error}", payload_path.display()))?;
    let mut sections = vec![
        DomainBuildUnitPayloadBlobSection {
            name: "contract_toml".to_owned(),
            bytes: payload,
        },
        DomainBuildUnitPayloadBlobSection {
            name: "lowering_plan".to_owned(),
            bytes: render_domain_build_unit_lowering_plan(unit).into_bytes(),
        },
        DomainBuildUnitPayloadBlobSection {
            name: "backend_stub".to_owned(),
            bytes: render_domain_build_unit_backend_stub(unit).into_bytes(),
        },
        DomainBuildUnitPayloadBlobSection {
            name: "bridge_plan".to_owned(),
            bytes: render_domain_build_unit_bridge_plan(unit).into_bytes(),
        },
    ];
    if unit.domain_family == "shader" {
        sections.push(DomainBuildUnitPayloadBlobSection {
            name: "shader_ir_sidecar".to_owned(),
            bytes: render_domain_build_unit_shader_ir_sidecar(unit).into_bytes(),
        });
    } else if unit.domain_family == "kernel" {
        sections.push(DomainBuildUnitPayloadBlobSection {
            name: "kernel_ir_sidecar".to_owned(),
            bytes: render_domain_build_unit_kernel_ir_sidecar(unit).into_bytes(),
        });
    } else if unit.domain_family == "network" {
        sections.push(DomainBuildUnitPayloadBlobSection {
            name: "network_ir_sidecar".to_owned(),
            bytes: render_domain_build_unit_network_ir_sidecar(unit).into_bytes(),
        });
    }
    let blob = DomainBuildUnitPayloadBlob::from_domain_unit_and_sections(unit, sections);
    shared_encode_domain_payload_blob(&blob).map_err(|error| error.to_string())
}

fn decode_domain_build_unit_payload_blob(
    bytes: &[u8],
) -> Result<DomainBuildUnitPayloadBlob, String> {
    shared_decode_domain_payload_blob(bytes).map_err(|error| error.to_string())
}

fn domain_build_contract_summary_for_unit(
    unit: &BuildManifestDomainBuildUnit,
) -> crate::registry::NustarDomainBuildContractSummary {
    match crate::registry::load_manifest(Path::new("nustar-packages"), &unit.package_id) {
        Ok(manifest) => crate::registry::domain_build_contract_summary(&manifest),
        Err(_) => crate::registry::domain_build_contract_summary_for_domain(&unit.domain_family),
    }
}

struct DerivedLoweringProfile<'a> {
    profile_key: &'a str,
    execution_route: &'static str,
    submission_adapter: &'static str,
    wake_adapter: &'static str,
}

fn derived_lowering_profile_for_unit<'a>(
    unit: &'a BuildManifestDomainBuildUnit,
) -> DerivedLoweringProfile<'a> {
    let profile_key = unit
        .selected_lowering_target
        .as_deref()
        .or(unit.backend_family.as_deref())
        .unwrap_or("none");
    let (execution_route, submission_adapter, wake_adapter) =
        match (unit.domain_family.as_str(), profile_key) {
            ("shader", "metal.apple-silicon-gpu") => (
                "unified-render-graph",
                "metal-command-encoder",
                "metal-shared-event",
            ),
            ("shader", "metal.mac-discrete-or-integrated-gpu") => (
                "render-graph-device-queue",
                "metal-command-buffer",
                "metal-frame-fence",
            ),
            ("shader", "vulkan.discrete-or-integrated-gpu") => (
                "spirv-render-queue",
                "vulkan-command-buffer",
                "vulkan-timeline-semaphore",
            ),
            ("shader", "directx.discrete-or-integrated-gpu") => {
                ("dxil-render-queue", "directx-command-list", "directx-fence")
            }
            ("shader", "opengl.discrete-or-integrated-gpu") => (
                "driver-managed-render-pipeline",
                "opengl-driver-submit",
                "gl-sync-object",
            ),
            ("shader", "cpu-fallback.cpu-host") => (
                "host-render-fallback",
                "cpu-raster-dispatch",
                "host-frame-complete",
            ),
            ("kernel", "coreml.apple-ane") => (
                "ane-graph-execution",
                "coreml-graph-submit",
                "coreml-completion-callback",
            ),
            ("kernel", "vulkan.discrete-or-integrated-gpu") => (
                "spirv-compute-queue",
                "vulkan-compute-submit",
                "vulkan-timeline-semaphore",
            ),
            ("kernel", "cpu-fallback.cpu-host") => (
                "host-kernel-fallback",
                "cpu-threadpool-dispatch",
                "host-join-wake",
            ),
            ("network", "urlsession.socket-io") => (
                "foundation-session-reactor",
                "urlsession-task-submit",
                "urlsession-callback",
            ),
            ("network", "winsock.socket-io") => (
                "windows-socket-reactor",
                "winsock-overlapped-submit",
                "iocp-ready",
            ),
            ("network", "socket-abi.socket-io") => (
                "posix-socket-reactor",
                "socket-send-recv-submit",
                "poll-ready",
            ),
            ("shader", _) => (
                "generic-render-dispatch",
                "render-submit-bridge",
                "frame-present",
            ),
            ("kernel", _) => (
                "generic-accelerator-dispatch",
                "hetero-submit-bridge",
                "completion-fence",
            ),
            ("network", _) => ("generic-io-reactor", "network-poll-bridge", "io-ready"),
            ("cpu", _) | ("host", _) => ("host-inline-call", "direct-call", "immediate"),
            _ => ("generic-dispatch", "generic-submit", "generic-wake"),
        };
    DerivedLoweringProfile {
        profile_key,
        execution_route,
        submission_adapter,
        wake_adapter,
    }
}

fn render_target_specific_backend_fields(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> String {
    let mut out = String::new();
    match (unit.domain_family.as_str(), profile.profile_key) {
        ("shader", "metal.apple-silicon-gpu") => {
            out.push_str("shader_ir = \"msl2.4\"\n");
            out.push_str("shader_entry_model = \"metal-function-constant-specialized\"\n");
            out.push_str("queue_binding_model = \"unified-command-queue\"\n");
            out.push_str("resource_binding_model = \"argument-buffer-table\"\n");
        }
        ("shader", "metal.mac-discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"msl2.4\"\n");
            out.push_str("shader_entry_model = \"metal-pipeline-state\"\n");
            out.push_str("queue_binding_model = \"device-command-queue\"\n");
            out.push_str("resource_binding_model = \"descriptor-table-emulated\"\n");
        }
        ("shader", "vulkan.discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"spirv1.6\"\n");
            out.push_str("shader_entry_model = \"vulkan-pipeline\"\n");
            out.push_str("queue_binding_model = \"explicit-device-queue\"\n");
            out.push_str("resource_binding_model = \"descriptor-set-layout\"\n");
        }
        ("shader", "directx.discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"dxil6.8\"\n");
            out.push_str("shader_entry_model = \"directx-pipeline-state\"\n");
            out.push_str("queue_binding_model = \"command-queue\"\n");
            out.push_str("resource_binding_model = \"root-signature\"\n");
        }
        ("shader", "opengl.discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"glsl460\"\n");
            out.push_str("shader_entry_model = \"driver-linked-program\"\n");
            out.push_str("queue_binding_model = \"driver-managed\"\n");
            out.push_str("resource_binding_model = \"uniform-slot-table\"\n");
        }
        ("shader", "cpu-fallback.cpu-host") => {
            out.push_str("shader_ir = \"host-simd\"\n");
            out.push_str("shader_entry_model = \"cpu-raster-kernel\"\n");
            out.push_str("queue_binding_model = \"threadpool-work-queue\"\n");
            out.push_str("resource_binding_model = \"host-buffer-slices\"\n");
        }
        ("kernel", "coreml.apple-ane") => {
            out.push_str("kernel_ir = \"coreml-program\"\n");
            out.push_str("kernel_entry_model = \"mlmodelc-function\"\n");
            out.push_str("queue_binding_model = \"ane-submission-service\"\n");
            out.push_str("resource_binding_model = \"tensor-argument-table\"\n");
        }
        ("kernel", "vulkan.discrete-or-integrated-gpu") => {
            out.push_str("kernel_ir = \"spirv1.6\"\n");
            out.push_str("kernel_entry_model = \"compute-pipeline\"\n");
            out.push_str("queue_binding_model = \"compute-queue\"\n");
            out.push_str("resource_binding_model = \"descriptor-set-layout\"\n");
        }
        ("kernel", "cpu-fallback.cpu-host") => {
            out.push_str("kernel_ir = \"host-simd\"\n");
            out.push_str("kernel_entry_model = \"threadpool-kernel\"\n");
            out.push_str("queue_binding_model = \"host-work-queue\"\n");
            out.push_str("resource_binding_model = \"host-buffer-slices\"\n");
        }
        ("network", "urlsession.socket-io") => {
            out.push_str("transport_ir = \"foundation-url-request\"\n");
            out.push_str("transport_entry_model = \"urlsession-task\"\n");
            out.push_str("socket_binding_model = \"session-owned-socket\"\n");
            out.push_str("completion_binding_model = \"delegate-callback\"\n");
        }
        ("network", "winsock.socket-io") => {
            out.push_str("transport_ir = \"winsock-overlapped\"\n");
            out.push_str("transport_entry_model = \"iocp-request\"\n");
            out.push_str("socket_binding_model = \"overlapped-socket-handle\"\n");
            out.push_str("completion_binding_model = \"iocp-completion-port\"\n");
        }
        ("network", "socket-abi.socket-io") => {
            out.push_str("transport_ir = \"posix-socket\"\n");
            out.push_str("transport_entry_model = \"poll-reactor-request\"\n");
            out.push_str("socket_binding_model = \"fd-owned-session\"\n");
            out.push_str("completion_binding_model = \"poll-ready-token\"\n");
        }
        _ => {}
    }
    out
}

fn render_target_specific_lowering_fields(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> String {
    let mut out = String::new();
    match (unit.domain_family.as_str(), profile.profile_key) {
        ("shader", "metal.apple-silicon-gpu") => {
            out.push_str("lowering_ir = \"msl2.4\"\n");
            out.push_str("shader_stage_model = \"metal-render-pipeline\"\n");
            out.push_str("stage_binding_model = \"argument-buffer-specialized\"\n");
            out.push_str("dispatch_encoding_model = \"tile-and-threadgroup\"\n");
        }
        ("shader", "metal.mac-discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"msl2.4\"\n");
            out.push_str("shader_stage_model = \"metal-render-pipeline\"\n");
            out.push_str("stage_binding_model = \"descriptor-table-emulated\"\n");
            out.push_str("dispatch_encoding_model = \"device-queue-encoder\"\n");
        }
        ("shader", "vulkan.discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"spirv1.6\"\n");
            out.push_str("shader_stage_model = \"spirv-graphics-pipeline\"\n");
            out.push_str("stage_binding_model = \"descriptor-set-layout\"\n");
            out.push_str("dispatch_encoding_model = \"renderpass-command-buffer\"\n");
        }
        ("shader", "directx.discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"dxil6.8\"\n");
            out.push_str("shader_stage_model = \"dxil-pso\"\n");
            out.push_str("stage_binding_model = \"root-signature\"\n");
            out.push_str("dispatch_encoding_model = \"command-list-recording\"\n");
        }
        ("shader", "opengl.discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"glsl460\"\n");
            out.push_str("shader_stage_model = \"linked-program-pipeline\"\n");
            out.push_str("stage_binding_model = \"uniform-slot-table\"\n");
            out.push_str("dispatch_encoding_model = \"driver-issued-draw\"\n");
        }
        ("shader", "cpu-fallback.cpu-host") => {
            out.push_str("lowering_ir = \"host-simd\"\n");
            out.push_str("shader_stage_model = \"cpu-raster-pipeline\"\n");
            out.push_str("stage_binding_model = \"host-buffer-slices\"\n");
            out.push_str("dispatch_encoding_model = \"threadpool-tile-dispatch\"\n");
        }
        ("network", "urlsession.socket-io") => {
            out.push_str("lowering_ir = \"foundation-url-request\"\n");
            out.push_str("transport_binding_model = \"session-task-packet\"\n");
            out.push_str("completion_encoding_model = \"delegate-callback-lifecycle\"\n");
        }
        ("network", "winsock.socket-io") => {
            out.push_str("lowering_ir = \"winsock-overlapped\"\n");
            out.push_str("transport_binding_model = \"overlapped-packet-reactor\"\n");
            out.push_str("completion_encoding_model = \"iocp-completion-lifecycle\"\n");
        }
        ("network", "socket-abi.socket-io") => {
            out.push_str("lowering_ir = \"posix-socket\"\n");
            out.push_str("transport_binding_model = \"packet-poll-reactor\"\n");
            out.push_str("completion_encoding_model = \"poll-ready-lifecycle\"\n");
        }
        _ => {}
    }
    out
}

fn shader_supported_stages_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match (unit.domain_family.as_str(), profile.profile_key) {
        (
            "shader",
            "metal.apple-silicon-gpu"
            | "metal.mac-discrete-or-integrated-gpu"
            | "vulkan.discrete-or-integrated-gpu"
            | "directx.discrete-or-integrated-gpu"
            | "opengl.discrete-or-integrated-gpu"
            | "cpu-fallback.cpu-host",
        ) => Some(&["vertex", "fragment", "compute"]),
        ("shader", _) => Some(&["fragment"]),
        _ => None,
    }
}

fn kernel_supported_dispatch_kinds_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match (unit.domain_family.as_str(), profile.profile_key) {
        ("kernel", "coreml.apple-ane") => Some(&["graph", "batch", "tile"]),
        ("kernel", "vulkan.discrete-or-integrated-gpu") => Some(&["grid", "indirect", "batch"]),
        ("kernel", "cpu-fallback.cpu-host") => Some(&["range", "tile", "batch"]),
        ("kernel", _) => Some(&["graph"]),
        _ => None,
    }
}

fn render_domain_build_unit_shader_ir_sidecar(unit: &BuildManifestDomainBuildUnit) -> String {
    let profile = derived_lowering_profile_for_unit(unit);
    let supported_stages = shader_supported_stages_for_profile(unit, &profile).unwrap_or(&[]);
    let has_stage = |stage: &str| supported_stages.contains(&stage);
    let mut out = String::new();
    out.push_str("schema = \"nuis-shader-ir-sidecar-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    if !supported_stages.is_empty() {
        out.push_str(&format!(
            "supported_stages = {}\n",
            render_string_array(
                &supported_stages
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    out.push_str(&render_target_specific_lowering_fields(unit, &profile));
    match profile.profile_key {
        "metal.apple-silicon-gpu" | "metal.mac-discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.msl\"\n");
            out.push_str("entry_symbol = \"main0\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"argument-buffer\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("threadgroup_topology = \"tile\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"material.uniforms, frame.texture0\"\n");
            out.push_str("push_constants = \"fragment.params\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main0\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            out.push_str("header = \"#include <metal_stdlib>\\nusing namespace metal;\"\n");
            if has_stage("vertex") {
                out.push_str(
                    "vertex_body = \"vertex float4 vs_main(uint vid [[vertex_id]]) {\\n    return float4(float(vid & 1), float((vid >> 1) & 1), 0.0, 1.0);\\n}\"\n",
                );
            }
            if has_stage("fragment") {
                out.push_str(
                    "body = \"fragment float4 main0(float2 uv [[stage_in]]) {\\n    return float4(uv.x, uv.y, 0.0, 1.0);\\n}\"\n",
                );
            }
            if has_stage("compute") {
                out.push_str(
                    "compute_body = \"kernel void cs_main(uint2 gid [[thread_position_in_grid]]) {\\n    (void)gid;\\n}\"\n",
                );
            }
        }
        "vulkan.discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.spirv\"\n");
            out.push_str("entry_symbol = \"main\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"descriptor-set\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("threadgroup_topology = \"quad-fragment\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"set0.binding0.texture, set0.binding1.sampler\"\n");
            out.push_str("push_constants = \"fragment.params\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            out.push_str("capabilities = \"OpCapability Shader\"\n");
            if has_stage("vertex") {
                out.push_str("vertex_body = \"OpEntryPoint Vertex %vs_main \\\"vs_main\\\"\"\n");
            }
            if has_stage("fragment") {
                out.push_str(
                    "body = \"OpMemoryModel Logical GLSL450\\nOpEntryPoint Fragment %main \\\"main\\\"\\n%main = OpFunction %void None %fn\\nOpFunctionEnd\"\n",
                );
            }
            if has_stage("compute") {
                out.push_str(
                    "compute_body = \"OpEntryPoint GLCompute %cs_main \\\"cs_main\\\"\"\n",
                );
            }
        }
        "directx.discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.dxil\"\n");
            out.push_str("entry_symbol = \"main\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"root-signature\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"t0.texture, s0.sampler\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            if has_stage("vertex") {
                out.push_str("vertex_body = \"float4 vs_main(uint vid : SV_VertexID) : SV_Position { return float4(0, 0, 0, 1); }\"\n");
            }
            if has_stage("fragment") {
                out.push_str(
                    "body = \"float4 main() : SV_Target0 { return float4(0, 0, 0, 1); }\"\n",
                );
            }
            if has_stage("compute") {
                out.push_str("compute_body = \"[numthreads(8,8,1)] void cs_main(uint3 tid : SV_DispatchThreadID) { }\"\n");
            }
        }
        "opengl.discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.glsl\"\n");
            out.push_str("entry_symbol = \"main\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"uniform-slots\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"sampler0, uniform0\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            out.push_str("header = \"#version 460 core\"\n");
            if has_stage("vertex") {
                out.push_str("vertex_body = \"void vs_main() { gl_Position = vec4(0.0, 0.0, 0.0, 1.0); }\"\n");
            }
            if has_stage("fragment") {
                out.push_str("body = \"out vec4 fragColor;\\nvoid main() { fragColor = vec4(0.0, 0.0, 0.0, 1.0); }\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute_body = \"layout(local_size_x = 8, local_size_y = 8) in;\\nvoid cs_main() { }\"\n");
            }
        }
        "cpu-fallback.cpu-host" => {
            out.push_str("ir_container = \"text.host-simd\"\n");
            out.push_str("entry_symbol = \"shade_stub\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"host-slices\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"host-rgba8\"]\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"tile.buffer, material.slice\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_stub\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"shade_stub\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_stub\"\n");
            }
            out.push_str("[source_stub]\n");
            if has_stage("vertex") {
                out.push_str(
                    "vertex_body = \"fn vs_stub(vid: u32) -> (f32, f32) { (vid as f32, 0.0) }\"\n",
                );
            }
            if has_stage("fragment") {
                out.push_str("body = \"fn shade_stub(tile: u32) -> u32 { tile }\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute_body = \"fn cs_stub(group: u32) -> u32 { group }\"\n");
            }
        }
        _ => {
            out.push_str("ir_container = \"text.unknown\"\n");
            out.push_str("entry_symbol = \"unimplemented\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"unimplemented\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"unimplemented\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"unimplemented\"\n");
            }
            out.push_str("[source_stub]\n");
            if has_stage("fragment") {
                out.push_str("body = \"unimplemented\"\n");
            }
        }
    }
    out
}

fn render_domain_build_unit_kernel_ir_sidecar(unit: &BuildManifestDomainBuildUnit) -> String {
    let profile = derived_lowering_profile_for_unit(unit);
    let dispatch_kinds = kernel_supported_dispatch_kinds_for_profile(unit, &profile).unwrap_or(&[]);
    let mut out = String::new();
    out.push_str("schema = \"nuis-kernel-ir-sidecar-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    if !dispatch_kinds.is_empty() {
        out.push_str(&format!(
            "supported_dispatch_kinds = {}\n",
            render_string_array(
                &dispatch_kinds
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    out.push_str(&render_target_specific_backend_fields(unit, &profile));
    out.push_str("[dispatch_shapes]\n");
    match profile.profile_key {
        "coreml.apple-ane" => {
            out.push_str("primary = \"graph\"\n");
            out.push_str("fallback = \"batch\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"tensor.input0, tensor.output0\"\n");
            out.push_str("argument_model = \"tensor-argument-table\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("graph = \"infer_main\"\n");
            out.push_str("batch = \"infer_batch\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("graph_body = \"program infer_main(tensor<1x4xf32> input) -> tensor<1x4xf32> { return input; }\"\n");
            out.push_str(
                "batch_body = \"batch infer_batch(count: i32) { /* coreml batch stub */ }\"\n",
            );
        }
        "vulkan.discrete-or-integrated-gpu" => {
            out.push_str("primary = \"grid\"\n");
            out.push_str("fallback = \"indirect\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"set0.buffer0, set0.buffer1\"\n");
            out.push_str("argument_model = \"descriptor-set-layout\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("grid = \"main\"\n");
            out.push_str("indirect = \"main_indirect\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("grid_body = \"OpEntryPoint GLCompute %main \\\"main\\\"\"\n");
            out.push_str(
                "indirect_body = \"OpEntryPoint GLCompute %main_indirect \\\"main_indirect\\\"\"\n",
            );
        }
        "cpu-fallback.cpu-host" => {
            out.push_str("primary = \"range\"\n");
            out.push_str("fallback = \"tile\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"slice.input, slice.output\"\n");
            out.push_str("argument_model = \"host-buffer-slices\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("range = \"run_range\"\n");
            out.push_str("tile = \"run_tile\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("range_body = \"fn run_range(start: u32, end: u32) { }\"\n");
            out.push_str("tile_body = \"fn run_tile(tile: u32) { }\"\n");
        }
        _ => {
            out.push_str("primary = \"graph\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("graph = \"unimplemented\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("graph_body = \"unimplemented\"\n");
        }
    }
    out
}

fn render_domain_build_unit_network_ir_sidecar(unit: &BuildManifestDomainBuildUnit) -> String {
    let profile = derived_lowering_profile_for_unit(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-network-ir-sidecar-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&render_target_specific_lowering_fields(unit, &profile));
    out.push_str(&render_target_specific_backend_fields(unit, &profile));
    out.push_str("[session_shapes]\n");
    match profile.profile_key {
        "urlsession.socket-io" => {
            out.push_str("request = \"http-client-session\"\n");
            out.push_str("response = \"completion-callback\"\n");
            out.push_str("streaming = \"delegate-push-stream\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"session.handle, request.packet, response.slot\"\n");
            out.push_str("argument_model = \"foundation-request-bundle\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"open_session\"\n");
            out.push_str("send = \"submit_request\"\n");
            out.push_str("recv = \"on_response\"\n");
            out.push_str("finalize = \"finish_exchange\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("connect_body = \"fn open_session(authority: text) -> session { session(authority) }\"\n");
            out.push_str("send_body = \"fn submit_request(session: session, request: packet) -> task { task(session, request) }\"\n");
            out.push_str(
                "recv_body = \"fn on_response(task: task) -> response { response(task) }\"\n",
            );
            out.push_str("finalize_body = \"fn finish_exchange(response: response) -> status { commit(response) }\"\n");
        }
        "winsock.socket-io" => {
            out.push_str("request = \"overlapped-client-session\"\n");
            out.push_str("response = \"iocp-completion\"\n");
            out.push_str("streaming = \"completion-port-stream\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"socket.handle, overlapped.packet, completion.port\"\n");
            out.push_str("argument_model = \"iocp-request-bundle\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"connect_overlapped\"\n");
            out.push_str("send = \"submit_overlapped_send\"\n");
            out.push_str("recv = \"await_iocp_completion\"\n");
            out.push_str("finalize = \"finish_iocp_exchange\"\n");
            out.push_str("[source_stub]\n");
            out.push_str(
                "connect_body = \"fn connect_overlapped(addr: text) -> socket { socket(addr) }\"\n",
            );
            out.push_str("send_body = \"fn submit_overlapped_send(socket: socket, packet: packet) -> overlapped { overlapped(socket, packet) }\"\n");
            out.push_str("recv_body = \"fn await_iocp_completion(op: overlapped) -> response { response(op) }\"\n");
            out.push_str("finalize_body = \"fn finish_iocp_exchange(response: response) -> status { finalize(response) }\"\n");
        }
        "socket-abi.socket-io" => {
            out.push_str("request = \"socket-reactor-session\"\n");
            out.push_str("response = \"poll-ready-response\"\n");
            out.push_str("streaming = \"fd-edge-stream\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"fd.handle, packet.buffer, ready.token\"\n");
            out.push_str("argument_model = \"socket-poll-bundle\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"open_fd_session\"\n");
            out.push_str("send = \"submit_send_recv\"\n");
            out.push_str("recv = \"poll_ready_response\"\n");
            out.push_str("finalize = \"finish_poll_exchange\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("connect_body = \"fn open_fd_session(addr: text) -> fd { fd(addr) }\"\n");
            out.push_str("send_body = \"fn submit_send_recv(fd: fd, packet: packet) -> token { token(fd, packet) }\"\n");
            out.push_str("recv_body = \"fn poll_ready_response(token: token) -> response { response(token) }\"\n");
            out.push_str("finalize_body = \"fn finish_poll_exchange(response: response) -> status { release(response) }\"\n");
        }
        _ => {
            out.push_str("request = \"generic-session\"\n");
            out.push_str("response = \"generic-response\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"unimplemented\"\n");
            out.push_str("send = \"unimplemented\"\n");
            out.push_str("recv = \"unimplemented\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("connect_body = \"unimplemented\"\n");
        }
    }
    out
}

fn render_domain_build_unit_lowering_plan(unit: &BuildManifestDomainBuildUnit) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let profile = derived_lowering_profile_for_unit(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-lowering-plan-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "vendor = \"{}\"\n",
        escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "device_class = \"{}\"\n",
        escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&format!(
        "machine_arch = \"{}\"\n",
        escape_toml_string(unit.machine_arch.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "machine_os = \"{}\"\n",
        escape_toml_string(unit.machine_os.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lane_policy = \"{}\"\n",
        escape_toml_string(&contract.lowering.lane_policy)
    ));
    out.push_str(&format!(
        "bridge_surface = \"{}\"\n",
        escape_toml_string(&contract.lowering.bridge_surface)
    ));
    out.push_str(&format!(
        "emission_kind = \"{}\"\n",
        escape_toml_string(&contract.lowering.emission_kind)
    ));
    out.push_str(&format!(
        "execution_route = \"{}\"\n",
        escape_toml_string(profile.execution_route)
    ));
    out.push_str(&format!(
        "submission_adapter = \"{}\"\n",
        escape_toml_string(profile.submission_adapter)
    ));
    out.push_str(&format!(
        "wake_adapter = \"{}\"\n",
        escape_toml_string(profile.wake_adapter)
    ));
    if let Some(dispatch_kinds) = kernel_supported_dispatch_kinds_for_profile(unit, &profile) {
        out.push_str(&format!(
            "supported_dispatch_kinds = {}\n",
            render_string_array(
                &dispatch_kinds
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(stages) = shader_supported_stages_for_profile(unit, &profile) {
        out.push_str(&format!(
            "supported_stages = {}\n",
            render_string_array(&stages.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>())
        ));
    }
    out.push_str(&render_target_specific_lowering_fields(unit, &profile));
    out
}

fn render_domain_build_unit_backend_stub(unit: &BuildManifestDomainBuildUnit) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let backend = contract.backend;
    let profile = derived_lowering_profile_for_unit(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-backend-stub-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "vendor = \"{}\"\n",
        escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "device_class = \"{}\"\n",
        escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "backend_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out.push_str(&format!(
        "machine_arch = \"{}\"\n",
        escape_toml_string(unit.machine_arch.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "machine_os = \"{}\"\n",
        escape_toml_string(unit.machine_os.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "stub_kind = \"{}\"\n",
        escape_toml_string(&backend.stub_kind)
    ));
    out.push_str(&format!(
        "bridge_entry = \"{}\"\n",
        escape_toml_string(&backend.bridge_entry)
    ));
    out.push_str(&format!(
        "submission_mode = \"{}\"\n",
        escape_toml_string(&backend.submission_mode)
    ));
    out.push_str(&format!(
        "wake_policy = \"{}\"\n",
        escape_toml_string(&backend.wake_policy)
    ));
    if let Some(value) = backend.transport_model {
        out.push_str(&format!(
            "transport_model = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.request_shape {
        out.push_str(&format!(
            "request_shape = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.response_shape {
        out.push_str(&format!(
            "response_shape = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.dispatch_shape {
        out.push_str(&format!(
            "dispatch_shape = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.memory_binding {
        out.push_str(&format!(
            "memory_binding = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.resource_binding {
        out.push_str(&format!(
            "resource_binding = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.completion_model {
        out.push_str(&format!(
            "completion_model = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    out.push_str(&format!(
        "scheduler_binding = \"{}\"\n",
        escape_toml_string(&backend.scheduler_binding)
    ));
    out.push_str(&format!(
        "execution_route = \"{}\"\n",
        escape_toml_string(profile.execution_route)
    ));
    out.push_str(&format!(
        "submission_adapter = \"{}\"\n",
        escape_toml_string(profile.submission_adapter)
    ));
    out.push_str(&format!(
        "wake_adapter = \"{}\"\n",
        escape_toml_string(profile.wake_adapter)
    ));
    if let Some(dispatch_kinds) = kernel_supported_dispatch_kinds_for_profile(unit, &profile) {
        out.push_str(&format!(
            "supported_dispatch_kinds = {}\n",
            render_string_array(
                &dispatch_kinds
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    out.push_str(&render_target_specific_backend_fields(unit, &profile));
    if let Some(value) = backend.phase_bind {
        let key = if unit.domain_family == "network" {
            "connect_phase"
        } else {
            "bind_phase"
        };
        out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(&value)));
    }
    if let Some(value) = backend.phase_submit {
        let key = if unit.domain_family == "network" {
            "send_phase"
        } else {
            "launch_phase"
        };
        out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(&value)));
    }
    if let Some(value) = backend.phase_wait {
        let key = if unit.domain_family == "network" {
            "recv_phase"
        } else {
            "wait_phase"
        };
        out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(&value)));
    }
    if let Some(value) = backend.phase_finalize {
        out.push_str(&format!(
            "finalize_phase = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    out
}

fn render_domain_build_unit_bridge_plan(unit: &BuildManifestDomainBuildUnit) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let bridge = contract.bridge;
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-bridge-plan-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "bridge_surface = \"{}\"\n",
        escape_toml_string(&bridge.bridge_surface)
    ));
    out.push_str(&format!(
        "bridge_entry = \"{}\"\n",
        escape_toml_string(&bridge.bridge_entry)
    ));
    out.push_str(&format!(
        "scheduler_binding = \"{}\"\n",
        escape_toml_string(&bridge.scheduler_binding)
    ));
    out.push_str(&format!(
        "phase_bind = \"{}\"\n",
        escape_toml_string(&bridge.phase_bind)
    ));
    out.push_str(&format!(
        "phase_submit = \"{}\"\n",
        escape_toml_string(&bridge.phase_submit)
    ));
    out.push_str(&format!(
        "phase_wait = \"{}\"\n",
        escape_toml_string(&bridge.phase_wait)
    ));
    out.push_str(&format!(
        "phase_finalize = \"{}\"\n",
        escape_toml_string(&bridge.phase_finalize)
    ));
    out.push_str(&format!(
        "bridge_kind = \"{}\"\n",
        escape_toml_string(&bridge.bridge_kind)
    ));
    out
}

fn render_domain_build_unit_host_bridge_stub(unit: &BuildManifestDomainBuildUnit) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let bridge = &contract.bridge;
    let host_bridge = &contract.host_bridge;
    let profile = derived_lowering_profile_for_unit(unit);
    let bridge_plan = render_domain_build_unit_bridge_plan(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-host-bridge-spec-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "vendor = \"{}\"\n",
        escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "device_class = \"{}\"\n",
        escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "bridge_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&format!(
        "bridge_surface = \"{}\"\n",
        escape_toml_string(&bridge.bridge_surface)
    ));
    out.push_str(&format!(
        "bridge_entry = \"{}\"\n",
        escape_toml_string(&bridge.bridge_entry)
    ));
    out.push_str(&format!(
        "scheduler_binding = \"{}\"\n",
        escape_toml_string(&bridge.scheduler_binding)
    ));
    out.push_str(&format!(
        "execution_route = \"{}\"\n",
        escape_toml_string(profile.execution_route)
    ));
    out.push_str(&format!(
        "submission_adapter = \"{}\"\n",
        escape_toml_string(profile.submission_adapter)
    ));
    out.push_str(&format!(
        "wake_adapter = \"{}\"\n",
        escape_toml_string(profile.wake_adapter)
    ));
    out.push_str(&format!(
        "host_ffi_surface = \"{}\"\n",
        escape_toml_string(&host_bridge.host_ffi_surface)
    ));
    out.push_str(&format!(
        "handle_family = \"{}\"\n",
        escape_toml_string(&host_bridge.handle_family)
    ));
    out.push_str(&format!(
        "phase_order = {}\n",
        render_string_array(&host_bridge.phase_order)
    ));
    out.push_str(&format!(
        "phase_bind_inputs = {}\n",
        render_string_array(&host_bridge.phase_bind_inputs)
    ));
    out.push_str(&format!(
        "phase_bind_outputs = {}\n",
        render_string_array(&host_bridge.phase_bind_outputs)
    ));
    out.push_str(&format!(
        "phase_submit_inputs = {}\n",
        render_string_array(&host_bridge.phase_submit_inputs)
    ));
    out.push_str(&format!(
        "phase_submit_outputs = {}\n",
        render_string_array(&host_bridge.phase_submit_outputs)
    ));
    out.push_str(&format!(
        "phase_wait_inputs = {}\n",
        render_string_array(&host_bridge.phase_wait_inputs)
    ));
    out.push_str(&format!(
        "phase_wait_outputs = {}\n",
        render_string_array(&host_bridge.phase_wait_outputs)
    ));
    out.push_str(&format!(
        "phase_finalize_inputs = {}\n",
        render_string_array(&host_bridge.phase_finalize_inputs)
    ));
    out.push_str(&format!(
        "phase_finalize_outputs = {}\n",
        render_string_array(&host_bridge.phase_finalize_outputs)
    ));
    out.push_str(&format!(
        "phase_bind_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_bind_wake)
    ));
    out.push_str(&format!(
        "phase_submit_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_submit_wake)
    ));
    out.push_str(&format!(
        "phase_wait_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_wait_wake)
    ));
    out.push_str(&format!(
        "phase_finalize_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_finalize_wake)
    ));
    out.push_str(&format!(
        "bridge_plan_begin = {}\n",
        if host_bridge.bridge_plan_begin {
            "true"
        } else {
            "false"
        }
    ));
    out.push_str(&bridge_plan);
    if !bridge_plan.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!(
        "bridge_plan_end = {}\n",
        if host_bridge.bridge_plan_end {
            "true"
        } else {
            "false"
        }
    ));
    out
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
    let mut domain_families = execution_contracts
        .iter()
        .map(|item| item.domain_family.clone())
        .collect::<Vec<_>>();
    domain_families.sort();
    domain_families.dedup();

    let mut contract_families = execution_contracts
        .iter()
        .map(|item| item.execution.contract_family.clone())
        .collect::<Vec<_>>();
    contract_families.sort();
    contract_families.dedup();

    let function_kind = execution_contracts
        .first()
        .map(|item| item.execution.function_kind.clone())
        .unwrap_or_else(|| "function-node".to_owned());
    let graph_kind = execution_contracts
        .first()
        .map(|item| item.execution.graph_kind.clone())
        .unwrap_or_else(|| "function-graph".to_owned());
    let default_time_mode = execution_contracts
        .first()
        .map(|item| item.execution.default_time_mode.clone())
        .unwrap_or_else(|| "logical".to_owned());

    NuisExecutableEnvelope {
        schema: "nuis-executable-envelope-v1".to_owned(),
        executable_kind: packaging_mode.to_owned(),
        package_count: execution_contracts.len(),
        domain_families,
        contract_families,
        function_kind,
        graph_kind,
        default_time_mode,
    }
}

fn build_nuis_lifecycle_contract(
    envelope: &NuisExecutableEnvelope,
    packaging_mode: &str,
) -> NuisLifecycleContract {
    let mut hook_surface = vec![
        "on_bridge_bind".to_owned(),
        "on_scheduler_tick".to_owned(),
        "on_task_poll".to_owned(),
        "on_result_commit".to_owned(),
        "on_summary_flush".to_owned(),
        "on_managed_rpc".to_owned(),
        "on_shutdown_prepare".to_owned(),
    ];
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.network")
    {
        hook_surface.push("on_network_bridge_progress".to_owned());
    }
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.shader" || family == "nustar.kernel")
    {
        hook_surface.push("on_hetero_submission_progress".to_owned());
    }
    let mut export_surface = vec![
        "nuis_lifecycle_bootstrap_export_v1".to_owned(),
        "nuis_lifecycle_tick_export_v1".to_owned(),
        "nuis_lifecycle_shutdown_export_v1".to_owned(),
        "nuis_lifecycle_yalivia_rpc_export_v1".to_owned(),
    ];
    let mut runtime_capability_flags = vec![
        "runtime.bootstrap".to_owned(),
        "runtime.tick".to_owned(),
        "runtime.shutdown".to_owned(),
        "runtime.rpc.yalivia".to_owned(),
    ];
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.network")
    {
        export_surface.push("nuis_lifecycle_network_bridge_progress_export_v1".to_owned());
        runtime_capability_flags.push("runtime.progress.network".to_owned());
    }
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.shader" || family == "nustar.kernel")
    {
        export_surface.push("nuis_lifecycle_hetero_submission_progress_export_v1".to_owned());
        runtime_capability_flags.push("runtime.progress.hetero".to_owned());
    }
    NuisLifecycleContract {
        schema: "nuis-lifecycle-contract-v1".to_owned(),
        bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
        tick_policy: if packaging_mode == "native-cpu-llvm" {
            "owned-pump.active-wait-drain".to_owned()
        } else {
            "owned-pump.bootstrap-adaptive".to_owned()
        },
        shutdown_policy: "flush-summaries-then-release-bridges".to_owned(),
        yalivia_rpc: "optional.lifecycle-hook-rpc.v1".to_owned(),
        hook_surface,
        export_surface,
        runtime_capability_flags,
    }
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("0x{hash:016x}")
}

fn hex_encode_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn hex_decode_bytes(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("hex payload length must be even".to_owned());
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let chunk = std::str::from_utf8(&bytes[index..index + 2])
            .map_err(|_| "hex payload is not valid UTF-8".to_owned())?;
        let byte =
            u8::from_str_radix(chunk, 16).map_err(|_| format!("invalid hex byte `{chunk}`"))?;
        out.push(byte);
        index += 2;
    }
    Ok(out)
}

fn artifact_hash_fallback_bytes(
    kind: &str,
    domain_build_units: &[BuildManifestDomainBuildUnit],
    bridge_registry_inline: Option<&str>,
    host_bridge_plan_index_inline: Option<&str>,
) -> Option<Vec<u8>> {
    if kind == "domain_bridge_registry" {
        return bridge_registry_inline.map(|value| value.as_bytes().to_vec());
    }
    if kind == "host_bridge_plan_index" {
        return host_bridge_plan_index_inline.map(|value| value.as_bytes().to_vec());
    }

    let (prefix, domain_family) = [
        ("domain_stub_", "stub"),
        ("domain_payload_", "payload"),
        ("domain_payload_blob_", "payload_blob"),
        ("domain_bridge_stub_", "bridge_stub"),
    ]
    .iter()
    .find_map(|(prefix, kind_name)| kind.strip_prefix(prefix).map(|domain| (*kind_name, domain)))?;

    let unit = domain_build_units
        .iter()
        .find(|unit| unit.domain_family == domain_family)?;

    match prefix {
        "stub" => Some(render_domain_build_unit_stub(unit).into_bytes()),
        "payload" => render_domain_build_unit_payload(unit)
            .ok()
            .map(|value| value.into_bytes()),
        "payload_blob" => unit
            .artifact_payload_blob_inline
            .as_deref()
            .and_then(|value| hex_decode_bytes(value).ok()),
        "bridge_stub" => Some(render_domain_build_unit_host_bridge_stub(unit).into_bytes()),
        _ => None,
    }
}

fn render_string_array(values: &[String]) -> String {
    let quoted = values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml_string(value)))
        .collect::<Vec<_>>();
    format!("[{}]", quoted.join(", "))
}

fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
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

pub struct BuildManifestVerifyReport {
    pub schema: String,
    pub input: String,
    pub output_dir: String,
    pub packaging_mode: String,
    pub envelope_path: String,
    pub envelope_schema: String,
    pub envelope_package_count: usize,
    pub artifact_path: String,
    pub artifact_schema: String,
    pub artifact_binary_name: String,
    pub artifact_binary_bytes: usize,
    pub lifecycle_schema: String,
    pub lifecycle_bootstrap_entry: String,
    pub lifecycle_tick_policy: String,
    pub lifecycle_shutdown_policy: String,
    pub lifecycle_yalivia_rpc: String,
    pub lifecycle_hook_count: usize,
    pub lifecycle_hook_surface: Vec<String>,
    pub lifecycle_export_count: usize,
    pub lifecycle_export_surface: Vec<String>,
    pub lifecycle_runtime_capability_flags: Vec<String>,
    pub execution_contracts_checked: usize,
    pub domain_build_unit_count: usize,
    pub heterogeneous_domain_count: usize,
    pub domain_payload_blobs_checked: usize,
    pub domain_payload_blob_sections_checked: usize,
    pub domain_payload_contract_sections_checked: usize,
    pub domain_payload_lowering_plans_checked: usize,
    pub domain_payload_backend_stubs_checked: usize,
    pub domain_payload_bridge_plans_checked: usize,
    pub domain_bridge_stubs_checked: usize,
    pub domain_build_units: Vec<BuildManifestDomainBuildUnit>,
    pub cpu_target_abi: String,
    pub cpu_target_machine_arch: String,
    pub cpu_target_machine_os: String,
    pub cpu_target_object_format: String,
    pub cpu_target_calling_abi: String,
    pub cpu_target_clang: String,
    pub cpu_target_cross: bool,
    pub loaded_nustar: Vec<String>,
    pub compile_cache_status: Option<String>,
    pub compile_cache_key: Option<String>,
    pub compile_cache_root: Option<String>,
    pub doc_index_path: Option<String>,
    pub doc_index_module_count: usize,
    pub doc_index_documented_item_count: usize,
    pub doc_index_checked: usize,
    pub project_text_handle_rewrite_helper_hits: usize,
    pub project_text_handle_rewrite_local_hits: usize,
    pub project_plan_index: Option<String>,
    pub project_docs_index: Option<String>,
    pub project_docs_module_count: usize,
    pub project_docs_documented_module_count: usize,
    pub project_docs_documented_item_count: usize,
    pub project_imports_index: Option<String>,
    pub project_imports_library_count: usize,
    pub project_imports_visible_library_count: usize,
    pub project_imports_visible_module_count: usize,
    pub project_imports_documented_visible_module_count: usize,
    pub project_imports_documented_visible_item_count: usize,
    pub project_galaxy_index: Option<String>,
    pub project_galaxy_count: usize,
    pub project_documented_galaxy_count: usize,
    pub project_documented_galaxy_library_module_count: usize,
    pub project_documented_galaxy_item_count: usize,
    pub project_packet_index: Option<String>,
    pub bridge_registry_path: Option<String>,
    pub bridge_registry_units: usize,
    pub bridge_registry_checked: usize,
    pub bridge_registry_entries_checked: usize,
    pub host_bridge_plan_index_path: Option<String>,
    pub host_bridge_plan_units: usize,
    pub host_bridge_plan_checked: usize,
    pub host_bridge_plan_entries_checked: usize,
    pub artifacts_checked: usize,
    pub project_metadata_checked: usize,
}

pub struct NuisCompiledArtifactVerifyReport {
    pub schema: String,
    pub packaging_mode: String,
    pub binary_name: String,
    pub binary_bytes: usize,
    pub build_manifest_bytes: usize,
    pub envelope_schema: String,
    pub envelope_package_count: usize,
    pub lifecycle_schema: String,
    pub lifecycle_bootstrap_entry: String,
    pub lifecycle_tick_policy: String,
    pub lifecycle_shutdown_policy: String,
    pub lifecycle_yalivia_rpc: String,
    pub lifecycle_hook_count: usize,
    pub lifecycle_hook_surface: Vec<String>,
    pub lifecycle_export_count: usize,
    pub lifecycle_export_surface: Vec<String>,
    pub lifecycle_runtime_capability_flags: Vec<String>,
    pub lifecycle_contract_consistent: bool,
    pub lifecycle_runtime_capability_flags_consistent: bool,
    pub execution_contracts_checked: usize,
    pub cpu_target_abi: String,
    pub cpu_target_machine_arch: String,
    pub cpu_target_machine_os: String,
    pub cpu_target_object_format: String,
    pub cpu_target_calling_abi: String,
    pub artifact_roundtrip_verified: bool,
}

fn project_metadata_summary_mismatch_error(
    index_kind: &str,
    index_path: &str,
    expected: &str,
    source: &str,
    project_input: &str,
    output_dir: &str,
) -> String {
    let actual = source
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("summary\t"))
        .unwrap_or("<missing summary line>");
    let source_exists = Path::new(project_input).exists();
    let build_manifest_path = Path::new(output_dir)
        .join("nuis.build.manifest.toml")
        .display()
        .to_string();
    let suggestions = if source_exists {
        vec![
            format!("nuisc compile \"{}\" \"{}\"", project_input, output_dir),
            format!("nuisc inspect-project-metadata \"{}\"", project_input),
        ]
    } else {
        vec![
            format!("nuisc inspect-project-metadata \"{}\"", build_manifest_path),
            format!("nuisc verify-build-manifest \"{}\"", build_manifest_path),
        ]
    };
    format!(
        "project {index_kind} index `{index_path}` summary mismatch: expected `{expected}`, found `{actual}`; this usually means the build artifact was produced by an older nuisc metadata format or the index file drifted after compilation. Rebuild the project with the current nuisc, or regenerate the build output before inspecting/verifying it. Suggested commands: {}.",
        suggestions
            .iter()
            .map(|command| format!("`{command}`"))
            .collect::<Vec<_>>()
            .join(" or ")
    )
}

pub fn verify_build_manifest(path: &Path) -> Result<BuildManifestVerifyReport, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let schema = parse_required_toml_string(&source, "manifest_schema", path)?;
    if schema != "nuis-build-manifest-v1" {
        return Err(format!(
            "`{}` has unsupported manifest schema `{}`; expected `nuis-build-manifest-v1`",
            path.display(),
            schema
        ));
    }
    let input = parse_required_toml_string(&source, "input", path)?;
    let output_dir = parse_required_toml_string(&source, "output_dir", path)?;
    let packaging_mode = parse_required_toml_string(&source, "packaging_mode", path)?;
    let envelope_path = parse_required_toml_string(&source, "path", path)?;
    let envelope_schema = parse_required_toml_string(&source, "schema", path)?;
    if envelope_schema != "nuis-executable-envelope-v1" {
        return Err(format!(
            "`{}` has unsupported nuis envelope schema `{}`; expected `nuis-executable-envelope-v1`",
            path.display(),
            envelope_schema
        ));
    }
    let envelope_package_count = parse_required_toml_usize(&source, "package_count", path)?;
    let artifact_path = parse_required_toml_string(&source, "artifact_path", path)?;
    let artifact_schema = parse_required_toml_string(&source, "artifact_schema", path)?;
    if artifact_schema != "nuis-compiled-artifact-v1" {
        return Err(format!(
            "`{}` has unsupported nuis artifact schema `{}`; expected `nuis-compiled-artifact-v1`",
            path.display(),
            artifact_schema
        ));
    }
    let artifact_binary_name = parse_required_toml_string(&source, "artifact_binary_name", path)?;
    validate_artifact_binary_name("artifact_binary_name", &artifact_binary_name, path)?;
    validate_manifest_path_in_output_dir("nuis_envelope.path", &envelope_path, &output_dir, path)?;
    validate_manifest_path_in_output_dir(
        "nuis_artifact.artifact_path",
        &artifact_path,
        &output_dir,
        path,
    )?;
    let artifact_binary_bytes = parse_required_toml_usize(&source, "artifact_binary_bytes", path)?;
    let lifecycle_schema = parse_required_toml_string(&source, "lifecycle_schema", path)?;
    if lifecycle_schema != "nuis-lifecycle-contract-v1" {
        return Err(format!(
            "`{}` has unsupported lifecycle schema `{}`; expected `nuis-lifecycle-contract-v1`",
            path.display(),
            lifecycle_schema
        ));
    }
    let lifecycle_bootstrap_entry =
        parse_required_toml_string(&source, "lifecycle_bootstrap_entry", path)?;
    let lifecycle_tick_policy = parse_required_toml_string(&source, "lifecycle_tick_policy", path)?;
    let lifecycle_shutdown_policy =
        parse_required_toml_string(&source, "lifecycle_shutdown_policy", path)?;
    let lifecycle_yalivia_rpc = parse_required_toml_string(&source, "lifecycle_yalivia_rpc", path)?;
    let lifecycle_hook_surface =
        parse_required_toml_string_array(&source, "lifecycle_hook_surface", path)?;
    let lifecycle_export_surface =
        parse_required_toml_string_array(&source, "lifecycle_export_surface", path)?;
    let lifecycle_runtime_capability_flags =
        parse_required_toml_string_array(&source, "lifecycle_runtime_capability_flags", path)?;
    let envelope_function_kind = parse_required_toml_string(&source, "function_kind", path)?;
    if envelope_function_kind != "function-node" {
        return Err(format!(
            "`{}` has unsupported nuis envelope function_kind `{}`; expected `function-node`",
            path.display(),
            envelope_function_kind
        ));
    }
    let envelope_graph_kind = parse_required_toml_string(&source, "graph_kind", path)?;
    if envelope_graph_kind != "function-graph" {
        return Err(format!(
            "`{}` has unsupported nuis envelope graph_kind `{}`; expected `function-graph`",
            path.display(),
            envelope_graph_kind
        ));
    }
    let envelope_time_mode = parse_required_toml_string(&source, "default_time_mode", path)?;
    if envelope_time_mode.is_empty() {
        return Err(format!(
            "`{}` has empty nuis envelope default_time_mode",
            path.display()
        ));
    }
    let cpu_target_abi = parse_required_toml_string(&source, "cpu_target_abi", path)?;
    let cpu_target_machine_arch =
        parse_required_toml_string(&source, "cpu_target_machine_arch", path)?;
    let cpu_target_machine_os = parse_required_toml_string(&source, "cpu_target_machine_os", path)?;
    let cpu_target_object_format =
        parse_required_toml_string(&source, "cpu_target_object_format", path)?;
    let cpu_target_calling_abi =
        parse_required_toml_string(&source, "cpu_target_calling_abi", path)?;
    let cpu_target_clang = parse_required_toml_string(&source, "cpu_target_clang", path)?;
    let cpu_target_cross = parse_required_toml_bool(&source, "cpu_target_cross", path)?;
    let loaded_nustar = parse_required_toml_string_array(&source, "loaded_nustar", path)?;
    let compile_cache_status = parse_optional_toml_string(&source, "compile_cache_status");
    let compile_cache_key = parse_optional_toml_string(&source, "compile_cache_key");
    let compile_cache_root = parse_optional_toml_string(&source, "compile_cache_root");
    let doc_index_path = parse_optional_toml_string(&source, "doc_index_path");
    let doc_index_module_count =
        parse_optional_toml_usize(&source, "doc_index_module_count").unwrap_or(0);
    let doc_index_documented_item_count =
        parse_optional_toml_usize(&source, "doc_index_documented_item_count").unwrap_or(0);
    let project_text_handle_rewrite_helper_hits =
        parse_optional_toml_usize(&source, "text_handle_rewrite_helper_hits").unwrap_or(0);
    let project_text_handle_rewrite_local_hits =
        parse_optional_toml_usize(&source, "text_handle_rewrite_local_hits").unwrap_or(0);
    let project_plan_index = parse_optional_toml_string(&source, "plan_index");
    let project_docs_index = parse_optional_toml_string(&source, "docs_index");
    let project_docs_module_count =
        parse_optional_toml_usize(&source, "docs_module_count").unwrap_or(0);
    let project_docs_documented_module_count =
        parse_optional_toml_usize(&source, "docs_documented_module_count").unwrap_or(0);
    let project_docs_documented_item_count =
        parse_optional_toml_usize(&source, "docs_documented_item_count").unwrap_or(0);
    let project_imports_index = parse_optional_toml_string(&source, "imports_index");
    let project_imports_library_count =
        parse_optional_toml_usize(&source, "imports_library_count").unwrap_or(0);
    let project_imports_visible_library_count =
        parse_optional_toml_usize(&source, "imports_visible_library_count").unwrap_or(0);
    let project_imports_visible_module_count =
        parse_optional_toml_usize(&source, "imports_visible_module_count").unwrap_or(0);
    let project_imports_documented_visible_module_count =
        parse_optional_toml_usize(&source, "imports_documented_visible_module_count").unwrap_or(0);
    let project_imports_documented_visible_item_count =
        parse_optional_toml_usize(&source, "imports_documented_visible_item_count").unwrap_or(0);
    let project_galaxy_index = parse_optional_toml_string(&source, "galaxy_index");
    let project_galaxy_count = parse_optional_toml_usize(&source, "galaxy_count").unwrap_or(0);
    let project_documented_galaxy_count =
        parse_optional_toml_usize(&source, "documented_galaxy_count").unwrap_or(0);
    let project_documented_galaxy_library_module_count =
        parse_optional_toml_usize(&source, "documented_galaxy_library_module_count").unwrap_or(0);
    let project_documented_galaxy_item_count =
        parse_optional_toml_usize(&source, "documented_galaxy_item_count").unwrap_or(0);
    let project_packet_index = parse_optional_toml_string(&source, "packet_index");
    let bridge_registry_path = parse_optional_toml_string(&source, "bridge_registry_path");
    let bridge_registry_schema = parse_optional_toml_string(&source, "bridge_registry_schema");
    let bridge_registry_units =
        parse_optional_toml_usize(&source, "bridge_registry_units").unwrap_or(0);
    let bridge_registry_inline = parse_optional_toml_string(&source, "bridge_registry_inline");
    let host_bridge_plan_index_path =
        parse_optional_toml_string(&source, "host_bridge_plan_index_path");
    let host_bridge_plan_index_schema =
        parse_optional_toml_string(&source, "host_bridge_plan_index_schema");
    let host_bridge_plan_units =
        parse_optional_toml_usize(&source, "host_bridge_plan_units").unwrap_or(0);
    let host_bridge_plan_index_inline =
        parse_optional_toml_string(&source, "host_bridge_plan_index_inline");
    let project_plan_summary = parse_optional_toml_string(&source, "plan_summary");

    let artifacts = parse_artifact_hash_blocks(&source, path)?;
    if artifacts.is_empty() {
        return Err(format!(
            "`{}` does not contain any `[[artifact_hash]]` blocks",
            path.display()
        ));
    }
    for item in &artifacts {
        validate_manifest_path_in_output_dir("artifact_hash.path", &item.path, &output_dir, path)?;
    }

    let execution_contracts_checked = source
        .lines()
        .filter(|line| line.trim() == "[[execution_contract]]")
        .count();
    if execution_contracts_checked != envelope_package_count {
        return Err(format!(
            "`{}` execution_contract block count mismatch: envelope package_count={}, blocks={}",
            path.display(),
            envelope_package_count,
            execution_contracts_checked
        ));
    }
    let domain_build_units = parse_domain_build_unit_blocks(&source, path)?;
    if domain_build_units.len() != envelope_package_count {
        return Err(format!(
            "`{}` domain_build_unit block count mismatch: envelope package_count={}, blocks={}",
            path.display(),
            envelope_package_count,
            domain_build_units.len()
        ));
    }
    let heterogeneous_domain_count = domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .count();
    let mut domain_payload_blobs_checked = 0usize;
    let mut domain_payload_blob_sections_checked = 0usize;
    let mut domain_payload_contract_sections_checked = 0usize;
    let mut domain_payload_lowering_plans_checked = 0usize;
    let mut domain_payload_backend_stubs_checked = 0usize;
    let mut domain_payload_bridge_plans_checked = 0usize;
    let mut domain_bridge_stubs_checked = 0usize;
    for unit in &domain_build_units {
        for (field, value) in [
            (
                "domain_build_unit.artifact_payload_blob_path",
                unit.artifact_payload_blob_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_stub_path",
                unit.artifact_stub_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_payload_path",
                unit.artifact_payload_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_bridge_stub_path",
                unit.artifact_bridge_stub_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_ir_sidecar_path",
                unit.artifact_ir_sidecar_path.as_deref(),
            ),
        ] {
            if let Some(value) = value {
                validate_manifest_path_in_output_dir(field, value, &output_dir, path)?;
            }
        }
    }
    let parsed_envelope = parse_nuis_executable_envelope(Path::new(&envelope_path))?;
    if parsed_envelope.schema != envelope_schema {
        return Err(format!(
            "`{}` nuis envelope schema mismatch between manifest and `{}`",
            path.display(),
            envelope_path
        ));
    }
    if parsed_envelope.package_count != envelope_package_count {
        return Err(format!(
            "`{}` nuis envelope package_count mismatch between manifest and `{}`",
            path.display(),
            envelope_path
        ));
    }
    if parsed_envelope.executable_kind != packaging_mode {
        return Err(format!(
            "`{}` nuis envelope executable_kind mismatch between manifest and `{}`",
            path.display(),
            envelope_path
        ));
    }
    let parsed_artifact = parse_nuis_compiled_artifact(Path::new(&artifact_path))?;
    if parsed_artifact.schema != artifact_schema {
        return Err(format!(
            "`{}` nuis artifact schema mismatch between manifest and `{}`",
            path.display(),
            artifact_path
        ));
    }
    if parsed_artifact.packaging_mode != packaging_mode {
        return Err(format!(
            "`{}` nuis artifact packaging_mode mismatch between manifest and `{}`",
            path.display(),
            artifact_path
        ));
    }
    if parsed_artifact.binary_name != artifact_binary_name {
        return Err(format!(
            "`{}` nuis artifact binary_name mismatch between manifest and `{}`",
            path.display(),
            artifact_path
        ));
    }
    if parsed_artifact.binary_bytes != artifact_binary_bytes {
        return Err(format!(
            "`{}` nuis artifact binary_bytes mismatch between manifest and `{}`",
            path.display(),
            artifact_path
        ));
    }
    if parsed_artifact.build_manifest_source != source {
        return Err(format!(
            "`{}` nuis artifact embedded build manifest does not match manifest source",
            path.display()
        ));
    }
    if parsed_artifact.envelope != parsed_envelope {
        return Err(format!(
            "`{}` nuis artifact envelope mismatch between manifest and `{}`",
            path.display(),
            artifact_path
        ));
    }
    if parsed_artifact.lifecycle.schema != "nuis-lifecycle-contract-v1" {
        return Err(format!(
            "`{}` nuis artifact lifecycle schema mismatch: expected `nuis-lifecycle-contract-v1`, found `{}`",
            path.display(),
            parsed_artifact.lifecycle.schema
        ));
    }
    if parsed_artifact.lifecycle.bootstrap_entry != lifecycle_bootstrap_entry
        || parsed_artifact.lifecycle.tick_policy != lifecycle_tick_policy
        || parsed_artifact.lifecycle.shutdown_policy != lifecycle_shutdown_policy
        || parsed_artifact.lifecycle.yalivia_rpc != lifecycle_yalivia_rpc
        || parsed_artifact.lifecycle.hook_surface != lifecycle_hook_surface
        || parsed_artifact.lifecycle.export_surface != lifecycle_export_surface
        || parsed_artifact.lifecycle.runtime_capability_flags != lifecycle_runtime_capability_flags
    {
        return Err(format!(
            "`{}` nuis artifact lifecycle contract mismatch between manifest and `{}`",
            path.display(),
            artifact_path
        ));
    }

    for item in &artifacts {
        let bytes = match fs::read(&item.path) {
            Ok(bytes) => bytes,
            Err(_) => artifact_hash_fallback_bytes(
                &item.kind,
                &domain_build_units,
                bridge_registry_inline.as_deref(),
                host_bridge_plan_index_inline.as_deref(),
            )
            .ok_or_else(|| format!("failed to read artifact `{}`", item.path))?,
        };
        if bytes.len() != item.bytes {
            return Err(format!(
                "artifact `{}` bytes mismatch for kind `{}`: manifest={}, actual={}",
                item.path,
                item.kind,
                item.bytes,
                bytes.len()
            ));
        }
        let actual_hash = fnv1a64_hex(&bytes);
        if actual_hash != item.fnv1a64 {
            return Err(format!(
                "artifact `{}` hash mismatch for kind `{}`: manifest={}, actual={}",
                item.path, item.kind, item.fnv1a64, actual_hash
            ));
        }
    }

    for unit in &domain_build_units {
        if unit.domain_family == "cpu" {
            if unit.artifact_payload_blob_path.is_some()
                || unit.artifact_payload_blob_bytes.is_some()
                || unit.artifact_payload_format.is_some()
                || unit.artifact_payload_blob_inline.is_some()
            {
                return Err(format!(
                    "`{}` cpu domain_build_unit must not declare hetero payload blob fields",
                    path.display()
                ));
            }
            continue;
        }
        let blob_path = unit.artifact_payload_blob_path.as_ref().ok_or_else(|| {
            format!(
                "`{}` domain_build_unit `{}` is missing `artifact_payload_blob_path`",
                path.display(),
                unit.domain_family
            )
        })?;
        let blob_bytes_declared = unit.artifact_payload_blob_bytes.ok_or_else(|| {
            format!(
                "`{}` domain_build_unit `{}` is missing `artifact_payload_blob_bytes`",
                path.display(),
                unit.domain_family
            )
        })?;
        let blob_format = unit.artifact_payload_format.as_deref().ok_or_else(|| {
            format!(
                "`{}` domain_build_unit `{}` is missing `artifact_payload_format`",
                path.display(),
                unit.domain_family
            )
        })?;
        if blob_format != "ndpb-v2" {
            return Err(format!(
                "`{}` domain_build_unit `{}` has unsupported artifact_payload_format `{}`; expected `ndpb-v2`",
                path.display(),
                unit.domain_family,
                blob_format
            ));
        }
        let (blob, blob_label) = match fs::read(blob_path) {
            Ok(blob) => (blob, blob_path.clone()),
            Err(_) => {
                let inline = unit.artifact_payload_blob_inline.as_ref().ok_or_else(|| {
                    format!(
                        "failed to read domain payload blob `{}` referenced by `{}` and no `artifact_payload_blob_inline` fallback is available",
                        blob_path,
                        path.display()
                    )
                })?;
                (
                    hex_decode_bytes(inline).map_err(|error| {
                        format!(
                            "invalid `artifact_payload_blob_inline` for domain `{}` in `{}`: {error}",
                            unit.domain_family,
                            path.display()
                        )
                    })?,
                    format!("<embedded-domain-payload-blob:{}>", unit.domain_family),
                )
            }
        };
        if blob.len() != blob_bytes_declared {
            return Err(format!(
                "domain payload blob `{}` byte length mismatch for `{}`: manifest={}, actual={}",
                blob_label,
                unit.domain_family,
                blob_bytes_declared,
                blob.len()
            ));
        }
        let decoded_blob = decode_domain_build_unit_payload_blob(&blob)
            .map_err(|error| format!("invalid domain payload blob `{}`: {error}", blob_label))?;
        if decoded_blob.domain_family != unit.domain_family {
            return Err(format!(
                "domain payload blob `{}` domain mismatch: manifest={}, blob={}",
                blob_label, unit.domain_family, decoded_blob.domain_family
            ));
        }
        if decoded_blob.package_id != unit.package_id {
            return Err(format!(
                "domain payload blob `{}` package mismatch: manifest={}, blob={}",
                blob_label, unit.package_id, decoded_blob.package_id
            ));
        }
        if decoded_blob.backend_family != unit.backend_family {
            return Err(format!(
                "domain payload blob `{}` backend_family mismatch for `{}`",
                blob_label, unit.domain_family
            ));
        }
        if decoded_blob.selected_lowering_target != unit.selected_lowering_target {
            return Err(format!(
                "domain payload blob `{}` selected_lowering_target mismatch for `{}`",
                blob_label, unit.domain_family
            ));
        }
        if decoded_blob.contract_family != unit.contract_family {
            return Err(format!(
                "domain payload blob `{}` contract_family mismatch: manifest={}, blob={}",
                blob_label, unit.contract_family, decoded_blob.contract_family
            ));
        }
        if decoded_blob.packaging_role != unit.packaging_role {
            return Err(format!(
                "domain payload blob `{}` packaging_role mismatch: manifest={}, blob={}",
                blob_label, unit.packaging_role, decoded_blob.packaging_role
            ));
        }
        if decoded_blob.payload_kind != "contract-sidecar" {
            return Err(format!(
                "domain payload blob `{}` payload_kind mismatch: expected `contract-sidecar`, found `{}`",
                blob_label,
                decoded_blob.payload_kind
            ));
        }
        if decoded_blob.payload_format != "toml" {
            return Err(format!(
                "domain payload blob `{}` payload_format mismatch: expected `toml`, found `{}`",
                blob_label, decoded_blob.payload_format
            ));
        }
        let payload_path = unit.artifact_payload_path.as_ref().ok_or_else(|| {
            format!(
                "`{}` domain_build_unit `{}` is missing `artifact_payload_path`",
                path.display(),
                unit.domain_family
            )
        })?;
        let bridge_stub_path = unit.artifact_bridge_stub_path.as_ref().ok_or_else(|| {
            format!(
                "`{}` domain_build_unit `{}` is missing `artifact_bridge_stub_path`",
                path.display(),
                unit.domain_family
            )
        })?;
        let expected_section_count = if unit.domain_family == "shader"
            || unit.domain_family == "kernel"
            || unit.domain_family == "network"
        {
            5
        } else {
            4
        };
        if decoded_blob.sections.len() != expected_section_count {
            return Err(format!(
                "domain payload blob `{}` section count mismatch: expected {}, found {}",
                blob_label,
                expected_section_count,
                decoded_blob.sections.len()
            ));
        }
        let contract_section = &decoded_blob.sections[0];
        if contract_section.name != "contract_toml" {
            return Err(format!(
                "domain payload blob `{}` section name mismatch: expected `contract_toml`, found `{}`",
                blob_label,
                contract_section.name
            ));
        }
        let payload = fs::read(payload_path).unwrap_or_else(|_| contract_section.bytes.clone());
        if contract_section.bytes != payload {
            return Err(format!(
                "domain payload blob `{}` payload content mismatch against `{}`",
                blob_label, payload_path
            ));
        }
        domain_payload_contract_sections_checked += 1;
        let lowering_section = &decoded_blob.sections[1];
        if lowering_section.name != "lowering_plan" {
            return Err(format!(
                "domain payload blob `{}` lowering section name mismatch: expected `lowering_plan`, found `{}`",
                blob_label,
                lowering_section.name
            ));
        }
        let expected_lowering_plan = render_domain_build_unit_lowering_plan(unit);
        if lowering_section.bytes != expected_lowering_plan.as_bytes() {
            return Err(format!(
                "domain payload blob `{}` lowering plan content mismatch for `{}`",
                blob_label, unit.domain_family
            ));
        }
        domain_payload_lowering_plans_checked += 1;
        let backend_section = &decoded_blob.sections[2];
        if backend_section.name != "backend_stub" {
            return Err(format!(
                "domain payload blob `{}` backend section name mismatch: expected `backend_stub`, found `{}`",
                blob_label,
                backend_section.name
            ));
        }
        let expected_backend_stub = render_domain_build_unit_backend_stub(unit);
        if backend_section.bytes != expected_backend_stub.as_bytes() {
            return Err(format!(
                "domain payload blob `{}` backend stub content mismatch for `{}`",
                blob_label, unit.domain_family
            ));
        }
        domain_payload_backend_stubs_checked += 1;
        let bridge_section = &decoded_blob.sections[3];
        if bridge_section.name != "bridge_plan" {
            return Err(format!(
                "domain payload blob `{}` bridge section name mismatch: expected `bridge_plan`, found `{}`",
                blob_label,
                bridge_section.name
            ));
        }
        let expected_bridge_plan = render_domain_build_unit_bridge_plan(unit);
        if bridge_section.bytes != expected_bridge_plan.as_bytes() {
            return Err(format!(
                "domain payload blob `{}` bridge plan content mismatch for `{}`",
                blob_label, unit.domain_family
            ));
        }
        domain_payload_bridge_plans_checked += 1;
        let expected_bridge_stub = render_domain_build_unit_host_bridge_stub(unit);
        let bridge_stub =
            fs::read_to_string(bridge_stub_path).unwrap_or_else(|_| expected_bridge_stub.clone());
        if bridge_stub != expected_bridge_stub {
            return Err(format!(
                "domain bridge stub `{}` content mismatch for `{}`",
                bridge_stub_path, unit.domain_family
            ));
        }
        domain_bridge_stubs_checked += 1;
        if unit.domain_family == "shader" {
            let ir_sidecar_path = unit.artifact_ir_sidecar_path.as_ref().ok_or_else(|| {
                format!(
                    "`{}` domain_build_unit `{}` is missing `artifact_ir_sidecar_path`",
                    path.display(),
                    unit.domain_family
                )
            })?;
            let shader_ir_section = &decoded_blob.sections[4];
            if shader_ir_section.name != "shader_ir_sidecar" {
                return Err(format!(
                    "domain payload blob `{}` shader section name mismatch: expected `shader_ir_sidecar`, found `{}`",
                    blob_label,
                    shader_ir_section.name
                ));
            }
            let expected_shader_ir_sidecar = render_domain_build_unit_shader_ir_sidecar(unit);
            if shader_ir_section.bytes != expected_shader_ir_sidecar.as_bytes() {
                return Err(format!(
                    "domain payload blob `{}` shader ir sidecar content mismatch for `{}`",
                    blob_label, unit.domain_family
                ));
            }
            let shader_ir_sidecar = fs::read_to_string(ir_sidecar_path)
                .unwrap_or_else(|_| expected_shader_ir_sidecar.clone());
            if shader_ir_sidecar != expected_shader_ir_sidecar {
                return Err(format!(
                    "domain shader ir sidecar `{}` content mismatch for `{}`",
                    ir_sidecar_path, unit.domain_family
                ));
            }
        } else if unit.domain_family == "kernel" {
            let ir_sidecar_path = unit.artifact_ir_sidecar_path.as_ref().ok_or_else(|| {
                format!(
                    "`{}` domain_build_unit `{}` is missing `artifact_ir_sidecar_path`",
                    path.display(),
                    unit.domain_family
                )
            })?;
            let kernel_ir_section = &decoded_blob.sections[4];
            if kernel_ir_section.name != "kernel_ir_sidecar" {
                return Err(format!(
                    "domain payload blob `{}` kernel section name mismatch: expected `kernel_ir_sidecar`, found `{}`",
                    blob_label,
                    kernel_ir_section.name
                ));
            }
            let expected_kernel_ir_sidecar = render_domain_build_unit_kernel_ir_sidecar(unit);
            if kernel_ir_section.bytes != expected_kernel_ir_sidecar.as_bytes() {
                return Err(format!(
                    "domain payload blob `{}` kernel ir sidecar content mismatch for `{}`",
                    blob_label, unit.domain_family
                ));
            }
            let kernel_ir_sidecar = fs::read_to_string(ir_sidecar_path)
                .unwrap_or_else(|_| expected_kernel_ir_sidecar.clone());
            if kernel_ir_sidecar != expected_kernel_ir_sidecar {
                return Err(format!(
                    "domain kernel ir sidecar `{}` content mismatch for `{}`",
                    ir_sidecar_path, unit.domain_family
                ));
            }
        } else if unit.domain_family == "network" {
            let ir_sidecar_path = unit.artifact_ir_sidecar_path.as_ref().ok_or_else(|| {
                format!(
                    "`{}` domain_build_unit `{}` is missing `artifact_ir_sidecar_path`",
                    path.display(),
                    unit.domain_family
                )
            })?;
            let network_ir_section = &decoded_blob.sections[4];
            if network_ir_section.name != "network_ir_sidecar" {
                return Err(format!(
                    "domain payload blob `{}` network section name mismatch: expected `network_ir_sidecar`, found `{}`",
                    blob_label,
                    network_ir_section.name
                ));
            }
            let expected_network_ir_sidecar = render_domain_build_unit_network_ir_sidecar(unit);
            if network_ir_section.bytes != expected_network_ir_sidecar.as_bytes() {
                return Err(format!(
                    "domain payload blob `{}` network ir sidecar content mismatch for `{}`",
                    blob_label, unit.domain_family
                ));
            }
            let network_ir_sidecar = fs::read_to_string(ir_sidecar_path)
                .unwrap_or_else(|_| expected_network_ir_sidecar.clone());
            if network_ir_sidecar != expected_network_ir_sidecar {
                return Err(format!(
                    "domain network ir sidecar `{}` content mismatch for `{}`",
                    ir_sidecar_path, unit.domain_family
                ));
            }
        }
        domain_payload_blob_sections_checked += decoded_blob.sections.len();
        domain_payload_blobs_checked += 1;
    }

    let mut bridge_registry_checked = 0usize;
    let mut bridge_registry_entries_checked = 0usize;
    if bridge_registry_path.is_some() || bridge_registry_inline.is_some() {
        if bridge_registry_schema.as_deref() != Some("nuis-bridge-registry-v1") {
            return Err(format!(
                "`{}` has unsupported bridge registry schema `{:?}`; expected `nuis-bridge-registry-v1`",
                path.display(),
                bridge_registry_schema
            ));
        }
        let (registry_source, registry_label) = if let Some(source) = &bridge_registry_inline {
            (source.clone(), "<embedded-bridge-registry>".to_owned())
        } else {
            let bridge_registry_path = bridge_registry_path.as_ref().unwrap();
            (
                fs::read_to_string(bridge_registry_path).map_err(|error| {
                    format!(
                        "failed to read bridge registry `{}` referenced by `{}`: {error}",
                        bridge_registry_path,
                        path.display()
                    )
                })?,
                bridge_registry_path.clone(),
            )
        };
        let registry_schema =
            parse_required_toml_string(&registry_source, "schema", Path::new(&registry_label))?;
        if registry_schema != "nuis-bridge-registry-v1" {
            return Err(format!(
                "bridge registry `{}` has unsupported schema `{}`",
                registry_label, registry_schema
            ));
        }
        let registry_count = parse_required_toml_usize(
            &registry_source,
            "bridge_count",
            Path::new(&registry_label),
        )?;
        if registry_count != bridge_registry_units {
            return Err(format!(
                "bridge registry `{}` count mismatch: manifest={}, registry={}",
                registry_label, bridge_registry_units, registry_count
            ));
        }
        let bridge_block_count = registry_source
            .lines()
            .filter(|line| line.trim() == "[[bridge]]")
            .count();
        if bridge_block_count != bridge_registry_units {
            return Err(format!(
                "bridge registry `{}` block count mismatch: manifest={}, blocks={}",
                registry_label, bridge_registry_units, bridge_block_count
            ));
        }
        if bridge_registry_units != heterogeneous_domain_count {
            return Err(format!(
                "`{}` bridge_registry_units mismatch: expected {}, found {}",
                path.display(),
                heterogeneous_domain_count,
                bridge_registry_units
            ));
        }
        for unit in domain_build_units
            .iter()
            .filter(|unit| unit.domain_family != "cpu")
        {
            let expected_bridge_stub = unit
                .artifact_bridge_stub_path
                .as_deref()
                .unwrap_or("<none>");
            if !registry_source.contains(&format!(
                "bridge_stub_path = \"{}\"",
                escape_toml_string(expected_bridge_stub)
            )) {
                return Err(format!(
                    "bridge registry `{}` is missing bridge stub path for `{}`",
                    registry_label, unit.domain_family
                ));
            }
            bridge_registry_entries_checked += 1;
        }
        bridge_registry_checked = 1;
    } else if heterogeneous_domain_count > 0 {
        return Err(format!(
            "`{}` is missing bridge registry for heterogeneous domains",
            path.display()
        ));
    }

    let mut host_bridge_plan_checked = 0usize;
    let mut host_bridge_plan_entries_checked = 0usize;
    if host_bridge_plan_index_path.is_some() || host_bridge_plan_index_inline.is_some() {
        if host_bridge_plan_index_schema.as_deref() != Some("nuis-host-bridge-plan-index-v1") {
            return Err(format!(
                "`{}` has unsupported host bridge plan index schema `{:?}`; expected `nuis-host-bridge-plan-index-v1`",
                path.display(),
                host_bridge_plan_index_schema
            ));
        }
        let (plan_index_source, plan_index_label) =
            if let Some(source) = &host_bridge_plan_index_inline {
                (
                    source.clone(),
                    "<embedded-host-bridge-plan-index>".to_owned(),
                )
            } else {
                let host_bridge_plan_index_path = host_bridge_plan_index_path.as_ref().unwrap();
                (
                    fs::read_to_string(host_bridge_plan_index_path).map_err(|error| {
                        format!(
                        "failed to read host bridge plan index `{}` referenced by `{}`: {error}",
                        host_bridge_plan_index_path,
                        path.display()
                    )
                    })?,
                    host_bridge_plan_index_path.clone(),
                )
            };
        let index_schema =
            parse_required_toml_string(&plan_index_source, "schema", Path::new(&plan_index_label))?;
        if index_schema != "nuis-host-bridge-plan-index-v1" {
            return Err(format!(
                "host bridge plan index `{}` has unsupported schema `{}`",
                plan_index_label, index_schema
            ));
        }
        let plan_count = parse_required_toml_usize(
            &plan_index_source,
            "plan_count",
            Path::new(&plan_index_label),
        )?;
        if plan_count != host_bridge_plan_units {
            return Err(format!(
                "host bridge plan index `{}` count mismatch: manifest={}, index={}",
                plan_index_label, host_bridge_plan_units, plan_count
            ));
        }
        let plan_block_count = plan_index_source
            .lines()
            .filter(|line| line.trim() == "[[plan]]")
            .count();
        if plan_block_count != host_bridge_plan_units {
            return Err(format!(
                "host bridge plan index `{}` block count mismatch: manifest={}, blocks={}",
                plan_index_label, host_bridge_plan_units, plan_block_count
            ));
        }
        if host_bridge_plan_units != heterogeneous_domain_count {
            return Err(format!(
                "`{}` host_bridge_plan_units mismatch: expected {}, found {}",
                path.display(),
                heterogeneous_domain_count,
                host_bridge_plan_units
            ));
        }
        for unit in domain_build_units
            .iter()
            .filter(|unit| unit.domain_family != "cpu")
        {
            let expected_bridge_stub = unit
                .artifact_bridge_stub_path
                .as_deref()
                .unwrap_or("<none>");
            if !plan_index_source.contains(&format!(
                "bridge_stub_path = \"{}\"",
                escape_toml_string(expected_bridge_stub)
            )) {
                return Err(format!(
                    "host bridge plan index `{}` is missing bridge stub path for `{}`",
                    plan_index_label, unit.domain_family
                ));
            }
            host_bridge_plan_entries_checked += 1;
        }
        host_bridge_plan_checked = 1;
    } else if heterogeneous_domain_count > 0 {
        return Err(format!(
            "`{}` is missing host bridge plan index for heterogeneous domains",
            path.display()
        ));
    }

    let mut doc_index_checked = 0usize;
    if let Some(doc_index_path) = &doc_index_path {
        let doc_index_source = fs::read_to_string(doc_index_path).map_err(|error| {
            format!(
                "failed to read doc index `{}` referenced by `{}`: {error}",
                doc_index_path,
                path.display()
            )
        })?;
        if !doc_index_source.contains("\"kind\":\"nuis_doc_index\"") {
            return Err(format!(
                "doc index `{}` has unexpected kind; expected `nuis_doc_index`",
                doc_index_path
            ));
        }
        if !doc_index_source.contains(&format!("\"module_count\":{}", doc_index_module_count)) {
            return Err(format!(
                "doc index `{}` module_count mismatch: manifest={}, index payload differs",
                doc_index_path, doc_index_module_count
            ));
        }
        if !doc_index_source.contains(&format!(
            "\"documented_item_count\":{}",
            doc_index_documented_item_count
        )) {
            return Err(format!(
                "doc index `{}` documented_item_count mismatch: manifest={}, index payload differs",
                doc_index_path, doc_index_documented_item_count
            ));
        }
        doc_index_checked = 1;
    }

    let mut project_metadata_checked = 0usize;
    if let Some(plan_index) = &project_plan_index {
        let plan_source = fs::read_to_string(plan_index).map_err(|error| {
            format!(
                "failed to read project plan index `{}` referenced by `{}`: {error}",
                plan_index,
                path.display()
            )
        })?;
        if let Some(summary) = &project_plan_summary {
            let expected = format!("summary {summary}");
            if !plan_source.lines().any(|line| line.trim() == expected) {
                return Err(project_metadata_summary_mismatch_error(
                    "plan",
                    plan_index,
                    &expected,
                    &plan_source,
                    &input,
                    &output_dir,
                ));
            }
        }
        project_metadata_checked += 1;
    }
    if let Some(docs_index) = &project_docs_index {
        let docs_source = fs::read_to_string(docs_index).map_err(|error| {
            format!(
                "failed to read project docs index `{}` referenced by `{}`: {error}",
                docs_index,
                path.display()
            )
        })?;
        let expected = format!(
            "summary\tmodules={}\tdocumented_modules={}\tdocumented_items={}",
            project_docs_module_count,
            project_docs_documented_module_count,
            project_docs_documented_item_count
        );
        if !docs_source.lines().any(|line| line.trim() == expected) {
            return Err(project_metadata_summary_mismatch_error(
                "docs",
                docs_index,
                &expected,
                &docs_source,
                &input,
                &output_dir,
            ));
        }
        project_metadata_checked += 1;
    }
    if let Some(imports_index) = &project_imports_index {
        let imports_source = fs::read_to_string(imports_index).map_err(|error| {
            format!(
                "failed to read project imports index `{}` referenced by `{}`: {error}",
                imports_index,
                path.display()
            )
        })?;
        let expected = format!(
            "summary\tlibraries={}\tvisible_libraries={}\tvisible_modules={}\tdocumented_visible_modules={}\tdocumented_visible_items={}",
            project_imports_library_count,
            project_imports_visible_library_count,
            project_imports_visible_module_count,
            project_imports_documented_visible_module_count,
            project_imports_documented_visible_item_count
        );
        if !imports_source.lines().any(|line| line.trim() == expected) {
            return Err(project_metadata_summary_mismatch_error(
                "imports",
                imports_index,
                &expected,
                &imports_source,
                &input,
                &output_dir,
            ));
        }
        project_metadata_checked += 1;
    }
    if let Some(galaxy_index) = &project_galaxy_index {
        let galaxy_source = fs::read_to_string(galaxy_index).map_err(|error| {
            format!(
                "failed to read project galaxy index `{}` referenced by `{}`: {error}",
                galaxy_index,
                path.display()
            )
        })?;
        let expected = format!(
            "summary\tgalaxies={}\tdocumented_galaxies={}\tdocumented_library_modules={}\tdocumented_items={}",
            project_galaxy_count,
            project_documented_galaxy_count,
            project_documented_galaxy_library_module_count,
            project_documented_galaxy_item_count
        );
        if !galaxy_source.lines().any(|line| line.trim() == expected) {
            return Err(project_metadata_summary_mismatch_error(
                "galaxy",
                galaxy_index,
                &expected,
                &galaxy_source,
                &input,
                &output_dir,
            ));
        }
        project_metadata_checked += 1;
    }
    if let Some(packet_index) = &project_packet_index {
        fs::read_to_string(packet_index).map_err(|error| {
            format!(
                "failed to read project packet index `{}` referenced by `{}`: {error}",
                packet_index,
                path.display()
            )
        })?;
        project_metadata_checked += 1;
    }

    Ok(BuildManifestVerifyReport {
        schema,
        input,
        output_dir,
        packaging_mode,
        envelope_path,
        envelope_schema,
        envelope_package_count,
        artifact_path,
        artifact_schema,
        artifact_binary_name,
        artifact_binary_bytes,
        lifecycle_schema: parsed_artifact.lifecycle.schema.clone(),
        lifecycle_bootstrap_entry,
        lifecycle_tick_policy,
        lifecycle_shutdown_policy,
        lifecycle_yalivia_rpc,
        lifecycle_hook_count: lifecycle_hook_surface.len(),
        lifecycle_hook_surface: lifecycle_hook_surface.clone(),
        lifecycle_export_count: lifecycle_export_surface.len(),
        lifecycle_export_surface: lifecycle_export_surface.clone(),
        lifecycle_runtime_capability_flags: lifecycle_runtime_capability_flags.clone(),
        execution_contracts_checked,
        domain_build_unit_count: domain_build_units.len(),
        heterogeneous_domain_count,
        domain_payload_blobs_checked,
        domain_payload_blob_sections_checked,
        domain_payload_contract_sections_checked,
        domain_payload_lowering_plans_checked,
        domain_payload_backend_stubs_checked,
        domain_payload_bridge_plans_checked,
        domain_bridge_stubs_checked,
        domain_build_units,
        cpu_target_abi,
        cpu_target_machine_arch,
        cpu_target_machine_os,
        cpu_target_object_format,
        cpu_target_calling_abi,
        cpu_target_clang,
        cpu_target_cross,
        loaded_nustar,
        compile_cache_status,
        compile_cache_key,
        compile_cache_root,
        doc_index_path,
        doc_index_module_count,
        doc_index_documented_item_count,
        doc_index_checked,
        project_text_handle_rewrite_helper_hits,
        project_text_handle_rewrite_local_hits,
        project_plan_index,
        project_docs_index,
        project_docs_module_count,
        project_docs_documented_module_count,
        project_docs_documented_item_count,
        project_imports_index,
        project_imports_library_count,
        project_imports_visible_library_count,
        project_imports_visible_module_count,
        project_imports_documented_visible_module_count,
        project_imports_documented_visible_item_count,
        project_galaxy_index,
        project_galaxy_count,
        project_documented_galaxy_count,
        project_documented_galaxy_library_module_count,
        project_documented_galaxy_item_count,
        project_packet_index,
        bridge_registry_path,
        bridge_registry_units,
        bridge_registry_checked,
        bridge_registry_entries_checked,
        host_bridge_plan_index_path,
        host_bridge_plan_units,
        host_bridge_plan_checked,
        host_bridge_plan_entries_checked,
        artifacts_checked: artifacts.len(),
        project_metadata_checked,
    })
}

pub fn verify_nuis_compiled_artifact(
    path: &Path,
) -> Result<NuisCompiledArtifactVerifyReport, String> {
    let artifact = parse_nuis_compiled_artifact(path)?;
    validate_nuis_compiled_artifact_layout(path, &artifact)?;
    if artifact.schema != "nuis-compiled-artifact-v1" {
        return Err(format!(
            "`{}` has unsupported nuis artifact schema `{}`; expected `nuis-compiled-artifact-v1`",
            path.display(),
            artifact.schema
        ));
    }
    if artifact.binary_blob.len() != artifact.binary_bytes {
        return Err(format!(
            "`{}` binary byte length mismatch: declared={}, actual={}",
            path.display(),
            artifact.binary_bytes,
            artifact.binary_blob.len()
        ));
    }
    if artifact.build_manifest_source.len() != artifact.build_manifest_bytes {
        return Err(format!(
            "`{}` build manifest byte length mismatch: declared={}, actual={}",
            path.display(),
            artifact.build_manifest_bytes,
            artifact.build_manifest_source.len()
        ));
    }
    let expected_lifecycle =
        build_nuis_lifecycle_contract(&artifact.envelope, &artifact.packaging_mode);
    if artifact.lifecycle != expected_lifecycle {
        return Err(format!(
            "`{}` lifecycle contract mismatch: artifact lifecycle does not match the expected contract derived from envelope/package mode",
            path.display()
        ));
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("nuis_artifact_verify_{nonce}"));
    fs::create_dir_all(&temp_root)
        .map_err(|error| format!("failed to create `{}`: {error}", temp_root.display()))?;

    let manifest_path = temp_root.join("nuis.build.manifest.toml");
    let envelope_path = temp_root.join("nuis.executable.envelope.toml");
    let artifact_path = temp_root.join("nuis.compiled.artifact");
    let binary_path = temp_root.join(&artifact.binary_name);

    fs::write(&binary_path, &artifact.binary_blob)
        .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
    write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;

    let relocated_manifest = render_relocated_unpacked_build_manifest(
        &artifact,
        &temp_root,
        &envelope_path,
        &artifact_path,
        &binary_path,
    )?;
    let mut relocated_artifact = artifact.clone();
    relocated_artifact.build_manifest_source = relocated_manifest.clone();
    relocated_artifact.build_manifest_bytes = relocated_manifest.len();
    write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
    fs::write(&manifest_path, &relocated_manifest)
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;

    let manifest_report = verify_build_manifest(&manifest_path)?;
    let _ = fs::remove_dir_all(&temp_root);

    Ok(NuisCompiledArtifactVerifyReport {
        schema: artifact.schema,
        packaging_mode: artifact.packaging_mode,
        binary_name: artifact.binary_name,
        binary_bytes: artifact.binary_bytes,
        build_manifest_bytes: artifact.build_manifest_bytes,
        envelope_schema: artifact.envelope.schema,
        envelope_package_count: artifact.envelope.package_count,
        lifecycle_schema: artifact.lifecycle.schema,
        lifecycle_bootstrap_entry: artifact.lifecycle.bootstrap_entry,
        lifecycle_tick_policy: artifact.lifecycle.tick_policy,
        lifecycle_shutdown_policy: artifact.lifecycle.shutdown_policy,
        lifecycle_yalivia_rpc: artifact.lifecycle.yalivia_rpc,
        lifecycle_hook_count: artifact.lifecycle.hook_surface.len(),
        lifecycle_hook_surface: artifact.lifecycle.hook_surface.clone(),
        lifecycle_export_count: artifact.lifecycle.export_surface.len(),
        lifecycle_export_surface: artifact.lifecycle.export_surface.clone(),
        lifecycle_runtime_capability_flags: artifact.lifecycle.runtime_capability_flags.clone(),
        lifecycle_contract_consistent: true,
        lifecycle_runtime_capability_flags_consistent: true,
        execution_contracts_checked: manifest_report.execution_contracts_checked,
        cpu_target_abi: artifact.cpu_target_abi,
        cpu_target_machine_arch: artifact.cpu_target_machine_arch,
        cpu_target_machine_os: artifact.cpu_target_machine_os,
        cpu_target_object_format: artifact.cpu_target_object_format,
        cpu_target_calling_abi: artifact.cpu_target_calling_abi,
        artifact_roundtrip_verified: true,
    })
}

pub fn parse_nuis_executable_envelope(path: &Path) -> Result<NuisExecutableEnvelope, String> {
    shared_parse_nuis_executable_envelope(path).map_err(|error| error.to_string())
}

pub fn parse_nuis_executable_envelope_from_source(
    source: &str,
    path: &Path,
) -> Result<NuisExecutableEnvelope, String> {
    shared_parse_nuis_executable_envelope_from_source(source, path)
        .map_err(|error| error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtifactHashRow {
    kind: String,
    path: String,
    bytes: usize,
    fnv1a64: String,
}

fn parse_domain_build_unit_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<BuildManifestDomainBuildUnit>, String> {
    shared_parse_domain_build_unit_blocks(source, path).map_err(|error| error.to_string())
}

fn parse_artifact_hash_blocks(source: &str, path: &Path) -> Result<Vec<ArtifactHashRow>, String> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[artifact_hash]]" {
            if in_block {
                rows.push(parse_artifact_hash_row(&current, path)?);
                current.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(parse_artifact_hash_row(&current, path)?);
                current.clear();
                in_block = false;
            }
            continue;
        }
        if in_block {
            if let Some((key, value)) = line.split_once('=') {
                current.insert(key.trim().to_owned(), value.trim().to_owned());
            }
        }
    }
    if in_block {
        rows.push(parse_artifact_hash_row(&current, path)?);
    }
    Ok(rows)
}

fn parse_artifact_hash_row(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<ArtifactHashRow, String> {
    let kind = parse_required_map_string(values, "kind", path)?;
    let artifact_path = parse_required_map_string(values, "path", path)?;
    let bytes = parse_required_map_usize(values, "bytes", path)?;
    let fnv1a64 = parse_required_map_string(values, "fnv1a64", path)?;
    Ok(ArtifactHashRow {
        kind,
        path: artifact_path,
        bytes,
        fnv1a64,
    })
}

fn parse_required_toml_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    parse_optional_toml_string(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

fn parse_required_toml_bool(source: &str, key: &str, path: &Path) -> Result<bool, String> {
    parse_optional_toml_bool(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

fn parse_required_toml_usize(source: &str, key: &str, path: &Path) -> Result<usize, String> {
    parse_optional_toml_usize(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

fn parse_required_toml_string_array(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<Vec<String>, String> {
    parse_optional_toml_string_array(source, key)
        .ok_or_else(|| format!("`{}` is missing required key `{key}`", path.display()))
}

fn parse_optional_toml_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                return unescape_toml_basic_string(&value[1..value.len() - 1]);
            }
            return None;
        }
    }
    None
}

fn parse_optional_toml_bool(source: &str, key: &str) -> Option<bool> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return match rest.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
    }
    None
}

fn parse_optional_toml_usize(source: &str, key: &str) -> Option<usize> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest.trim().parse::<usize>().ok();
        }
    }
    None
}

fn parse_optional_toml_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if !value.starts_with('[') || !value.ends_with(']') {
                return None;
            }
            let inner = value[1..value.len() - 1].trim();
            if inner.is_empty() {
                return Some(Vec::new());
            }
            let mut items = Vec::new();
            for part in inner.split(',') {
                let item = part.trim();
                if !item.starts_with('"') || !item.ends_with('"') || item.len() < 2 {
                    return None;
                }
                items.push(unescape_toml_basic_string(&item[1..item.len() - 1])?);
            }
            return Some(items);
        }
    }
    None
}

fn parse_required_map_string(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
) -> Result<String, String> {
    parse_required_map_string_in_block(values, key, manifest_path, "artifact_hash")
}

fn parse_required_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
) -> Result<usize, String> {
    let value = values.get(key).ok_or_else(|| {
        format!(
            "`{}` artifact_hash block is missing required key `{key}`",
            manifest_path.display()
        )
    })?;
    value.parse::<usize>().map_err(|_| {
        format!(
            "`{}` artifact_hash key `{key}` must be an unsigned integer",
            manifest_path.display()
        )
    })
}

fn parse_required_map_string_in_block(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
    block_name: &str,
) -> Result<String, String> {
    let value = values.get(key).ok_or_else(|| {
        format!(
            "`{}` {block_name} block is missing required key `{key}`",
            manifest_path.display()
        )
    })?;
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return unescape_toml_basic_string(&value[1..value.len() - 1]).ok_or_else(|| {
            format!(
                "`{}` {block_name} key `{key}` contains an unsupported escape sequence",
                manifest_path.display()
            )
        });
    }
    Err(format!(
        "`{}` {block_name} key `{key}` must be a quoted string",
        manifest_path.display()
    ))
}

fn unescape_toml_basic_string(value: &str) -> Option<String> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let escaped = chars.next()?;
        match escaped {
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            _ => return None,
        }
    }
    Some(out)
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
        encode_nuis_executable_envelope_binary, parse_nuis_compiled_artifact,
        parse_nuis_executable_envelope, render_nuis_executable_envelope,
        resolve_cpu_build_target_from_abi, verify_build_manifest, verify_nuis_compiled_artifact,
        BuildManifestCacheInfo, BuildManifestContext, BuildManifestDomainBuildUnit,
        BuildManifestProjectInfo, CompileArtifacts, CpuBuildTarget, NuisExecutableEnvelope,
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
        let kernel_payload = dir.join("nuis.domain.kernel.payload.toml");
        let kernel_bridge_stub = dir.join("nuis.domain.kernel.bridge.stub.txt");
        let kernel_payload_blob = dir.join("nuis.domain.kernel.payload.bin");
        let network_payload = dir.join("nuis.domain.network.payload.toml");
        let network_bridge_stub = dir.join("nuis.domain.network.bridge.stub.txt");
        let network_payload_blob = dir.join("nuis.domain.network.payload.bin");
        let bridge_registry = dir.join("nuis.bridge.registry.toml");
        let host_bridge_plan_index = dir.join("nuis.host-bridge.plan-index.toml");
        assert!(kernel_payload.exists());
        assert!(kernel_bridge_stub.exists());
        assert!(kernel_payload_blob.exists());
        assert!(network_payload.exists());
        assert!(network_bridge_stub.exists());
        assert!(network_payload_blob.exists());
        assert!(bridge_registry.exists());
        assert!(host_bridge_plan_index.exists());
        let kernel_payload_text = fs::read_to_string(&kernel_payload).unwrap();
        let kernel_bridge_stub_text = fs::read_to_string(&kernel_bridge_stub).unwrap();
        let network_payload_text = fs::read_to_string(&network_payload).unwrap();
        let network_bridge_stub_text = fs::read_to_string(&network_bridge_stub).unwrap();
        let bridge_registry_text = fs::read_to_string(&bridge_registry).unwrap();
        let host_bridge_plan_index_text = fs::read_to_string(&host_bridge_plan_index).unwrap();
        let bridge_registry_path_text = bridge_registry.display().to_string();
        let host_bridge_plan_index_path_text = host_bridge_plan_index.display().to_string();
        assert_eq!(
            report.bridge_registry_path.as_deref(),
            Some(bridge_registry_path_text.as_str())
        );
        assert_eq!(
            report.host_bridge_plan_index_path.as_deref(),
            Some(host_bridge_plan_index_path_text.as_str())
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
        assert!(host_bridge_plan_index_text
            .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
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
