use std::{fs, path::Path};

use nuis_semantics::model::{AstModule, NirExpr, NirModule, NirStmt};
use yir_core::YirModule;

pub struct PipelineArtifacts {
    pub ast: AstModule,
    pub nir: NirModule,
    pub yir: YirModule,
    pub llvm_ir: String,
    pub loaded_nustar: Vec<String>,
}

pub fn compile_source_path(path: &Path) -> Result<PipelineArtifacts, String> {
    if crate::project::is_project_input(path) {
        let project = crate::project::load_project(path)?;
        let ast = crate::frontend::parse_nuis_ast(&project.entry_source)?;
        let nir = crate::frontend::lower_ast_to_nir(&ast)?;
        crate::nir_verify::verify_nir_module(&nir)?;
        let lowering_manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &nir.domain)?;
        validate_externs(&ast, &lowering_manifest)?;
        crate::registry::validate_unit_binding(
            std::slice::from_ref(&lowering_manifest),
            &ast.domain,
            &ast.unit,
        )?;
        validate_used_units(&nir)?;
        validate_instantiated_units(&nir)?;
        crate::project::validate_project_links_against_nir(&project, &nir)?;
        let yir = crate::lowering::lower_nir_to_yir(&nir, &lowering_manifest)?;
        let mut loaded_nustar =
            collect_loaded_nustar(&nir, &yir, &lowering_manifest.package_id)?;
        let mut artifacts = PipelineArtifacts {
            ast,
            nir,
            yir,
            llvm_ir: String::new(),
            loaded_nustar: std::mem::take(&mut loaded_nustar),
        };
        crate::project::apply_project_support_modules_to_yir(&project, &mut artifacts.yir)?;
        crate::project::apply_project_links_to_yir(&project, &mut artifacts.yir)?;
        crate::project::validate_project_links_against_yir(&project, &artifacts.yir)?;
        artifacts.llvm_ir = yir_lower_llvm::emit_module(&artifacts.yir)?;
        let lowering_manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &artifacts.nir.domain)?;
        artifacts.loaded_nustar =
            collect_loaded_nustar(&artifacts.nir, &artifacts.yir, &lowering_manifest.package_id)?;
        return Ok(artifacts);
    }
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    compile_source(&source)
}

pub fn compile_source(source: &str) -> Result<PipelineArtifacts, String> {
    let ast = crate::frontend::parse_nuis_ast(source)?;
    let nir = crate::frontend::lower_ast_to_nir(&ast)?;
    crate::nir_verify::verify_nir_module(&nir)?;
    let lowering_manifest =
        crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &nir.domain)?;
    validate_externs(&ast, &lowering_manifest)?;
    crate::registry::validate_unit_binding(
        std::slice::from_ref(&lowering_manifest),
        &ast.domain,
        &ast.unit,
    )?;
    validate_used_units(&nir)?;
    validate_instantiated_units(&nir)?;
    let yir = crate::lowering::lower_nir_to_yir(&nir, &lowering_manifest)?;
    let llvm_ir = yir_lower_llvm::emit_module(&yir)?;
    let loaded_nustar = collect_loaded_nustar(&nir, &yir, &lowering_manifest.package_id)?;
    Ok(PipelineArtifacts {
        ast,
        nir,
        yir,
        llvm_ir,
        loaded_nustar,
    })
}

fn validate_externs(
    ast: &AstModule,
    lowering_manifest: &crate::registry::NustarPackageManifest,
) -> Result<(), String> {
    if ast.externs.is_empty() && ast.extern_interfaces.is_empty() {
        return Ok(());
    }
    if ast.domain != "cpu" {
        return Err("extern declarations are currently only supported inside `mod cpu <unit>`".to_owned());
    }
    for function in ast
        .externs
        .iter()
        .chain(ast.extern_interfaces.iter().flat_map(|item| item.methods.iter()))
    {
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
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &domain)?;
        crate::registry::validate_unit_binding(&[manifest], &domain, &unit)?;
    }
    Ok(())
}

fn validate_used_units(module: &NirModule) -> Result<(), String> {
    for item in &module.uses {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &item.domain)?;
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
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &item.domain)?;
        loaded.push(manifest.package_id);
    }
    for (domain, _) in collect_instantiated_units(module) {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &domain)?;
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
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. } => {}
        NirExpr::ShaderProfileColorSeed { base, delta, .. } => {
            collect_instantiated_units_expr(base, units);
            collect_instantiated_units_expr(delta, units);
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta,
            scale,
            base,
            ..
        } => {
            collect_instantiated_units_expr(delta, units);
            collect_instantiated_units_expr(scale, units);
            collect_instantiated_units_expr(base, units);
        }
        NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            collect_instantiated_units_expr(base, units);
            collect_instantiated_units_expr(delta, units);
        }
        NirExpr::Borrow(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => collect_instantiated_units_expr(inner, units),
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
