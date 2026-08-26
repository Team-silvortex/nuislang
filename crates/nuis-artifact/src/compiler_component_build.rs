use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

use crate::{
    parse_build_manifest, read_compiler_stage_handoff,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError,
};

#[path = "compiler_component_build_identity.rs"]
mod identity;

use identity::{
    component_build_identity, dependency_closure_identity, reproducible_build_identity, sha256_hex,
};

pub const COMPILER_COMPONENT_BUILD_PROTOCOL: &str = "nuis-compiler-component-build-v1";
pub const COMPILER_COMPONENT_DRIVER_CONTRACT: &str = "nuis-stage0-stage1-driver-v1";
pub const COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT: &str =
    "nuis-compiler-dependency-closure-v1";
pub const COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT: &str =
    "nuis-compiler-component-reproducible-build-v1";
pub const COMPILER_COMPONENT_BUILD_FILE: &str = "nuis.compiler-component-build.toml";
pub const COMPILER_COMPONENT_STAGE0_ROLE: &str = "stage0";
pub const COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE: &str = "stage1-candidate";
const MAX_DEPENDENCIES: usize = 4096;

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentDependencyInput<'a> {
    pub kind: &'a str,
    pub identity: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentDependency {
    pub ordinal: usize,
    pub kind: String,
    pub identity: String,
    pub content_bytes: usize,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentBuildInput<'a> {
    pub stage_role: &'a str,
    pub bootstrap_subset_protocol: &'a str,
    pub component_id: &'a str,
    pub component_domain: &'a str,
    pub component_unit: &'a str,
    pub producer_id: &'a str,
    pub compiler_image: &'a [u8],
    pub stage_handoff_file: &'a str,
    pub stage_handoff_bundle_sha256: &'a str,
    pub build_manifest_file: &'a str,
    pub build_manifest: &'a [u8],
    pub compiled_artifact_file: &'a str,
    pub compiled_artifact: &'a [u8],
    pub native_binary_file: &'a str,
    pub native_binary: &'a [u8],
    pub dependencies: &'a [CompilerComponentDependencyInput<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentCandidatePromotionInput<'a> {
    pub stage0: &'a CompilerComponentBuild,
    pub producer_id: &'a str,
    pub compiler_image: &'a [u8],
    pub stage_handoff_bundle_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentBuild {
    pub protocol: String,
    pub driver_contract: String,
    pub stage_role: String,
    pub bootstrap_subset_protocol: String,
    pub component_id: String,
    pub component_domain: String,
    pub component_unit: String,
    pub producer_id: String,
    pub compiler_image_bytes: usize,
    pub compiler_image_sha256: String,
    pub stage_handoff_file: String,
    pub stage_handoff_bundle_sha256: String,
    pub build_manifest_file: String,
    pub build_manifest_bytes: usize,
    pub build_manifest_sha256: String,
    pub compiled_artifact_file: String,
    pub compiled_artifact_bytes: usize,
    pub compiled_artifact_sha256: String,
    pub native_binary_file: String,
    pub native_binary_bytes: usize,
    pub native_binary_sha256: String,
    pub dependency_closure_contract: String,
    pub dependency_count: usize,
    pub dependency_closure_sha256: String,
    pub reproducible_identity_contract: String,
    pub reproducible_build_sha256: String,
    pub record_sha256: String,
    pub dependencies: Vec<CompilerComponentDependency>,
}

pub fn build_compiler_component_build(
    input: &CompilerComponentBuildInput<'_>,
) -> Result<CompilerComponentBuild, ArtifactError> {
    for (label, value) in [
        ("bootstrap subset protocol", input.bootstrap_subset_protocol),
        ("component id", input.component_id),
        ("component domain", input.component_domain),
        ("component unit", input.component_unit),
        ("producer id", input.producer_id),
    ] {
        validate_header_value(value, label)?;
    }
    for (label, file) in [
        ("stage handoff", input.stage_handoff_file),
        ("build manifest", input.build_manifest_file),
        ("compiled artifact", input.compiled_artifact_file),
        ("native binary", input.native_binary_file),
    ] {
        validate_relative_file_name(file, label)?;
    }
    validate_sha256(input.stage_handoff_bundle_sha256, "stage handoff bundle")?;
    for (label, bytes) in [
        ("compiler image", input.compiler_image),
        ("build manifest", input.build_manifest),
        ("compiled artifact", input.compiled_artifact),
        ("native binary", input.native_binary),
    ] {
        if bytes.is_empty() {
            return Err(ArtifactError::new(format!(
                "compiler component build {label} cannot be empty"
            )));
        }
    }
    if input.dependencies.is_empty() || input.dependencies.len() > MAX_DEPENDENCIES {
        return Err(ArtifactError::new(format!(
            "compiler component build requires 1..={MAX_DEPENDENCIES} dependencies"
        )));
    }

    let mut ordered = input.dependencies.to_vec();
    ordered.sort_by(|lhs, rhs| lhs.kind.cmp(rhs.kind).then(lhs.identity.cmp(rhs.identity)));
    let mut seen = BTreeSet::new();
    let mut dependencies = Vec::with_capacity(ordered.len());
    for (ordinal, dependency) in ordered.iter().enumerate() {
        validate_dependency_kind(dependency.kind)?;
        validate_dependency_identity(dependency.identity)?;
        if dependency.bytes.is_empty() {
            return Err(ArtifactError::new(format!(
                "compiler dependency `{}` cannot be empty",
                dependency.identity
            )));
        }
        if !seen.insert((dependency.kind, dependency.identity)) {
            return Err(ArtifactError::new(format!(
                "duplicate compiler dependency `{}:{}`",
                dependency.kind, dependency.identity
            )));
        }
        dependencies.push(CompilerComponentDependency {
            ordinal,
            kind: dependency.kind.to_owned(),
            identity: dependency.identity.to_owned(),
            content_bytes: dependency.bytes.len(),
            content_sha256: sha256_hex(dependency.bytes),
        });
    }

    let dependency_closure_sha256 = dependency_closure_identity(input.component_id, &dependencies);
    let mut build = CompilerComponentBuild {
        protocol: COMPILER_COMPONENT_BUILD_PROTOCOL.to_owned(),
        driver_contract: COMPILER_COMPONENT_DRIVER_CONTRACT.to_owned(),
        stage_role: input.stage_role.to_owned(),
        bootstrap_subset_protocol: input.bootstrap_subset_protocol.to_owned(),
        component_id: input.component_id.to_owned(),
        component_domain: input.component_domain.to_owned(),
        component_unit: input.component_unit.to_owned(),
        producer_id: input.producer_id.to_owned(),
        compiler_image_bytes: input.compiler_image.len(),
        compiler_image_sha256: sha256_hex(input.compiler_image),
        stage_handoff_file: input.stage_handoff_file.to_owned(),
        stage_handoff_bundle_sha256: input.stage_handoff_bundle_sha256.to_owned(),
        build_manifest_file: input.build_manifest_file.to_owned(),
        build_manifest_bytes: input.build_manifest.len(),
        build_manifest_sha256: sha256_hex(input.build_manifest),
        compiled_artifact_file: input.compiled_artifact_file.to_owned(),
        compiled_artifact_bytes: input.compiled_artifact.len(),
        compiled_artifact_sha256: sha256_hex(input.compiled_artifact),
        native_binary_file: input.native_binary_file.to_owned(),
        native_binary_bytes: input.native_binary.len(),
        native_binary_sha256: sha256_hex(input.native_binary),
        dependency_closure_contract: COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT.to_owned(),
        dependency_count: dependencies.len(),
        dependency_closure_sha256,
        reproducible_identity_contract: COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT
            .to_owned(),
        reproducible_build_sha256: String::new(),
        record_sha256: String::new(),
        dependencies,
    };
    build.reproducible_build_sha256 = reproducible_build_identity(&build);
    build.record_sha256 = component_build_identity(&build);
    validate_compiler_component_build(&build)?;
    Ok(build)
}

pub fn promote_compiler_component_candidate(
    input: &CompilerComponentCandidatePromotionInput<'_>,
) -> Result<CompilerComponentBuild, ArtifactError> {
    validate_compiler_component_build(input.stage0)?;
    if input.stage0.stage_role != COMPILER_COMPONENT_STAGE0_ROLE {
        return Err(ArtifactError::new(
            "compiler candidate promotion requires a verified stage0 component",
        ));
    }
    validate_header_value(input.producer_id, "candidate producer id")?;
    if input.producer_id == input.stage0.producer_id {
        return Err(ArtifactError::new(
            "compiler candidate promotion requires a distinct producer id",
        ));
    }
    if input.compiler_image.is_empty() {
        return Err(ArtifactError::new(
            "compiler candidate promotion requires a non-empty Nuis compiler image",
        ));
    }
    validate_sha256(
        input.stage_handoff_bundle_sha256,
        "candidate stage handoff bundle",
    )?;
    if input.stage_handoff_bundle_sha256 != input.stage0.stage_handoff_bundle_sha256 {
        return Err(ArtifactError::new(
            "compiler candidate promotion must preserve the stage0 semantic bundle",
        ));
    }

    let mut candidate = input.stage0.clone();
    candidate.stage_role = COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE.to_owned();
    candidate.producer_id = input.producer_id.to_owned();
    candidate.compiler_image_bytes = input.compiler_image.len();
    candidate.compiler_image_sha256 = sha256_hex(input.compiler_image);
    candidate.stage_handoff_bundle_sha256 = input.stage_handoff_bundle_sha256.to_owned();
    candidate.reproducible_build_sha256 = reproducible_build_identity(&candidate);
    candidate.record_sha256 = component_build_identity(&candidate);
    validate_compiler_component_build(&candidate)?;
    Ok(candidate)
}

pub fn render_compiler_component_build(build: &CompilerComponentBuild) -> String {
    let mut out = format!(
        "protocol = \"{}\"\ndriver_contract = \"{}\"\nstage_role = \"{}\"\nbootstrap_subset_protocol = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\nproducer_id = \"{}\"\ncompiler_image_bytes = {}\ncompiler_image_sha256 = \"{}\"\nstage_handoff_file = \"{}\"\nstage_handoff_bundle_sha256 = \"{}\"\nbuild_manifest_file = \"{}\"\nbuild_manifest_bytes = {}\nbuild_manifest_sha256 = \"{}\"\ncompiled_artifact_file = \"{}\"\ncompiled_artifact_bytes = {}\ncompiled_artifact_sha256 = \"{}\"\nnative_binary_file = \"{}\"\nnative_binary_bytes = {}\nnative_binary_sha256 = \"{}\"\ndependency_closure_contract = \"{}\"\ndependency_count = {}\ndependency_closure_sha256 = \"{}\"\nreproducible_identity_contract = \"{}\"\nreproducible_build_sha256 = \"{}\"\nrecord_sha256 = \"{}\"\n",
        build.protocol,
        build.driver_contract,
        build.stage_role,
        escape_toml_string(&build.bootstrap_subset_protocol),
        escape_toml_string(&build.component_id),
        escape_toml_string(&build.component_domain),
        escape_toml_string(&build.component_unit),
        escape_toml_string(&build.producer_id),
        build.compiler_image_bytes,
        build.compiler_image_sha256,
        build.stage_handoff_file,
        build.stage_handoff_bundle_sha256,
        build.build_manifest_file,
        build.build_manifest_bytes,
        build.build_manifest_sha256,
        build.compiled_artifact_file,
        build.compiled_artifact_bytes,
        build.compiled_artifact_sha256,
        escape_toml_string(&build.native_binary_file),
        build.native_binary_bytes,
        build.native_binary_sha256,
        build.dependency_closure_contract,
        build.dependency_count,
        build.dependency_closure_sha256,
        build.reproducible_identity_contract,
        build.reproducible_build_sha256,
        build.record_sha256,
    );
    for dependency in &build.dependencies {
        out.push_str(&format!(
            "\n[[dependency]]\nordinal = {}\nkind = \"{}\"\nidentity = \"{}\"\ncontent_bytes = {}\ncontent_sha256 = \"{}\"\n",
            dependency.ordinal,
            dependency.kind,
            escape_toml_string(&dependency.identity),
            dependency.content_bytes,
            dependency.content_sha256,
        ));
    }
    out
}

pub fn parse_compiler_component_build(
    path: &Path,
) -> Result<CompilerComponentBuild, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!("failed to read `{}`: {error}", path.display()))
    })?;
    parse_compiler_component_build_from_source(&source, path)
}

