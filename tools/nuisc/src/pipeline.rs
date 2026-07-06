use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use nuis_semantics::model::{AstModule, NirModule};
use yir_core::YirModule;

pub(crate) const NUSTAR_REGISTRY_ROOT: &str = "nustar-packages";

#[path = "pipeline_ffi.rs"]
mod pipeline_ffi;
#[path = "pipeline_report.rs"]
mod pipeline_report;
#[path = "pipeline_units.rs"]
mod pipeline_units;
#[cfg(test)]
#[path = "pipeline_tests.rs"]
mod tests;

use pipeline_ffi::validate_externs;
pub use pipeline_report::compile_pipeline_report;
use pipeline_units::{
    collect_loaded_nustar, validate_instantiated_units, validate_used_units_with_local_units,
};

pub struct PipelineArtifacts {
    pub ast: AstModule,
    pub nir: NirModule,
    pub yir: YirModule,
    pub llvm_ir: String,
    pub loaded_nustar: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilePipelineStage {
    pub id: &'static str,
    pub status: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilePipelineReport {
    pub source_kind: &'static str,
    pub input_path: String,
    pub effective_input_path: String,
    pub project_name: Option<String>,
    pub domain: String,
    pub unit: String,
    pub ast_functions: usize,
    pub nir_functions: usize,
    pub yir_nodes: usize,
    pub yir_resources: usize,
    pub yir_edges: usize,
    pub llvm_ir_bytes: usize,
    pub loaded_nustar: Vec<String>,
    pub stages: Vec<CompilePipelineStage>,
    pub ready_for_aot: bool,
    pub recommended_next_step: &'static str,
    pub recommended_reason: String,
}

impl CompilePipelineReport {
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    pub fn ok_stage_count(&self) -> usize {
        self.stages
            .iter()
            .filter(|stage| stage.status == "ok")
            .count()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "source_kind={} domain={} unit={} stages={}/{} ready_for_aot={} next={}",
            self.source_kind,
            self.domain,
            self.unit,
            self.ok_stage_count(),
            self.stage_count(),
            self.ready_for_aot,
            self.recommended_next_step
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PipelineCompileOptions {
    pub lowering_target: Option<crate::lowering::LoweringTargetConfig>,
}

pub struct ResolvedCompileInput {
    pub input_path: PathBuf,
    pub effective_input_path: PathBuf,
    pub project: Option<crate::project::LoadedProject>,
    pub project_plan: Option<crate::project::ProjectCompilationPlan>,
}

struct PreparedPipeline {
    ast: AstModule,
    nir: NirModule,
    lowering_manifest: crate::registry::NustarPackageManifest,
}

impl ResolvedCompileInput {
    pub fn compile(&self) -> Result<PipelineArtifacts, String> {
        self.compile_with_options(&PipelineCompileOptions::default())
    }

    pub fn compile_with_options(
        &self,
        options: &PipelineCompileOptions,
    ) -> Result<PipelineArtifacts, String> {
        if let (Some(project), Some(plan)) = (&self.project, &self.project_plan) {
            compile_project_plan_with_options(project, plan, options)
        } else {
            compile_source_path_with_options(&self.input_path, options)
        }
    }

    pub fn compile_report(&self, artifacts: &PipelineArtifacts) -> CompilePipelineReport {
        compile_pipeline_report(self, artifacts)
    }
}

pub fn resolve_compile_input(path: &Path) -> Result<ResolvedCompileInput, String> {
    if crate::project::is_project_input(path) {
        let project = crate::project::load_project(path)?;
        let plan = crate::project::build_project_compilation_plan(&project)?;
        return Ok(ResolvedCompileInput {
            input_path: path.to_path_buf(),
            effective_input_path: plan.effective_input_path.clone(),
            project: Some(project),
            project_plan: Some(plan),
        });
    }
    Ok(ResolvedCompileInput {
        input_path: path.to_path_buf(),
        effective_input_path: path.to_path_buf(),
        project: None,
        project_plan: None,
    })
}

pub fn compile_source_path(path: &Path) -> Result<PipelineArtifacts, String> {
    compile_source_path_with_options(path, &PipelineCompileOptions::default())
}

pub fn compile_source_path_with_options(
    path: &Path,
    options: &PipelineCompileOptions,
) -> Result<PipelineArtifacts, String> {
    let resolved = resolve_compile_input(path)?;
    if resolved.project.is_some() {
        return resolved.compile_with_options(options);
    }
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let ast = crate::frontend::parse_nuis_ast(&source)?;
    let helper_modules = stdlib_library_helpers_for_source_path(path)?;
    if helper_modules.is_empty() {
        compile_ast_with_options(ast, options)
    } else {
        compile_ast_with_helper_modules(ast, &helper_modules, options)
    }
}

pub fn compile_project(path: &Path) -> Result<PipelineArtifacts, String> {
    let project = crate::project::load_project(path)?;
    let plan = crate::project::build_project_compilation_plan(&project)?;
    compile_project_plan(&project, &plan)
}

pub fn compile_project_plan(
    project: &crate::project::LoadedProject,
    plan: &crate::project::ProjectCompilationPlan,
) -> Result<PipelineArtifacts, String> {
    compile_project_plan_with_options(project, plan, &PipelineCompileOptions::default())
}

pub fn compile_project_plan_with_options(
    project: &crate::project::LoadedProject,
    plan: &crate::project::ProjectCompilationPlan,
    options: &PipelineCompileOptions,
) -> Result<PipelineArtifacts, String> {
    crate::project::ensure_project_abi_selections_valid(project, &plan.abi_resolution)?;
    crate::registry::ensure_project_domain_registry_valid(plan)?;
    crate::project::ensure_project_lowering_selections_valid(&plan.abi_resolution)?;
    let (ast, nir, local_units) = prepare_project_nir(project)?;
    let lowering_target = options.lowering_target.clone().or_else(|| {
        project_lowering_target_for_domain(&nir.domain, &plan.abi_resolution)
            .ok()
            .and_then(|target| target)
    });
    let prepared = prepare_pipeline(ast, nir, &local_units, |_, nir, _| {
        crate::project::validate_project_links_against_nir(project, nir)
    })?;
    let mut artifacts = lower_prepared_pipeline(prepared, lowering_target)?;
    crate::project::apply_project_support_modules_to_yir(project, &mut artifacts.yir)?;
    crate::project::apply_project_links_to_yir(project, &mut artifacts.yir)?;
    crate::project::validate_project_links_against_yir(project, &artifacts.yir)?;
    crate::project::validate_project_abi_against_yir(project, &artifacts.yir)?;
    crate::project::prune_project_topology_for_codegen(project, &mut artifacts.yir)?;
    artifacts.llvm_ir = yir_lower_llvm::emit_module(&artifacts.yir)?;
    refresh_loaded_nustar(&mut artifacts)?;
    Ok(artifacts)
}

pub fn compile_source(source: &str) -> Result<PipelineArtifacts, String> {
    compile_source_with_options(source, &PipelineCompileOptions::default())
}

pub fn compile_source_with_options(
    source: &str,
    options: &PipelineCompileOptions,
) -> Result<PipelineArtifacts, String> {
    let ast = crate::frontend::parse_nuis_ast(source)?;
    compile_ast_with_options(ast, options)
}

pub fn compile_ast(ast: AstModule) -> Result<PipelineArtifacts, String> {
    compile_ast_with_options(ast, &PipelineCompileOptions::default())
}

pub fn compile_ast_with_options(
    ast: AstModule,
    options: &PipelineCompileOptions,
) -> Result<PipelineArtifacts, String> {
    let nir = crate::frontend::lower_ast_to_nir(&ast)?;
    let prepared = prepare_pipeline(ast, nir, &BTreeSet::new(), |_, _, _| Ok(()))?;
    lower_prepared_pipeline(prepared, options.lowering_target.clone())
}

fn compile_ast_with_helper_modules(
    ast: AstModule,
    helper_modules: &[AstModule],
    options: &PipelineCompileOptions,
) -> Result<PipelineArtifacts, String> {
    let local_units = helper_modules
        .iter()
        .map(|module| (module.domain.clone(), module.unit.clone()))
        .collect::<BTreeSet<_>>();
    let nir = crate::frontend::lower_project_ast_to_nir(&ast, helper_modules)?;
    let prepared = prepare_pipeline(ast, nir, &local_units, |_, _, _| Ok(()))?;
    lower_prepared_pipeline(prepared, options.lowering_target.clone())
}

fn stdlib_library_helpers_for_source_path(path: &Path) -> Result<Vec<AstModule>, String> {
    let stdlib_root = match crate::stdlib_registry::resolve_stdlib_root() {
        Ok(root) => root,
        Err(_) => return Ok(Vec::new()),
    };
    let canonical_root = stdlib_root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize `{}`: {error}",
            stdlib_root.display()
        )
    })?;
    let canonical_path = path
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize `{}`: {error}", path.display()))?;
    let Ok(relative) = canonical_path.strip_prefix(&canonical_root) else {
        return Ok(Vec::new());
    };
    let Some(module_path) = relative.components().next().and_then(|component| {
        let value = component.as_os_str().to_string_lossy();
        (!value.is_empty()).then(|| value.into_owned())
    }) else {
        return Ok(Vec::new());
    };
    if !canonical_root
        .join(&module_path)
        .join("module.toml")
        .is_file()
    {
        return Ok(Vec::new());
    }

    let mut visited = BTreeSet::new();
    let mut helper_paths = Vec::new();
    collect_stdlib_library_module_paths(
        &canonical_root,
        &module_path,
        &canonical_path,
        &mut visited,
        &mut helper_paths,
    )?;
    helper_paths
        .into_iter()
        .map(|helper_path| {
            let source = fs::read_to_string(&helper_path)
                .map_err(|error| format!("failed to read `{}`: {error}", helper_path.display()))?;
            crate::frontend::parse_nuis_ast(&source)
        })
        .collect()
}

fn collect_stdlib_library_module_paths(
    stdlib_root: &Path,
    module_path: &str,
    input_path: &Path,
    visited: &mut BTreeSet<String>,
    helper_paths: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !visited.insert(module_path.to_owned()) {
        return Ok(());
    }
    let manifest = crate::stdlib_registry::load_stdlib_module_manifest(stdlib_root, module_path)?;
    for dependency in &manifest.depends_on {
        collect_stdlib_library_module_paths(
            stdlib_root,
            dependency,
            input_path,
            visited,
            helper_paths,
        )?;
    }
    for library_module in &manifest.library_modules {
        let helper_path = stdlib_root.join(module_path).join(library_module);
        let canonical_helper = helper_path.canonicalize().map_err(|error| {
            format!(
                "failed to canonicalize stdlib library module `{}`: {error}",
                helper_path.display()
            )
        })?;
        if canonical_helper != input_path && !helper_paths.contains(&canonical_helper) {
            helper_paths.push(canonical_helper);
        }
    }
    Ok(())
}

fn prepare_project_nir(
    project: &crate::project::LoadedProject,
) -> Result<(AstModule, NirModule, BTreeSet<(String, String)>), String> {
    let local_units = project
        .modules
        .iter()
        .map(|module| (module.ast.domain.clone(), module.ast.unit.clone()))
        .collect::<BTreeSet<_>>();
    let ast = if let Some(module) = project
        .modules
        .iter()
        .find(|module| module.path == project.entry_path)
    {
        module.ast.clone()
    } else {
        crate::frontend::parse_nuis_ast(&project.entry_source)?
    };
    let helper_modules = project
        .modules
        .iter()
        .filter(|module| module.path != project.entry_path)
        .map(|module| module.ast.clone())
        .collect::<Vec<_>>();
    let nir = crate::frontend::lower_project_ast_to_nir(&ast, &helper_modules)?;
    Ok((ast, nir, local_units))
}

fn prepare_pipeline<F>(
    ast: AstModule,
    mut nir: NirModule,
    local_units: &BTreeSet<(String, String)>,
    validate_nir_hook: F,
) -> Result<PreparedPipeline, String>
where
    F: FnOnce(
        &AstModule,
        &NirModule,
        &crate::registry::NustarPackageManifest,
    ) -> Result<(), String>,
{
    crate::optimize::simplify_nir_module(&mut nir);
    crate::nir_verify::verify_nir_module(&nir)?;
    let lowering_manifest =
        crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), &nir.domain)?;
    validate_externs(&ast, &lowering_manifest)?;
    crate::registry::validate_unit_binding(
        std::slice::from_ref(&lowering_manifest),
        &ast.domain,
        &ast.unit,
    )?;
    validate_used_units_with_local_units(&nir, local_units)?;
    validate_instantiated_units(&nir)?;
    validate_nir_hook(&ast, &nir, &lowering_manifest)?;
    Ok(PreparedPipeline {
        ast,
        nir,
        lowering_manifest,
    })
}

