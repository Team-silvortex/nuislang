use std::{fs, path::Path, path::PathBuf};

use nuis_artifact::{
    build_compiler_component_build, build_compiler_diagnostic_report,
    read_compiler_component_build, read_compiler_diagnostic_report, read_compiler_stage_handoff,
    render_compiler_component_build, render_compiler_diagnostic_report,
    verify_compiler_component_build_image, CompilerComponentBuildInput,
    CompilerComponentDependencyInput, CompilerDiagnosticReportInput, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_DIAGNOSTIC_REPORT_FILE,
};
use nuis_semantics::bootstrap_subset::BOOTSTRAP_SUBSET_PROTOCOL;

use crate::command_bootstrap::ensure_bootstrap_resolved;
use crate::command_compile::run_compile_resolved;
use crate::command_helpers::{resolve_compile_input, NUSTAR_REGISTRY_ROOT};
use crate::{aot, registry, registry_load};

const BUILD_MANIFEST_FILE: &str = "nuis.build.manifest.toml";
const COMPILED_ARTIFACT_FILE: &str = "nuis.compiled.artifact";
const STAGE_HANDOFF_FILE: &str = "nuis.compiler-stage-handoff.toml";

struct OwnedDependency {
    kind: &'static str,
    identity: String,
    bytes: Vec<u8>,
}