pub fn parse_compiler_component_build_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentBuild, ArtifactError> {
    validate_text(source, path)?;
    let build = CompilerComponentBuild {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        driver_contract: parse_required_toml_string(source, "driver_contract", path)?,
        stage_role: parse_required_toml_string(source, "stage_role", path)?,
        bootstrap_subset_protocol: parse_required_toml_string(
            source,
            "bootstrap_subset_protocol",
            path,
        )?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        component_domain: parse_required_toml_string(source, "component_domain", path)?,
        component_unit: parse_required_toml_string(source, "component_unit", path)?,
        producer_id: parse_required_toml_string(source, "producer_id", path)?,
        compiler_image_bytes: parse_required_toml_usize(source, "compiler_image_bytes", path)?,
        compiler_image_sha256: parse_required_toml_string(source, "compiler_image_sha256", path)?,
        stage_handoff_file: parse_required_toml_string(source, "stage_handoff_file", path)?,
        stage_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "stage_handoff_bundle_sha256",
            path,
        )?,
        build_manifest_file: parse_required_toml_string(source, "build_manifest_file", path)?,
        build_manifest_bytes: parse_required_toml_usize(source, "build_manifest_bytes", path)?,
        build_manifest_sha256: parse_required_toml_string(source, "build_manifest_sha256", path)?,
        compiled_artifact_file: parse_required_toml_string(source, "compiled_artifact_file", path)?,
        compiled_artifact_bytes: parse_required_toml_usize(
            source,
            "compiled_artifact_bytes",
            path,
        )?,
        compiled_artifact_sha256: parse_required_toml_string(
            source,
            "compiled_artifact_sha256",
            path,
        )?,
        native_binary_file: parse_required_toml_string(source, "native_binary_file", path)?,
        native_binary_bytes: parse_required_toml_usize(source, "native_binary_bytes", path)?,
        native_binary_sha256: parse_required_toml_string(source, "native_binary_sha256", path)?,
        dependency_closure_contract: parse_required_toml_string(
            source,
            "dependency_closure_contract",
            path,
        )?,
        dependency_count: parse_required_toml_usize(source, "dependency_count", path)?,
        dependency_closure_sha256: parse_required_toml_string(
            source,
            "dependency_closure_sha256",
            path,
        )?,
        reproducible_identity_contract: parse_required_toml_string(
            source,
            "reproducible_identity_contract",
            path,
        )?,
        reproducible_build_sha256: parse_required_toml_string(
            source,
            "reproducible_build_sha256",
            path,
        )?,
        record_sha256: parse_required_toml_string(source, "record_sha256", path)?,
        dependencies: parse_dependency_blocks(source, path)?,
    };
    validate_compiler_component_build(&build)?;
    if render_compiler_component_build(&build) != source {
        return Err(ArtifactError::new(format!(
            "compiler component build `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(build)
}

pub fn read_compiler_component_build(path: &Path) -> Result<CompilerComponentBuild, ArtifactError> {
    let build = parse_compiler_component_build(path)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let build_manifest_path = checked_payload_path(root, &build.build_manifest_file)?;
    let compiled_artifact_path = checked_payload_path(root, &build.compiled_artifact_file)?;
    let native_binary_path = checked_payload_path(root, &build.native_binary_file)?;
    let stage_handoff_path = checked_payload_path(root, &build.stage_handoff_file)?;

    verify_file(
        &build_manifest_path,
        build.build_manifest_bytes,
        &build.build_manifest_sha256,
        "build manifest",
    )?;
    verify_file(
        &compiled_artifact_path,
        build.compiled_artifact_bytes,
        &build.compiled_artifact_sha256,
        "compiled artifact",
    )?;
    verify_file(
        &native_binary_path,
        build.native_binary_bytes,
        &build.native_binary_sha256,
        "native binary",
    )?;

    let manifest = parse_build_manifest(&build_manifest_path)?;
    if manifest.artifact_binary_name != build.native_binary_file {
        return Err(ArtifactError::new(format!(
            "compiler component build native binary `{}` disagrees with build manifest `{}`",
            build.native_binary_file, manifest.artifact_binary_name
        )));
    }
    if Path::new(&manifest.artifact_path)
        .file_name()
        .and_then(|value| value.to_str())
        != Some(build.compiled_artifact_file.as_str())
    {
        return Err(ArtifactError::new(
            "compiler component build compiled artifact disagrees with build manifest",
        ));
    }

    let (handoff, _) = read_compiler_stage_handoff(&stage_handoff_path)?;
    if handoff.bundle_sha256 != build.stage_handoff_bundle_sha256
        || handoff.producer_id != build.producer_id
        || handoff.module_domain != build.component_domain
        || handoff.module_unit != build.component_unit
    {
        return Err(ArtifactError::new(
            "compiler component build stage handoff identity does not match the component record",
        ));
    }
    Ok(build)
}

pub fn verify_compiler_component_build_image(
    build: &CompilerComponentBuild,
    compiler_image: &[u8],
) -> Result<(), ArtifactError> {
    if compiler_image.len() != build.compiler_image_bytes
        || sha256_hex(compiler_image) != build.compiler_image_sha256
    {
        return Err(ArtifactError::new(
            "compiler component build compiler image identity mismatch",
        ));
    }
    Ok(())
}

fn parse_dependency_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerComponentDependency>, ArtifactError> {
    let mut dependencies = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[dependency]]" {
            if in_block {
                dependencies.push(parse_dependency(&values, path)?);
                values.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                dependencies.push(parse_dependency(&values, path)?);
                values.clear();
                in_block = false;
            }
            continue;
        }
        if in_block && !line.is_empty() && !line.starts_with('#') {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_owned();
                if values
                    .insert(key.clone(), value.trim().to_owned())
                    .is_some()
                {
                    return Err(ArtifactError::new(format!(
                        "`{}` dependency block repeats key `{key}`",
                        path.display()
                    )));
                }
            }
        }
    }
    if in_block {
        dependencies.push(parse_dependency(&values, path)?);
    }
    Ok(dependencies)
}

