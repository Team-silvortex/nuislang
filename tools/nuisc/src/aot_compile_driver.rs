use std::{fs, path::Path};

use nuis_semantics::model::{AstModule, NirModule};
use yir_core::YirModule;

use crate::aot::CpuBuildTarget;
use crate::aot_c_shim_source::render_c_shim_source;
use crate::aot_manifest_types::{
    CompileArtifacts, CompileHostObject, CompileStageHandoffArtifacts,
};
use crate::aot_native_runner::{
    build_application_bundle, compile_native_binary, requires_window_bundle,
};
use crate::aot_output_layout::output_layout;

#[derive(Clone, Copy)]
pub struct AotCompileProgram<'a> {
    pub ast: &'a AstModule,
    pub nir: &'a NirModule,
    pub yir: &'a YirModule,
    pub llvm_ir: Option<&'a str>,
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
    write_and_link_impl(
        input,
        output_dir,
        None,
        AotCompileProgram {
            ast,
            nir,
            yir,
            llvm_ir: Some(llvm_ir),
        },
        cpu_target,
        None,
    )
}

pub fn write_and_link_with_source(
    input: &Path,
    output_dir: &Path,
    source: &str,
    program: AotCompileProgram<'_>,
    cpu_target: &CpuBuildTarget,
) -> Result<CompileArtifacts, String> {
    write_and_link_with_source_and_packaging_mode(
        input, output_dir, source, program, cpu_target, None,
    )
}

pub fn write_and_link_with_source_and_packaging_mode(
    input: &Path,
    output_dir: &Path,
    source: &str,
    program: AotCompileProgram<'_>,
    cpu_target: &CpuBuildTarget,
    packaging_mode: Option<&str>,
) -> Result<CompileArtifacts, String> {
    write_and_link_impl(
        input,
        output_dir,
        Some(source),
        program,
        cpu_target,
        packaging_mode,
    )
}

