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
    pub manifest_copy_path: Option<String>,
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
    pub artifacts_checked: usize,
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
    let cpu_target_machine_os =
        parse_required_toml_string(&source, "cpu_target_machine_os", path)?;
    let cpu_target_object_format =
        parse_required_toml_string(&source, "cpu_target_object_format", path)?;
    let cpu_target_calling_abi =
        parse_required_toml_string(&source, "cpu_target_calling_abi", path)?;
    let cpu_target_clang = parse_required_toml_string(&source, "cpu_target_clang", path)?;
    let cpu_target_cross = parse_required_toml_bool(&source, "cpu_target_cross", path)?;
    let compile_cache_status = parse_optional_toml_string(&source, "compile_cache_status");
    let compile_cache_key = parse_optional_toml_string(&source, "compile_cache_key");
    let compile_cache_root = parse_optional_toml_string(&source, "compile_cache_root");

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
        artifacts_checked: artifacts.len(),
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

extern int64_t nuis_yir_entry(void);

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
    out.push_str(
        r#"

int main(void) {
    return (int)nuis_yir_entry();
}
"#,
    );
    out
}

fn collect_host_ffi_symbols(ast: &AstModule) -> BTreeMap<String, AstExternFunction> {
    let mut out = BTreeMap::new();
    for function in &ast.externs {
        out.insert(function.name.clone(), function.clone());
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
        resolve_cpu_build_target_from_abi, verify_build_manifest, BuildManifestCacheInfo,
        BuildManifestContext, BuildManifestProjectInfo, CompileArtifacts, CpuBuildTarget,
    };
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

        let linux =
            resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.sysv64").unwrap();
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
                    manifest_copy_path: None,
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
    }
}