fn parse_dependency(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerComponentDependency, ArtifactError> {
    Ok(CompilerComponentDependency {
        ordinal: parse_optional_map_usize(values, "ordinal", path, "dependency")?.ok_or_else(
            || {
                ArtifactError::new(format!(
                    "`{}` dependency is missing `ordinal`",
                    path.display()
                ))
            },
        )?,
        kind: parse_required_map_string_in_block(values, "kind", path, "dependency")?,
        identity: parse_required_map_string_in_block(values, "identity", path, "dependency")?,
        content_bytes: parse_optional_map_usize(values, "content_bytes", path, "dependency")?
            .ok_or_else(|| {
                ArtifactError::new(format!(
                    "`{}` dependency is missing `content_bytes`",
                    path.display()
                ))
            })?,
        content_sha256: parse_required_map_string_in_block(
            values,
            "content_sha256",
            path,
            "dependency",
        )?,
    })
}

fn validate_compiler_component_build(build: &CompilerComponentBuild) -> Result<(), ArtifactError> {
    if build.protocol != COMPILER_COMPONENT_BUILD_PROTOCOL
        || build.driver_contract != COMPILER_COMPONENT_DRIVER_CONTRACT
        || !matches!(
            build.stage_role.as_str(),
            COMPILER_COMPONENT_STAGE0_ROLE | COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        )
        || build.dependency_closure_contract != COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT
        || build.reproducible_identity_contract != COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT
    {
        return Err(ArtifactError::new(
            "compiler component build declares an unsupported protocol contract",
        ));
    }
    for (label, value) in [
        (
            "bootstrap subset protocol",
            build.bootstrap_subset_protocol.as_str(),
        ),
        ("component id", build.component_id.as_str()),
        ("component domain", build.component_domain.as_str()),
        ("component unit", build.component_unit.as_str()),
        ("producer id", build.producer_id.as_str()),
    ] {
        validate_header_value(value, label)?;
    }
    for (label, file) in [
        ("stage handoff", build.stage_handoff_file.as_str()),
        ("build manifest", build.build_manifest_file.as_str()),
        ("compiled artifact", build.compiled_artifact_file.as_str()),
        ("native binary", build.native_binary_file.as_str()),
    ] {
        validate_relative_file_name(file, label)?;
    }
    for (label, bytes) in [
        ("compiler image", build.compiler_image_bytes),
        ("build manifest", build.build_manifest_bytes),
        ("compiled artifact", build.compiled_artifact_bytes),
        ("native binary", build.native_binary_bytes),
    ] {
        if bytes == 0 {
            return Err(ArtifactError::new(format!(
                "compiler component build {label} byte length cannot be zero"
            )));
        }
    }
    for (label, value) in [
        ("compiler image", build.compiler_image_sha256.as_str()),
        (
            "stage handoff bundle",
            build.stage_handoff_bundle_sha256.as_str(),
        ),
        ("build manifest", build.build_manifest_sha256.as_str()),
        ("compiled artifact", build.compiled_artifact_sha256.as_str()),
        ("native binary", build.native_binary_sha256.as_str()),
        (
            "dependency closure",
            build.dependency_closure_sha256.as_str(),
        ),
        (
            "reproducible build",
            build.reproducible_build_sha256.as_str(),
        ),
        ("component build record", build.record_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    if build.dependencies.is_empty()
        || build.dependencies.len() > MAX_DEPENDENCIES
        || build.dependency_count != build.dependencies.len()
    {
        return Err(ArtifactError::new(
            "compiler component build dependency count is invalid",
        ));
    }
    let mut previous = None;
    for (ordinal, dependency) in build.dependencies.iter().enumerate() {
        if dependency.ordinal != ordinal || dependency.content_bytes == 0 {
            return Err(ArtifactError::new(format!(
                "compiler component dependency {ordinal} has invalid ordinal or byte length"
            )));
        }
        validate_dependency_kind(&dependency.kind)?;
        validate_dependency_identity(&dependency.identity)?;
        validate_sha256(&dependency.content_sha256, "dependency content")?;
        let key = (dependency.kind.as_str(), dependency.identity.as_str());
        if previous.is_some_and(|previous| previous >= key) {
            return Err(ArtifactError::new(
                "compiler component dependencies must be uniquely sorted by kind and identity",
            ));
        }
        previous = Some(key);
    }
    if dependency_closure_identity(&build.component_id, &build.dependencies)
        != build.dependency_closure_sha256
    {
        return Err(ArtifactError::new(
            "compiler component dependency closure identity mismatch",
        ));
    }
    if reproducible_build_identity(build) != build.reproducible_build_sha256 {
        return Err(ArtifactError::new(
            "compiler component reproducible build identity mismatch",
        ));
    }
    if component_build_identity(build) != build.record_sha256 {
        return Err(ArtifactError::new(
            "compiler component build record identity mismatch",
        ));
    }
    Ok(())
}

fn verify_file(
    path: &Path,
    expected_bytes: usize,
    expected_sha256: &str,
    label: &str,
) -> Result<(), ArtifactError> {
    let bytes = fs::read(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read {label} `{}`: {error}",
            path.display()
        ))
    })?;
    if bytes.len() != expected_bytes || sha256_hex(&bytes) != expected_sha256 {
        return Err(ArtifactError::new(format!(
            "compiler component build {label} length or SHA-256 mismatch"
        )));
    }
    Ok(())
}