pub(crate) fn run_bootstrap_build(input: PathBuf, output_dir: PathBuf) -> Result<(), String> {
    let resolved = resolve_compile_input(&input)?;
    let project = resolved.project.as_ref().ok_or_else(|| {
        "bootstrap-build v1 requires a Nuis project so its complete dependency closure can be recorded"
            .to_owned()
    })?;
    ensure_bootstrap_resolved(&resolved)?;

    run_compile_resolved(
        input.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
        &resolved,
    )?;

    let build_manifest_path = output_dir.join(BUILD_MANIFEST_FILE);
    let build_report = aot::verify_build_manifest(&build_manifest_path)?;
    let stage_handoff_path = output_dir.join(STAGE_HANDOFF_FILE);
    let (handoff, _) = read_compiler_stage_handoff(&stage_handoff_path)
        .map_err(|error| format!("failed to consume stage0 handoff: {error}"))?;
    if handoff.producer_id != "nuisc-stage0-reference" {
        return Err(format!(
            "bootstrap-build v1 requires producer `nuisc-stage0-reference`, found `{}`",
            handoff.producer_id
        ));
    }

    let native_binary_file = portable_binary_name(&build_report.artifact_binary_name)?;
    let compiler_image = read_file(
        &std::env::current_exe()
            .map_err(|error| format!("failed to identify stage0 host executable: {error}"))?,
        "stage0 host executable",
    )?;
    let build_manifest = read_file(&build_manifest_path, "build manifest")?;
    let compiled_artifact = read_file(
        &output_dir.join(COMPILED_ARTIFACT_FILE),
        "compiled artifact",
    )?;
    let native_binary = read_file(&output_dir.join(&native_binary_file), "native binary")?;
    let dependencies = collect_dependencies(project, &build_report.loaded_nustar, &output_dir)?;
    let dependency_inputs = dependencies
        .iter()
        .map(|dependency| CompilerComponentDependencyInput {
            kind: dependency.kind,
            identity: &dependency.identity,
            bytes: &dependency.bytes,
        })
        .collect::<Vec<_>>();

    let record = build_compiler_component_build(&CompilerComponentBuildInput {
        stage_role: COMPILER_COMPONENT_STAGE0_ROLE,
        bootstrap_subset_protocol: BOOTSTRAP_SUBSET_PROTOCOL,
        component_id: &project.manifest.name,
        component_domain: &handoff.module_domain,
        component_unit: &handoff.module_unit,
        producer_id: &handoff.producer_id,
        compiler_image: &compiler_image,
        stage_handoff_file: STAGE_HANDOFF_FILE,
        stage_handoff_bundle_sha256: &handoff.bundle_sha256,
        build_manifest_file: BUILD_MANIFEST_FILE,
        build_manifest: &build_manifest,
        compiled_artifact_file: COMPILED_ARTIFACT_FILE,
        compiled_artifact: &compiled_artifact,
        native_binary_file: &native_binary_file,
        native_binary: &native_binary,
        dependencies: &dependency_inputs,
    })
    .map_err(|error| format!("failed to build compiler component record: {error}"))?;
    let record_path = output_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    fs::write(&record_path, render_compiler_component_build(&record)).map_err(|error| {
        format!(
            "failed to write compiler component build `{}`: {error}",
            record_path.display()
        )
    })?;
    let verified = read_compiler_component_build(&record_path)
        .map_err(|error| format!("failed to verify compiler component build: {error}"))?;
    verify_compiler_component_build_image(&verified, &compiler_image)
        .map_err(|error| format!("failed to verify stage0 compiler image: {error}"))?;
    let diagnostic_report = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: &verified.producer_id,
        component_record_sha256: &verified.record_sha256,
        bootstrap_subset_protocol: &verified.bootstrap_subset_protocol,
        accepted: true,
        semantic_pipeline: "checked",
        semantic_error: None,
        diagnostics: &[],
    })
    .map_err(|error| format!("failed to build compiler diagnostic report: {error}"))?;
    let diagnostic_path = output_dir.join(COMPILER_DIAGNOSTIC_REPORT_FILE);
    fs::write(
        &diagnostic_path,
        render_compiler_diagnostic_report(&diagnostic_report),
    )
    .map_err(|error| {
        format!(
            "failed to write compiler diagnostic report `{}`: {error}",
            diagnostic_path.display()
        )
    })?;
    let verified_diagnostics = read_compiler_diagnostic_report(
        &diagnostic_path,
        &verified.record_sha256,
        &verified.producer_id,
    )
    .map_err(|error| format!("failed to verify compiler diagnostic report: {error}"))?;

    println!("bootstrap component build: recorded");
    println!("  protocol: {}", verified.protocol);
    println!("  component: {}", verified.component_id);
    println!("  stage_role: {}", verified.stage_role);
    println!("  producer: {}", verified.producer_id);
    println!("  dependencies: {}", verified.dependency_count);
    println!(
        "  dependency_closure_sha256: {}",
        verified.dependency_closure_sha256
    );
    println!(
        "  stage_handoff_bundle_sha256: {}",
        verified.stage_handoff_bundle_sha256
    );
    println!(
        "  compiler_image_sha256: {}",
        verified.compiler_image_sha256
    );
    println!(
        "  reproducible_build_sha256: {}",
        verified.reproducible_build_sha256
    );
    println!("  record_sha256: {}", verified.record_sha256);
    println!(
        "  diagnostics_sha256: {}",
        verified_diagnostics.diagnostics_sha256
    );
    println!("  record: {}", record_path.display());
    println!("  diagnostics: {}", diagnostic_path.display());
    Ok(())
}

fn collect_dependencies(
    project: &crate::project::LoadedProject,
    loaded_nustar: &[String],
    output_dir: &Path,
) -> Result<Vec<OwnedDependency>, String> {
    let mut dependencies = vec![OwnedDependency {
        kind: "component-manifest",
        identity: "nuis.toml".to_owned(),
        bytes: read_file(&project.manifest_path, "component manifest")?,
    }];
    for module in &project.modules {
        let crate::project::ProjectModuleOrigin::LocalProject { manifest_spec } = &module.origin
        else {
            continue;
        };
        dependencies.push(OwnedDependency {
            kind: "component-source",
            identity: manifest_spec.clone(),
            bytes: read_file(&module.path, "component source")?,
        });
    }
    collect_galaxy_dependencies(project, &mut dependencies)?;

    let galaxy_lock_path = output_dir.join("nuis.project.galaxy.lock");
    if galaxy_lock_path.is_file() {
        dependencies.push(OwnedDependency {
            kind: "galaxy-lock",
            identity: "nuis.project.galaxy.lock".to_owned(),
            bytes: read_file(&galaxy_lock_path, "Galaxy resolution lock")?,
        });
    }
    collect_nustar_dependencies(loaded_nustar, &mut dependencies)?;
    Ok(dependencies)
}

