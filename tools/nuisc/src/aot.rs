use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
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
    pub abi_entries: Vec<(String, String)>,
    pub plan_summary: Option<String>,
    pub effective_input: Option<String>,
    pub manifest_copy_path: Option<String>,
    pub plan_index_path: Option<String>,
    pub organization_index_path: Option<String>,
    pub exchange_index_path: Option<String>,
    pub modules_index_path: Option<String>,
    pub links_index_path: Option<String>,
    pub host_ffi_index_path: Option<String>,
    pub abi_index_path: Option<String>,
}

pub struct BuildManifestContext {
    pub input_path: String,
    pub output_dir: String,
    pub loaded_nustar: Vec<String>,
    pub compile_cache: Option<BuildManifestCacheInfo>,
    pub project: Option<BuildManifestProjectInfo>,
    pub cpu_target: CpuBuildTarget,
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
        cross_compile: false,
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
        cross_compile: registered.machine_arch != host_machine_arch()
            || registered.machine_os != host_machine_os(),
    })
}

pub fn resolve_cpu_build_target_from_target(
    registry_root: &Path,
    target: &str,
) -> Result<CpuBuildTarget, String> {
    let manifest = crate::registry::load_manifest_for_domain(registry_root, "cpu")?;
    let registered = crate::registry::registered_abi_target_for_clang(&manifest, target)?;
    Ok(CpuBuildTarget {
        abi: registered.abi,
        machine_arch: registered.machine_arch.clone(),
        machine_os: registered.machine_os.clone(),
        object_format: registered.object_format,
        calling_abi: registered.calling_abi,
        clang_target: registered.clang_target,
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
    let layout = output_layout(input, output_dir);
    let (binary_path, packaging_mode) = if requires_window_bundle(yir) {
        (
            layout.binary_stub_path.display().to_string(),
            "window-aot-bundle".to_owned(),
        )
    } else {
        (
            layout.binary_stub_path.display().to_string(),
            "native-cpu-llvm".to_owned(),
        )
    };
    Ok(CompileArtifacts {
        ast_path: layout.ast_path.display().to_string(),
        nir_path: layout.nir_path.display().to_string(),
        yir_path: layout.yir_path.display().to_string(),
        llvm_ir_path: layout.llvm_ir_path.display().to_string(),
        binary_path,
        packaging_mode,
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
    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_secs();
    let engine = crate::engine::default_engine();
    let vcs = detect_vcs_info(&context.input_path, &context.output_dir);

    let mut loaded_nustar = context.loaded_nustar.clone();
    loaded_nustar.sort();
    loaded_nustar.dedup();

    let artifacts = vec![
        ("ast".to_owned(), PathBuf::from(&written.ast_path)),
        ("nir".to_owned(), PathBuf::from(&written.nir_path)),
        ("yir".to_owned(), PathBuf::from(&written.yir_path)),
        ("llvm_ir".to_owned(), PathBuf::from(&written.llvm_ir_path)),
        ("binary".to_owned(), PathBuf::from(&written.binary_path)),
    ];

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
    out.push('\n');
    out.push_str("[artifacts]\n");
    for (kind, artifact_path) in &artifacts {
        out.push_str(&format!(
            "{kind} = \"{}\"\n",
            escape_toml_string(&artifact_path.display().to_string())
        ));
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
        if let Some(value) = &project.links_index_path {
            out.push_str(&format!(
                "links_index = \"{}\"\n",
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

    fs::write(&path, out)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(path.display().to_string())
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

fn render_string_array(values: &[String]) -> String {
    let quoted = values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml_string(value)))
        .collect::<Vec<_>>();
    format!("[{}]", quoted.join(", "))
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
    pub cpu_target_abi: String,
    pub cpu_target_machine_arch: String,
    pub cpu_target_machine_os: String,
    pub cpu_target_object_format: String,
    pub cpu_target_calling_abi: String,
    pub cpu_target_clang: String,
    pub cpu_target_cross: bool,
    pub compile_cache_status: Option<String>,
    pub compile_cache_key: Option<String>,
    pub compile_cache_root: Option<String>,
    pub project_plan_index: Option<String>,
    pub artifacts_checked: usize,
    pub project_metadata_checked: usize,
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
    let compile_cache_status = parse_optional_toml_string(&source, "compile_cache_status");
    let compile_cache_key = parse_optional_toml_string(&source, "compile_cache_key");
    let compile_cache_root = parse_optional_toml_string(&source, "compile_cache_root");
    let project_plan_index = parse_optional_toml_string(&source, "plan_index");
    let project_plan_summary = parse_optional_toml_string(&source, "plan_summary");

    let artifacts = parse_artifact_hash_blocks(&source, path)?;
    if artifacts.is_empty() {
        return Err(format!(
            "`{}` does not contain any `[[artifact_hash]]` blocks",
            path.display()
        ));
    }

    for item in &artifacts {
        let bytes = fs::read(&item.path)
            .map_err(|error| format!("failed to read artifact `{}`: {error}", item.path))?;
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
                return Err(format!(
                    "project plan index `{}` summary mismatch: expected line `{}`",
                    plan_index, expected
                ));
            }
        }
        project_metadata_checked += 1;
    }

    Ok(BuildManifestVerifyReport {
        schema,
        input,
        output_dir,
        packaging_mode,
        cpu_target_abi,
        cpu_target_machine_arch,
        cpu_target_machine_os,
        cpu_target_object_format,
        cpu_target_calling_abi,
        cpu_target_clang,
        cpu_target_cross,
        compile_cache_status,
        compile_cache_key,
        compile_cache_root,
        project_plan_index,
        artifacts_checked: artifacts.len(),
        project_metadata_checked,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtifactHashRow {
    kind: String,
    path: String,
    bytes: usize,
    fnv1a64: String,
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

fn parse_optional_toml_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                return Some(value[1..value.len() - 1].to_owned());
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

fn parse_required_map_string(
    values: &BTreeMap<String, String>,
    key: &str,
    manifest_path: &Path,
) -> Result<String, String> {
    let value = values.get(key).ok_or_else(|| {
        format!(
            "`{}` artifact_hash block is missing required key `{key}`",
            manifest_path.display()
        )
    })?;
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return Ok(value[1..value.len() - 1].to_owned());
    }
    Err(format!(
        "`{}` artifact_hash key `{key}` must be a quoted string",
        manifest_path.display()
    ))
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

fn requires_window_bundle(yir: &YirModule) -> bool {
    yir.nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "window")
}

fn host_machine_arch() -> &'static str {
    match std::env::consts::ARCH {
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
    match (machine_arch, machine_os) {
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
    match (machine_arch, machine_os) {
        ("arm64", "darwin") => "aarch64-apple-darwin".to_owned(),
        ("arm64", "linux") => "aarch64-unknown-linux-gnu".to_owned(),
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

fn c_shim_source(ast: &AstModule) -> String {
    let mut out = String::new();
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
static int64_t nuis_host_command_len = 0;
static pid_t nuis_host_subprocess_pids[256];
static int64_t nuis_host_subprocess_status_slots[256];
static int nuis_host_subprocess_done[256];
static int64_t nuis_host_subprocess_len = 0;

static int64_t nuis_host_text_register(const char* text) {
    if (text == NULL) return 0;
    if (nuis_host_text_len >= 4096) return 0;
    size_t size = strlen(text) + 1;
    char* copy = (char*)malloc(size);
    if (copy == NULL) return 0;
    memcpy(copy, text, size);
    nuis_host_text_slots[nuis_host_text_len] = copy;
    nuis_host_text_len += 1;
    return nuis_host_text_len;
}

int64_t nuis_host_text_lift(const char* text) {
    return nuis_host_text_register(text);
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
    return (int64_t)strlen(nuis_host_text_lookup(handle));
}

static int64_t nuis_host_text_concat(int64_t lhs_handle, int64_t rhs_handle) {
    const char* lhs = nuis_host_text_lookup(lhs_handle);
    const char* rhs = nuis_host_text_lookup(rhs_handle);
    size_t lhs_len = lhs != NULL ? strlen(lhs) : 0;
    size_t rhs_len = rhs != NULL ? strlen(rhs) : 0;
    size_t total = lhs_len + rhs_len + 1;
    char* buffer = (char*)malloc(total);
    if (buffer == NULL) return 0;
    snprintf(buffer, total, "%s%s", lhs != NULL ? lhs : "", rhs != NULL ? rhs : "");
    int64_t handle = nuis_host_text_register(buffer);
    free(buffer);
    return handle;
}

static int64_t nuis_host_file_open(int64_t path_handle, int64_t flags) {
    const char* path = nuis_host_text_lookup(path_handle);
    if (path == NULL || path[0] == '\0') return 0;
    int fd = open(path, (int)flags, 0644);
    return fd >= 0 ? (int64_t)fd : 0;
}

static int64_t nuis_host_file_read(int64_t file_handle, int64_t buffer_handle, int64_t len) {
    (void)buffer_handle;
    if (file_handle < 0 || len <= 0) return 0;
    char scratch[4096];
    size_t read_len = (size_t)len;
    if (read_len > sizeof(scratch)) read_len = sizeof(scratch);
    ssize_t got = read((int)file_handle, scratch, read_len);
    return got > 0 ? (int64_t)got : 0;
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
    (void)nuis_network_try_connect_probe(local_port, remote_port, connect_timeout_ms);
    return local_port + remote_port + connect_timeout_ms;
}

static int64_t nuis_host_network_accept_probe(
    int64_t local_port,
    int64_t read_timeout_ms,
    int64_t write_timeout_ms
) {
    if (local_port <= 0) return 0;
    if (read_timeout_ms < 0 || write_timeout_ms < 0) return 0;
    (void)nuis_network_try_accept_probe(local_port, read_timeout_ms, write_timeout_ms);
    return local_port + read_timeout_ms + write_timeout_ms;
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
    (void)nuis_network_try_send_probe(stream_window, send_window);
    return stream_window + send_window + remote_port;
}

static int64_t nuis_host_network_recv_probe(
    int64_t stream_window,
    int64_t recv_window,
    int64_t local_port
) {
    if (stream_window <= 0 || recv_window <= 0 || local_port <= 0) return 0;
    (void)local_port;
    (void)nuis_network_try_recv_probe(stream_window, recv_window);
    return stream_window + recv_window + local_port;
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

static pid_t nuis_host_spawn_shell(char* program) {
    if (program == NULL || program[0] == '\0') return -1;
    pid_t pid = fork();
    if (pid < 0) return -1;
    if (pid == 0) {
        execl("/bin/sh", "sh", "-c", program, (char*)NULL);
        _exit(127);
    }
    return pid;
}

static int64_t nuis_host_command_spawn(int64_t program_handle, int64_t argv_handle) {
    if (nuis_host_command_len >= 256) return 0;
    char* command = nuis_host_build_shell_command(program_handle, argv_handle, 0);
    pid_t pid = nuis_host_spawn_shell(command);
    free(command);
    if (pid < 0) return 0;
    nuis_host_command_pids[nuis_host_command_len] = pid;
    nuis_host_command_status_slots[nuis_host_command_len] = 0;
    nuis_host_command_done[nuis_host_command_len] = 0;
    nuis_host_command_len += 1;
    return nuis_host_command_len;
}

static int64_t nuis_host_command_status(int64_t command_handle) {
    if (command_handle <= 0 || command_handle > nuis_host_command_len) return 0;
    int64_t idx = command_handle - 1;
    if (nuis_host_command_done[idx]) return nuis_host_command_status_slots[idx];
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
    int status = 0;
    pid_t result = waitpid(nuis_host_command_pids[idx], &status, 0);
    if (result < 0) return 0;
    nuis_host_command_done[idx] = 1;
    nuis_host_command_status_slots[idx] = (int64_t)status;
    return nuis_host_command_status_slots[idx];
}

static int64_t nuis_host_command_wait_exit(int64_t command_handle) {
    int64_t raw = nuis_host_command_wait(command_handle);
    return nuis_host_process_exit_code(raw);
}

static int64_t nuis_host_subprocess_spawn(int64_t program_handle, int64_t argv_handle, int64_t env_handle) {
    if (nuis_host_subprocess_len >= 256) return 0;
    char* command = nuis_host_build_shell_command(program_handle, argv_handle, env_handle);
    pid_t pid = nuis_host_spawn_shell(command);
    free(command);
    if (pid < 0) return 0;
    nuis_host_subprocess_pids[nuis_host_subprocess_len] = pid;
    nuis_host_subprocess_status_slots[nuis_host_subprocess_len] = 0;
    nuis_host_subprocess_done[nuis_host_subprocess_len] = 0;
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
    int status = 0;
    pid_t result = waitpid(nuis_host_subprocess_pids[idx], &status, 0);
    if (result < 0) return 0;
    nuis_host_subprocess_done[idx] = 1;
    nuis_host_subprocess_status_slots[idx] = (int64_t)status;
    return nuis_host_subprocess_status_slots[idx];
}

static int64_t nuis_host_subprocess_join_exit(int64_t process_handle) {
    int64_t raw = nuis_host_subprocess_join(process_handle);
    return nuis_host_process_exit_code(raw);
}

static int64_t nuis_host_wall_time_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) return 0;
    return (int64_t)ts.tv_sec * 1000000000LL + (int64_t)ts.tv_nsec;
}

static int64_t nuis_host_monotonic_time_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) return 0;
    return (int64_t)ts.tv_sec * 1000000000LL + (int64_t)ts.tv_nsec;
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
    out.push_str(
        r#"

int main(int argc, char** argv) {
    nuis_argc = argc;
    nuis_argv = argv;
    return (int)nuis_yir_entry();
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

fn collect_host_ffi_symbols(ast: &AstModule) -> BTreeMap<String, AstExternFunction> {
    let mut out = BTreeMap::new();
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
        c_shim_source, resolve_cpu_build_target_from_abi, verify_build_manifest,
        BuildManifestCacheInfo, BuildManifestContext, BuildManifestProjectInfo, CompileArtifacts,
        CpuBuildTarget,
    };
    use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef};
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
            "manifest_schema = \"nustar-manifest-v1\"\npackage_id = \"official.cpu\"\ndomain_family = \"cpu\"\nfrontend = \"nustar-cpu\"\nentry_crate = \"crates/yir-domain-cpu\"\nast_entry = \"cpu.ast.bootstrap.v1\"\nnir_entry = \"cpu.nir.bootstrap.v1\"\nyir_lowering_entry = \"cpu.yir.lowering.v1\"\npart_verify_entry = \"cpu.verify.partial.v1\"\nast_surface = [\"cpu.mod-ast.v1\"]\nnir_surface = [\"nir.cpu.surface.v1\"]\nyir_lowering = [\"yir.cpu.lowering.v1\"]\npart_verify = [\"verify.cpu.contract.v1\"]\nbinary_extension = \"nustar\"\npackage_layout = \"single-envelope\"\nmachine_abi_policy = \"exact-match\"\nabi_profiles = [\"cpu.arm64.apple_aapcs64\", \"cpu.x86_64.sysv64\", \"cpu.x86_64.win64\"]\nabi_capabilities = [\"cpu.arm64.apple_aapcs64:op:cpu.*\", \"cpu.x86_64.sysv64:op:cpu.*\", \"cpu.x86_64.win64:op:cpu.*\"]\nabi_targets = [\"cpu.arm64.apple_aapcs64:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin\", \"cpu.x86_64.sysv64:arch=x86_64|os=linux|object=elf|calling=sysv64|clang=x86_64-unknown-linux-gnu\", \"cpu.x86_64.win64:arch=x86_64|os=windows|object=coff|calling=win64|clang=x86_64-pc-windows-msvc\"]\nimplementation_kinds = [\"native-stub\"]\nloader_entry = \"nustar.bootstrap.v1\"\nloader_abi = \"nustar-loader-v1\"\nhost_ffi_surface = []\nhost_ffi_abis = []\nhost_ffi_bridge = \"none\"\nsupport_surface = []\nsupport_profile_slots = []\ndefault_lanes = []\nprofiles = [\"aot\"]\nresource_families = [\"cpu\", \"cpu.arm64\"]\nunit_types = [\"Main\"]\nlowering_targets = [\"llvm\"]\nops = [\"cpu.const\"]\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn resolve_cpu_build_target_for_known_abis() {
        let registry_root = registry_root();
        let apple =
            resolve_cpu_build_target_from_abi(&registry_root, "cpu.arm64.apple_aapcs64").unwrap();
        assert_eq!(apple.machine_arch, "arm64");
        assert_eq!(apple.machine_os, "darwin");
        assert_eq!(apple.clang_target, "aarch64-apple-darwin");

        let linux = resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.sysv64").unwrap();
        assert_eq!(linux.machine_arch, "x86_64");
        assert_eq!(linux.machine_os, "linux");
        assert_eq!(linux.object_format, "elf");
        assert_eq!(linux.calling_abi, "sysv64");

        let windows =
            resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.win64").unwrap();
        assert_eq!(windows.machine_os, "windows");
        assert_eq!(windows.clang_target, "x86_64-pc-windows-msvc");
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
                    abi_entries: vec![("cpu".to_owned(), cpu_target.abi.clone())],
                    plan_summary: None,
                    effective_input: None,
                    manifest_copy_path: None,
                    plan_index_path: None,
                    organization_index_path: None,
                    exchange_index_path: None,
                    modules_index_path: None,
                    links_index_path: None,
                    host_ffi_index_path: None,
                    abi_index_path: None,
                }),
                cpu_target: cpu_target.clone(),
            },
        )
        .unwrap();
        let report = verify_build_manifest(PathBuf::from(manifest).as_path()).unwrap();
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_argv_count".to_owned(),
                    params: Vec::new(),
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_cwd_handle".to_owned(),
                    params: Vec::new(),
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_monotonic_time_ns".to_owned(),
                    params: Vec::new(),
                    return_type: i64_ty(),
                },
            ],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            functions: Vec::new(),
        };
        let shim = c_shim_source(&ast);
        assert!(shim.contains("int main(int argc, char** argv)"));
        assert!(shim.contains("nuis_argc = argc;"));
        assert!(shim.contains("return nuis_host_argv_count();"));
        assert!(shim.contains("return nuis_host_cwd_handle();"));
        assert!(shim.contains("return nuis_host_monotonic_time_ns();"));
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_env_has".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "key_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_basename".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_filename".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_parent".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_has_parent".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_is_basename_only".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_depth".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_is_empty".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_is_dot".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_is_dotdot".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_is_relative".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_is_root".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_stem".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_extension".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_has_extension".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_starts_with_dot".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_ends_with_slash".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_is_hidden".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_stat_mode".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
            ],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_tty_width".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "fd".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
            ],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_network_close".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
            ],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            functions: Vec::new(),
        };

        let shim = c_shim_source(&ast);
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_dir_open".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_dir_create".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_dir_remove".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
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
                },
                AstExternFunction {
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
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_path_remove".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_temp_file_handle".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "prefix_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
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
                },
            ],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
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
        assert!(shim.contains("static char* nuis_host_build_shell_command("));
        assert!(shim.contains("env %s %s %s"));
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_command_wait_exit".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "command_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
                AstExternFunction {
                    abi: "c".to_owned(),
                    interface: None,
                    name: "host_subprocess_join_exit".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "process_handle".to_owned(),
                        ty: i64_ty(),
                    }],
                    return_type: i64_ty(),
                },
            ],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            functions: Vec::new(),
        };
        let shim = c_shim_source(&ast);
        assert!(shim.contains("static int64_t nuis_host_command_wait_exit("));
        assert!(shim.contains("static int64_t nuis_host_subprocess_join_exit("));
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![AstExternFunction {
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
            }],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            functions: Vec::new(),
        };
        let shim = c_shim_source(&ast);
        assert!(shim.contains("static int64_t nuis_host_text_concat("));
        assert!(shim.contains("return nuis_host_text_concat(lhs_handle, rhs_handle);"));
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![AstExternFunction {
                abi: "c".to_owned(),
                interface: None,
                name: "usleep".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "usec".to_owned(),
                    ty: ty("i64"),
                }],
                return_type: ty("i32"),
            }],
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
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
            uses: Vec::new(),
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: Vec::new(),
            extern_interfaces: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            functions: vec![nuis_semantics::model::AstFunction {
                name: "main".to_owned(),
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
                is_async: false,
                generic_params: Vec::new(),
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