fn checked_payload_path(root: &Path, file: &str) -> Result<std::path::PathBuf, ArtifactError> {
    validate_relative_file_name(file, "payload")?;
    let canonical_root = root.canonicalize().map_err(|error| {
        ArtifactError::new(format!("failed to resolve `{}`: {error}", root.display()))
    })?;
    let candidate = root.join(file);
    let canonical = candidate.canonicalize().map_err(|error| {
        ArtifactError::new(format!(
            "failed to resolve `{}`: {error}",
            candidate.display()
        ))
    })?;
    if canonical.parent() != Some(canonical_root.as_path()) {
        return Err(ArtifactError::new(format!(
            "compiler component payload `{file}` escapes its manifest directory"
        )));
    }
    Ok(canonical)
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler component build `{}` must be non-empty UTF-8/LF text without NUL bytes",
            path.display()
        )));
    }
    Ok(())
}

fn validate_header_value(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.is_empty() || value.trim() != value || value.chars().any(|ch| ch.is_control()) {
        return Err(ArtifactError::new(format!(
            "compiler component build {label} must be a non-empty canonical string"
        )));
    }
    Ok(())
}

fn validate_dependency_kind(kind: &str) -> Result<(), ArtifactError> {
    if kind.is_empty()
        || !kind
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ArtifactError::new(format!(
            "compiler dependency kind `{kind}` must use lowercase ASCII tokens"
        )));
    }
    Ok(())
}

fn validate_dependency_identity(identity: &str) -> Result<(), ArtifactError> {
    validate_header_value(identity, "dependency identity")?;
    if identity.starts_with('/')
        || identity.contains('\\')
        || identity
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(ArtifactError::new(format!(
            "compiler dependency identity `{identity}` is not portable"
        )));
    }
    Ok(())
}

fn validate_relative_file_name(file: &str, label: &str) -> Result<(), ArtifactError> {
    let path = Path::new(file);
    if file.is_empty()
        || file.contains('/')
        || file.contains('\\')
        || path.is_absolute()
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(ArtifactError::new(format!(
            "compiler component build {label} file `{file}` must be one relative file name"
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ArtifactError::new(format!(
            "compiler component build {label} must be lowercase SHA-256"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "compiler_component_build_tests.rs"]
mod tests;
