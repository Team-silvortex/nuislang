use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use nuis_semantics::model::{AstModule, NirExpr, NirModule, NirStmt};
use yir_core::YirModule;

const NUSTAR_REGISTRY_ROOT: &str = "nustar-packages";

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

pub fn compile_pipeline_report(
    resolved: &ResolvedCompileInput,
    artifacts: &PipelineArtifacts,
) -> CompilePipelineReport {
    let source_kind = if resolved.project.is_some() {
        "project"
    } else {
        "single_source"
    };
    let project_name = resolved
        .project
        .as_ref()
        .map(|project| project.manifest.name.clone());
    let mut stages = Vec::new();
    stages.push(CompilePipelineStage {
        id: "resolve_input",
        status: "ok",
        detail: format!(
            "{} -> {}",
            resolved.input_path.display(),
            resolved.effective_input_path.display()
        ),
    });
    if let Some(project) = &resolved.project {
        stages.push(CompilePipelineStage {
            id: "compile_plan",
            status: "ok",
            detail: format!(
                "project={} modules={} tests={} galaxy_deps={}",
                project.manifest.name,
                project.modules.len(),
                project.manifest.tests.len(),
                project.manifest.galaxy_dependencies.len()
            ),
        });
    }
    stages.extend([
        CompilePipelineStage {
            id: "ast_parse",
            status: "ok",
            detail: format!(
                "domain={} unit={} functions={} uses={}",
                artifacts.ast.domain,
                artifacts.ast.unit,
                artifacts.ast.functions.len(),
                artifacts.ast.uses.len()
            ),
        },
        CompilePipelineStage {
            id: "nir_lower_verify",
            status: "ok",
            detail: format!(
                "functions={} externs={} structs={} enums={}",
                artifacts.nir.functions.len(),
                artifacts.nir.externs.len() + artifacts.nir.extern_interfaces.len(),
                artifacts.nir.structs.len(),
                artifacts.nir.enums.len()
            ),
        },
        CompilePipelineStage {
            id: "yir_lower",
            status: "ok",
            detail: format!(
                "nodes={} resources={} edges={}",
                artifacts.yir.nodes.len(),
                artifacts.yir.resources.len(),
                artifacts.yir.edges.len()
            ),
        },
        CompilePipelineStage {
            id: "llvm_emit",
            status: "ok",
            detail: format!("bytes={}", artifacts.llvm_ir.len()),
        },
        CompilePipelineStage {
            id: "nustar_closure",
            status: "ok",
            detail: artifacts.loaded_nustar.join(","),
        },
    ]);
    let ready_for_aot = !artifacts.llvm_ir.is_empty()
        && !artifacts.loaded_nustar.is_empty()
        && artifacts
            .yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu");
    let (recommended_next_step, recommended_reason) = if ready_for_aot {
        (
            "build",
            "pipeline reached LLVM and has a non-empty Nustar closure, so the next durable step is AOT packaging/linking",
        )
    } else {
        (
            "inspect_yir",
            "pipeline compiled but does not yet look like a native AOT-ready CPU artifact source",
        )
    };
    CompilePipelineReport {
        source_kind,
        input_path: resolved.input_path.display().to_string(),
        effective_input_path: resolved.effective_input_path.display().to_string(),
        project_name,
        domain: artifacts.nir.domain.clone(),
        unit: artifacts.nir.unit.clone(),
        ast_functions: artifacts.ast.functions.len(),
        nir_functions: artifacts.nir.functions.len(),
        yir_nodes: artifacts.yir.nodes.len(),
        yir_resources: artifacts.yir.resources.len(),
        yir_edges: artifacts.yir.edges.len(),
        llvm_ir_bytes: artifacts.llvm_ir.len(),
        loaded_nustar: artifacts.loaded_nustar.clone(),
        stages,
        ready_for_aot,
        recommended_next_step,
        recommended_reason: recommended_reason.to_owned(),
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
    compile_source_with_options(&source, options)
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

fn validate_externs(
    ast: &AstModule,
    lowering_manifest: &crate::registry::NustarPackageManifest,
) -> Result<(), String> {
    if ast.externs.is_empty() && ast.extern_interfaces.is_empty() {
        return Ok(());
    }
    if ast.domain != "cpu" {
        return Err(
            "extern declarations are currently only supported inside `mod cpu <unit>`".to_owned(),
        );
    }
    for function in ast.externs.iter().chain(
        ast.extern_interfaces
            .iter()
            .flat_map(|item| item.methods.iter()),
    ) {
        if !lowering_manifest
            .host_ffi_abis
            .iter()
            .any(|abi| abi == &function.abi)
        {
            return Err(format!(
                "extern ABI `{}` is not registered by nustar package `{}` for mod domain `{}`",
                function.abi, lowering_manifest.package_id, ast.domain
            ));
        }
    }
    Ok(())
}

fn validate_instantiated_units(module: &NirModule) -> Result<(), String> {
    for (domain, unit) in collect_instantiated_units(module) {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), &domain)?;
        crate::registry::validate_unit_binding(&[manifest], &domain, &unit)?;
    }
    Ok(())
}