fn lower_prepared_pipeline(
    prepared: PreparedPipeline,
    lowering_target: Option<crate::lowering::LoweringTargetConfig>,
) -> Result<PipelineArtifacts, String> {
    let yir = crate::lowering::lower_nir_to_yir(
        &prepared.nir,
        &prepared.lowering_manifest,
        lowering_target.as_ref(),
    )?;
    let llvm_ir = yir_lower_llvm::emit_module(&yir)?;
    let loaded_nustar =
        collect_loaded_nustar(&prepared.nir, &yir, &prepared.lowering_manifest.package_id)?;
    Ok(PipelineArtifacts {
        ast: prepared.ast,
        nir: prepared.nir,
        yir,
        llvm_ir,
        loaded_nustar,
    })
}

fn project_lowering_target_for_domain(
    domain: &str,
    resolution: &crate::project::ProjectAbiResolution,
) -> Result<Option<crate::lowering::LoweringTargetConfig>, String> {
    let Some(abi) = resolution
        .requirements
        .iter()
        .find(|item| item.domain == domain)
        .map(|item| item.abi.as_str())
    else {
        return Ok(None);
    };
    if domain != "cpu" {
        return Ok(None);
    }
    let target =
        crate::aot::resolve_cpu_build_target_from_abi(Path::new(NUSTAR_REGISTRY_ROOT), abi)?;
    Ok(Some(
        crate::lowering::LoweringTargetConfig::from_cpu_build_target(&target),
    ))
}

fn refresh_loaded_nustar(artifacts: &mut PipelineArtifacts) -> Result<(), String> {
    let lowering_manifest = crate::registry::load_manifest_for_domain(
        Path::new(NUSTAR_REGISTRY_ROOT),
        &artifacts.nir.domain,
    )?;
    artifacts.loaded_nustar = collect_loaded_nustar(
        &artifacts.nir,
        &artifacts.yir,
        &lowering_manifest.package_id,
    )?;
    Ok(())
}
