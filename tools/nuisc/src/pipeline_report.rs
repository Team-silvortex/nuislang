use super::{CompilePipelineReport, CompilePipelineStage, PipelineArtifacts, ResolvedCompileInput};

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