fn validate_used_units_with_local_units(
    module: &NirModule,
    local_units: &BTreeSet<(String, String)>,
) -> Result<(), String> {
    for item in &module.uses {
        if local_units.contains(&(item.domain.clone(), item.unit.clone())) {
            continue;
        }
        let manifest = crate::registry::load_manifest_for_domain(
            Path::new(NUSTAR_REGISTRY_ROOT),
            &item.domain,
        )?;
        crate::registry::validate_unit_binding(&[manifest], &item.domain, &item.unit)?;
    }
    Ok(())
}

fn collect_loaded_nustar(
    module: &NirModule,
    yir: &YirModule,
    root_package: &str,
) -> Result<Vec<String>, String> {
    let mut loaded = crate::registry::required_package_ids(yir);
    loaded.push(root_package.to_owned());
    for item in &module.uses {
        let manifest = crate::registry::load_manifest_for_domain(
            Path::new(NUSTAR_REGISTRY_ROOT),
            &item.domain,
        )?;
        loaded.push(manifest.package_id);
    }
    for (domain, _) in collect_instantiated_units(module) {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), &domain)?;
        loaded.push(manifest.package_id);
    }
    loaded.sort();
    loaded.dedup();
    Ok(loaded)
}

fn collect_instantiated_units(module: &NirModule) -> Vec<(String, String)> {
    let mut units = Vec::new();
    for function in &module.functions {
        for stmt in &function.body {
            collect_instantiated_units_stmt(stmt, &mut units);
        }
    }
    units.sort();
    units.dedup();
    units
}

fn collect_instantiated_units_stmt(stmt: &NirStmt, units: &mut Vec<(String, String)>) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => collect_instantiated_units_expr(value, units),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_instantiated_units_expr(condition, units);
            for stmt in then_body {
                collect_instantiated_units_stmt(stmt, units);
            }
            for stmt in else_body {
                collect_instantiated_units_stmt(stmt, units);
            }
        }
        NirStmt::While { condition, body } => {
            collect_instantiated_units_expr(condition, units);
            for stmt in body {
                collect_instantiated_units_stmt(stmt, units);
            }
        }
        NirStmt::Break | NirStmt::Continue => {}
        NirStmt::Return(value) => {
            if let Some(value) = value {
                collect_instantiated_units_expr(value, units);
            }
        }
    }
}