fn write_and_link_impl(
    input: &Path,
    output_dir: &Path,
    source: Option<&str>,
    program: AotCompileProgram<'_>,
    cpu_target: &CpuBuildTarget,
    requested_packaging_mode: Option<&str>,
) -> Result<CompileArtifacts, String> {
    let AotCompileProgram {
        ast,
        nir,
        yir,
        llvm_ir,
    } = program;
    let packaging_mode = select_packaging_mode(yir, requested_packaging_mode)?;
    if packaging_mode == "headless-aot-bundle" {
        if llvm_ir.is_some() || source.is_none() {
            return Err(
                "headless packaging requires a verified-YIR source checkpoint, not LLVM".to_owned(),
            );
        }
    } else if llvm_ir.is_none_or(|ir| ir.trim().is_empty()) {
        return Err("native/window packaging requires a real LLVM checkpoint".to_owned());
    }
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;

    let layout = output_layout(input, output_dir);
    let ast_path = layout.ast_path.clone();
    let nir_path = layout.nir_path.clone();
    let yir_path = layout.yir_path.clone();
    let ll_path = layout.llvm_ir_path.clone();
    let shim_path = layout.shim_path.clone();
    let llvm_object_path = layout.llvm_object_path.clone();
    let runtime_object_path = layout.runtime_object_path.clone();
    let exe_path = layout.binary_stub_path.clone();

    let stage_handoff = if let Some(source) = source {
        Some(
            crate::stage_handoff::write_and_verify_compiler_stage_handoff(
                source, &layout, ast, nir, yir,
            )?,
        )
    } else {
        fs::write(&ast_path, crate::render::render_ast(ast))
            .map_err(|error| format!("failed to write `{}`: {error}", ast_path.display()))?;
        fs::write(&nir_path, crate::render::render_nir(nir))
            .map_err(|error| format!("failed to write `{}`: {error}", nir_path.display()))?;
        fs::write(&yir_path, crate::render::render_yir(yir))
            .map_err(|error| format!("failed to write `{}`: {error}", yir_path.display()))?;
        None
    };
    if let Some(llvm_ir) = llvm_ir {
        fs::write(&ll_path, llvm_ir)
            .map_err(|error| format!("failed to write `{}`: {error}", ll_path.display()))?;
        fs::write(&shim_path, render_c_shim_source(ast))
            .map_err(|error| format!("failed to write `{}`: {error}", shim_path.display()))?;
    }

    let (binary_path, host_objects) =
        if matches!(packaging_mode, "window-aot-bundle" | "headless-aot-bundle") {
            build_application_bundle(
                &yir_path,
                output_dir,
                cpu_target,
                packaging_mode == "headless-aot-bundle",
            )?;
            (exe_path.display().to_string(), Vec::new())
        } else {
            compile_native_binary(
                &ll_path,
                &shim_path,
                &llvm_object_path,
                &runtime_object_path,
                &exe_path,
                cpu_target,
            )?;
            (
                exe_path.display().to_string(),
                native_host_objects(&llvm_object_path, &runtime_object_path),
            )
        };

    Ok(CompileArtifacts {
        ast_path: ast_path.display().to_string(),
        nir_path: nir_path.display().to_string(),
        yir_path: yir_path.display().to_string(),
        llvm_ir_path: llvm_ir.map(|_| ll_path.display().to_string()),
        binary_path,
        packaging_mode: packaging_mode.to_owned(),
        host_objects,
        stage_handoff,
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
    let host_objects = if packaging_mode == "native-cpu-llvm" {
        native_host_objects(&layout.llvm_object_path, &layout.runtime_object_path)
    } else {
        Vec::new()
    };
    for object in &host_objects {
        if !Path::new(&object.path).is_file() {
            return Err(format!(
                "cached native host object `{}` is missing at `{}`",
                object.object_id, object.path
            ));
        }
    }
    Ok(CompileArtifacts {
        ast_path: layout.ast_path.display().to_string(),
        nir_path: layout.nir_path.display().to_string(),
        yir_path: layout.yir_path.display().to_string(),
        llvm_ir_path: (packaging_mode != "headless-aot-bundle")
            .then(|| layout.llvm_ir_path.display().to_string()),
        binary_path: layout.binary_stub_path.display().to_string(),
        packaging_mode: packaging_mode.to_owned(),
        host_objects,
        stage_handoff: Some(CompileStageHandoffArtifacts {
            manifest_path: layout.stage_handoff_path.display().to_string(),
            source_path: layout.source_snapshot_path.display().to_string(),
            tokens_path: layout.token_stream_path.display().to_string(),
        }),
    })
}

fn native_host_objects(
    llvm_object_path: &Path,
    runtime_object_path: &Path,
) -> Vec<CompileHostObject> {
    vec![
        CompileHostObject {
            object_id: "host.program-llvm".to_owned(),
            role: "program-llvm".to_owned(),
            path: llvm_object_path.display().to_string(),
        },
        CompileHostObject {
            object_id: "host.runtime-shim".to_owned(),
            role: "runtime-shim".to_owned(),
            path: runtime_object_path.display().to_string(),
        },
    ]
}

fn is_supported_packaging_mode(packaging_mode: &str) -> bool {
    matches!(
        packaging_mode,
        "window-aot-bundle"
            | "headless-aot-bundle"
            | "native-cpu-llvm"
            | "nuis-self-contained-image"
    )
}

fn select_packaging_mode(yir: &YirModule, requested: Option<&str>) -> Result<&'static str, String> {
    let window = requires_window_bundle(yir);
    match requested {
        Some("headless-aot-bundle") if !yir.application_sessions.is_empty() => {
            Ok("headless-aot-bundle")
        }
        Some("headless-aot-bundle") => {
            Err("headless AOT packaging requires a registered application session".to_owned())
        }
        Some("native-cpu-llvm") if window => {
            Err("native CPU packaging cannot replace a required window bundle".to_owned())
        }
        Some("window-aot-bundle") if !window => {
            Err("window AOT packaging requires a window entry".to_owned())
        }
        // The self-contained route retains its native bootstrap for Nsld finalization.
        Some("nuis-self-contained-image") if !window => Ok("nuis-self-contained-image"),
        None | Some("native-cpu-llvm" | "window-aot-bundle") => Ok(if window {
            "window-aot-bundle"
        } else {
            "native-cpu-llvm"
        }),
        Some(other) => Err(format!("incompatible AOT packaging mode `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_packaging_is_explicit_and_requires_registration() {
        let mut yir = YirModule::new("0.1");
        assert_eq!(
            select_packaging_mode(&yir, None).unwrap(),
            "native-cpu-llvm"
        );
        assert!(select_packaging_mode(&yir, Some("headless-aot-bundle")).is_err());
        assert!(select_packaging_mode(&yir, Some("window-aot-bundle")).is_err());
        yir.application_sessions.push(
            yir_core::YirApplicationSession::from_fields(&[
                "counter",
                yir_core::APPLICATION_SESSION_CONTRACT,
                "open",
                "event",
                "close",
                "state",
            ])
            .unwrap(),
        );
        assert_eq!(
            select_packaging_mode(&yir, Some("headless-aot-bundle")).unwrap(),
            "headless-aot-bundle"
        );
        assert_eq!(
            select_packaging_mode(&yir, None).unwrap(),
            "native-cpu-llvm"
        );
        assert!(select_packaging_mode(&yir, Some("unknown")).is_err());
        assert!(is_supported_packaging_mode("headless-aot-bundle"));
    }
}