fn collect_galaxy_dependencies(
    project: &crate::project::LoadedProject,
    dependencies: &mut Vec<OwnedDependency>,
) -> Result<(), String> {
    for galaxy in &project.resolved_galaxies {
        push_verified_galaxy_dependency(
            dependencies,
            "galaxy-manifest",
            &galaxy.package_id,
            &galaxy.version,
            &galaxy.manifest_path,
            &galaxy.manifest_content_identity,
        )?;
        for (path, identity) in galaxy
            .resolved_source_paths
            .iter()
            .zip(&galaxy.source_content_identities)
        {
            push_verified_galaxy_dependency(
                dependencies,
                "galaxy-source",
                &galaxy.package_id,
                &galaxy.version,
                path,
                identity,
            )?;
        }
        for (path, identity) in galaxy
            .resolved_library_paths
            .iter()
            .zip(&galaxy.library_content_identities)
        {
            push_verified_galaxy_dependency(
                dependencies,
                "galaxy-library",
                &galaxy.package_id,
                &galaxy.version,
                path,
                identity,
            )?;
        }
    }
    Ok(())
}

fn push_verified_galaxy_dependency(
    dependencies: &mut Vec<OwnedDependency>,
    kind: &'static str,
    package_id: &str,
    version: &str,
    path: &Path,
    expected: &crate::stdlib_registry::ResolvedGalaxyContentIdentity,
) -> Result<(), String> {
    let bytes = read_file(path, kind)?;
    let actual_sha256 = format!("sha256:{}", crate::digest_sha256::sha256_hex(&bytes));
    if bytes.len() != expected.bytes || actual_sha256 != expected.sha256 {
        return Err(format!(
            "bootstrap dependency `{}` drifted after Galaxy resolution",
            expected.logical_path
        ));
    }
    dependencies.push(OwnedDependency {
        kind,
        identity: format!("{package_id}@{version}:{}", expected.logical_path),
        bytes,
    });
    Ok(())
}

fn collect_nustar_dependencies(
    loaded_nustar: &[String],
    dependencies: &mut Vec<OwnedDependency>,
) -> Result<(), String> {
    let requested_root = Path::new(NUSTAR_REGISTRY_ROOT);
    let root = registry_load::resolve_registry_root(requested_root);
    dependencies.push(OwnedDependency {
        kind: "nustar-index",
        identity: "index.toml".to_owned(),
        bytes: read_file(&root.join("index.toml"), "Nustar index")?,
    });
    let index = registry::load_index(requested_root)?;
    let mut package_ids = loaded_nustar.to_vec();
    package_ids.sort();
    package_ids.dedup();
    for package_id in package_ids {
        let entry = index
            .iter()
            .find(|entry| entry.package_id == package_id)
            .ok_or_else(|| format!("loaded Nustar `{package_id}` is missing from the registry"))?;
        dependencies.push(OwnedDependency {
            kind: "nustar-manifest",
            identity: format!("{}:{}", entry.package_id, entry.manifest),
            bytes: read_file(&registry::manifest_path(&root, entry), "Nustar manifest")?,
        });
    }
    Ok(())
}

fn portable_binary_name(value: &str) -> Result<String, String> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('/')
        || value.contains('\\')
        || path.file_name().and_then(|item| item.to_str()) != Some(value)
    {
        return Err(format!(
            "bootstrap build manifest binary name `{value}` is not portable"
        ));
    }
    Ok(value.to_owned())
}

fn read_file(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("failed to read {label} `{}`: {error}", path.display()))
}