fn collect_instantiated_units_expr(expr: &NirExpr, units: &mut Vec<(String, String)>) {
    match expr {
        NirExpr::Instantiate { domain, unit } => units.push((domain.clone(), unit.clone())),
        NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderProfileTargetRef { .. }
        | NirExpr::ShaderProfileViewportRef { .. }
        | NirExpr::ShaderProfilePipelineRef { .. }
        | NirExpr::ShaderProfileVertexCountRef { .. }
        | NirExpr::ShaderProfileInstanceCountRef { .. }
        | NirExpr::ShaderProfilePacketColorSlotRef { .. }
        | NirExpr::ShaderProfilePacketSpeedSlotRef { .. }
        | NirExpr::ShaderProfilePacketRadiusSlotRef { .. }
        | NirExpr::ShaderProfileSliderColorSlotRef { .. }
        | NirExpr::ShaderProfileSliderSpeedSlotRef { .. }
        | NirExpr::ShaderProfileSliderRadiusSlotRef { .. }
        | NirExpr::ShaderProfileHeaderAccentSlotRef { .. }
        | NirExpr::ShaderProfileToggleLiveSlotRef { .. }
        | NirExpr::ShaderProfileFocusSlotRef { .. }
        | NirExpr::ShaderProfilePacketTagRef { .. }
        | NirExpr::ShaderProfileMaterialModeRef { .. }
        | NirExpr::ShaderProfilePassKindRef { .. }
        | NirExpr::ShaderProfilePacketFieldCountRef { .. }
        | NirExpr::DataProfileBindCoreRef { .. }
        | NirExpr::DataProfileWindowOffsetRef { .. }
        | NirExpr::DataProfileUplinkLenRef { .. }
        | NirExpr::DataProfileDownlinkLenRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::DataProfileMarkerRef { .. }
        | NirExpr::NetworkProfileBindCoreRef { .. }
        | NirExpr::NetworkProfileEndpointKindRef { .. }
        | NirExpr::NetworkProfileTransportFamilyRef { .. }
        | NirExpr::NetworkProfileLocalPortRef { .. }
        | NirExpr::NetworkProfileRemotePortRef { .. }
        | NirExpr::NetworkProfileConnectTimeoutRef { .. }
        | NirExpr::NetworkProfileReadTimeoutRef { .. }
        | NirExpr::NetworkProfileWriteTimeoutRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelTensor { .. }
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. }
        | NirExpr::ShaderInlineWgsl { .. } => {}
        NirExpr::ShaderProfileColorSeed { base, delta, .. } => {
            collect_instantiated_units_expr(base, units);
            collect_instantiated_units_expr(delta, units);
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            collect_instantiated_units_expr(delta, units);
            collect_instantiated_units_expr(scale, units);
            collect_instantiated_units_expr(base, units);
        }
        NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            collect_instantiated_units_expr(base, units);
            collect_instantiated_units_expr(delta, units);
        }
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            ..
        } => {
            collect_instantiated_units_expr(texture, units);
            collect_instantiated_units_expr(sampler, units);
            collect_instantiated_units_expr(x, units);
            collect_instantiated_units_expr(y, units);
        }
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            ..
        } => {
            collect_instantiated_units_expr(texture, units);
            collect_instantiated_units_expr(sampler, units);
            collect_instantiated_units_expr(uv, units);
        }
        NirExpr::ShaderBinding { value, .. } => {
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            collect_instantiated_units_expr(pipeline, units);
            for binding in bindings {
                collect_instantiated_units_expr(binding, units);
            }
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
            ..
        } => {
            collect_instantiated_units_expr(color, units);
            collect_instantiated_units_expr(speed, units);
            collect_instantiated_units_expr(radius, units);
            if let Some(accent) = accent {
                collect_instantiated_units_expr(accent, units);
            }
            if let Some(toggle_state) = toggle_state {
                collect_instantiated_units_expr(toggle_state, units);
            }
            if let Some(focus_index) = focus_index {
                collect_instantiated_units_expr(focus_index, units);
            }
        }
        NirExpr::Borrow(inner)
        | NirExpr::Await(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::Move(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRow(inner)
        | NirExpr::KernelCol(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner)
        | NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::KernelArgmax(inner)
        | NirExpr::KernelArgmin(inner)
        | NirExpr::KernelArgmaxAxis { input: inner, .. }
        | NirExpr::KernelArgminAxis { input: inner, .. }
        | NirExpr::KernelReduceMaxAxis { input: inner, .. }
        | NirExpr::KernelReduceMeanAxis { input: inner, .. }
        | NirExpr::KernelReduceSumAxis { input: inner, .. }
        | NirExpr::KernelSort(inner)
        | NirExpr::KernelSortAxis { input: inner, .. }
        | NirExpr::KernelTopkAxis { input: inner, .. }
        | NirExpr::NetworkResult { value: inner, .. }
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => collect_instantiated_units_expr(inner, units),
        NirExpr::KernelMatmul { lhs, rhs } => {
            collect_instantiated_units_expr(lhs, units);
            collect_instantiated_units_expr(rhs, units);
        }
        NirExpr::KernelElementAt { input, row, col } => {
            collect_instantiated_units_expr(input, units);
            collect_instantiated_units_expr(row, units);
            collect_instantiated_units_expr(col, units);
        }
        NirExpr::KernelReshape { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::KernelBroadcast { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::KernelMap { input, scalar, .. } => {
            collect_instantiated_units_expr(input, units);
            if let Some(scalar) = scalar {
                collect_instantiated_units_expr(scalar, units);
            }
        }
        NirExpr::KernelMapAxis { input, scalar, .. } => {
            collect_instantiated_units_expr(input, units);
            if let Some(scalar) = scalar {
                collect_instantiated_units_expr(scalar, units);
            }
        }
        NirExpr::KernelTopk { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::KernelZip { lhs, rhs, .. } => {
            collect_instantiated_units_expr(lhs, units);
            collect_instantiated_units_expr(rhs, units);
        }
        NirExpr::KernelAddBias { input, bias } => {
            collect_instantiated_units_expr(input, units);
            collect_instantiated_units_expr(bias, units);
        }
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuThreadSpawn { args, .. } => {
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            collect_instantiated_units_expr(task, units);
            collect_instantiated_units_expr(limit, units);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            collect_instantiated_units_expr(target, units);
            collect_instantiated_units_expr(pipeline, units);
            collect_instantiated_units_expr(viewport, units);
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            collect_instantiated_units_expr(packet, units);
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            collect_instantiated_units_expr(pass, units);
            collect_instantiated_units_expr(packet, units);
        }
        NirExpr::CpuExternCall { args, .. } => {
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::AllocNode { value, next } => {
            collect_instantiated_units_expr(value, units);
            collect_instantiated_units_expr(next, units);
        }
        NirExpr::AllocBuffer { len, fill } => {
            collect_instantiated_units_expr(len, units);
            collect_instantiated_units_expr(fill, units);
        }
        NirExpr::LoadAt { buffer, index } => {
            collect_instantiated_units_expr(buffer, units);
            collect_instantiated_units_expr(index, units);
        }
        NirExpr::StoreValue { target, value } => {
            collect_instantiated_units_expr(target, units);
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::StoreNext { target, next } => {
            collect_instantiated_units_expr(target, units);
            collect_instantiated_units_expr(next, units);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            collect_instantiated_units_expr(buffer, units);
            collect_instantiated_units_expr(index, units);
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::DataResult { value: input, .. }
        | NirExpr::ShaderResult { value: input, .. }
        | NirExpr::KernelResult { value: input, .. } => {
            collect_instantiated_units_expr(input, units)
        }
        NirExpr::DataReadWindow { window, index } => {
            collect_instantiated_units_expr(window, units);
            collect_instantiated_units_expr(index, units);
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            collect_instantiated_units_expr(window, units);
            collect_instantiated_units_expr(index, units);
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            collect_instantiated_units_expr(input, units);
            collect_instantiated_units_expr(offset, units);
            collect_instantiated_units_expr(len, units);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            collect_instantiated_units_expr(receiver, units);
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_instantiated_units_expr(value, units);
            }
        }
        NirExpr::FieldAccess { base, .. } => collect_instantiated_units_expr(base, units),
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_instantiated_units_expr(lhs, units);
            collect_instantiated_units_expr(rhs, units);
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Var(_)
        | NirExpr::Null
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{compile_pipeline_report, resolve_compile_input};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nuis_pipeline_{label}_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn compile_pipeline_report_marks_single_source_ready_for_aot() {
        let dir = temp_dir("single_source_report");
        let input = dir.join("main.ns");
        fs::write(
            &input,
            "mod cpu Main {\n  fn main() -> i64 {\n    return 7;\n  }\n}\n",
        )
        .unwrap();

        let resolved = resolve_compile_input(&input).unwrap();
        let artifacts = resolved.compile().unwrap();
        let report = compile_pipeline_report(&resolved, &artifacts);

        assert_eq!(report.source_kind, "single_source");
        assert_eq!(report.domain, "cpu");
        assert_eq!(report.unit, "Main");
        assert!(report.ready_for_aot);
        assert_eq!(report.recommended_next_step, "build");
        assert!(report.stage_count() >= 5);
        assert_eq!(report.stage_count(), report.ok_stage_count());
        assert!(report
            .stages
            .iter()
            .any(|stage| stage.id == "llvm_emit" && stage.status == "ok"));
        assert!(report.loaded_nustar.contains(&"official.cpu".to_owned()));
        assert!(report.summary_line().contains("ready_for_aot=true"));
    }
}
