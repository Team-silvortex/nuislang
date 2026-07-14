use std::{fs, path::Path};

use nuis_semantics::model::{AstModule, NirModule};
use yir_core::YirModule;

use crate::aot::CpuBuildTarget;
use crate::aot_c_shim_source::render_c_shim_source;
use crate::aot_manifest_types::CompileArtifacts;
use crate::aot_native_runner::{
    build_window_bundle, compile_native_binary, requires_window_bundle,
};
use crate::aot_output_layout::output_layout;
use crate::render;

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
    fs::write(&shim_path, render_c_shim_source(ast))
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
    if !is_supported_packaging_mode(packaging_mode) {
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

fn is_supported_packaging_mode(packaging_mode: &str) -> bool {
    matches!(
        packaging_mode,
        "window-aot-bundle" | "native-cpu-llvm" | "nuis-self-contained-image"
    )
}
