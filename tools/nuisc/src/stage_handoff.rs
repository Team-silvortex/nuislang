use std::{fs, path::Path};

use nuis_artifact::{
    build_compiler_stage_handoff, parse_compiler_stage_handoff_from_source,
    read_compiler_stage_handoff, render_compiler_stage_handoff, CompilerStageHandoff,
    CompilerStageKind, CompilerStagePayloadInput, VerifiedCompilerStagePayload,
};
use nuis_semantics::model::{AstModule, NirModule};
use yir_core::YirModule;

use crate::{
    aot_manifest_types::CompileStageHandoffArtifacts, aot_output_layout::OutputLayout, frontend,
    render,
};

const STAGE0_PRODUCER_ID: &str = "nuisc-stage0-reference";

pub(crate) fn write_and_verify_compiler_stage_handoff(
    source: &str,
    layout: &OutputLayout,
    ast: &AstModule,
    nir: &NirModule,
    yir: &YirModule,
) -> Result<CompileStageHandoffArtifacts, String> {
    if source.contains('\r') || source.contains('\0') {
        return Err("compiler stage source must use UTF-8/LF text without NUL bytes".to_owned());
    }
    let tokens = frontend::render_stage_neutral_token_stream(&source)?;
    let ast_text = render::render_ast(ast);
    let nir_text = render::render_nir(nir);
    let yir_text = render::render_yir(yir);

    write_text(&layout.source_snapshot_path, source)?;
    write_text(&layout.token_stream_path, &tokens)?;
    write_text(&layout.ast_path, &ast_text)?;
    write_text(&layout.nir_path, &nir_text)?;
    write_text(&layout.yir_path, &yir_text)?;

    let source_file = file_name(&layout.source_snapshot_path)?;
    let tokens_file = file_name(&layout.token_stream_path)?;
    let ast_file = file_name(&layout.ast_path)?;
    let nir_file = file_name(&layout.nir_path)?;
    let yir_file = file_name(&layout.yir_path)?;
    let payloads = [
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Source,
            payload_file: &source_file,
            bytes: source.as_bytes(),
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Tokens,
            payload_file: &tokens_file,
            bytes: tokens.as_bytes(),
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Ast,
            payload_file: &ast_file,
            bytes: ast_text.as_bytes(),
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Nir,
            payload_file: &nir_file,
            bytes: nir_text.as_bytes(),
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Yir,
            payload_file: &yir_file,
            bytes: yir_text.as_bytes(),
        },
    ];
    let handoff =
        build_compiler_stage_handoff(STAGE0_PRODUCER_ID, &ast.domain, &ast.unit, &payloads)
            .map_err(|error| error.to_string())?;
    let rendered = render_compiler_stage_handoff(&handoff);
    let reparsed = parse_compiler_stage_handoff_from_source(&rendered, &layout.stage_handoff_path)
        .map_err(|error| error.to_string())?;
    if reparsed != handoff || render_compiler_stage_handoff(&reparsed) != rendered {
        return Err("compiler stage handoff manifest failed canonical round-trip".to_owned());
    }
    write_text(&layout.stage_handoff_path, &rendered)?;
    verify_compiler_stage_handoff(&layout.stage_handoff_path, Some(ast), Some(nir), Some(yir))?;

    Ok(CompileStageHandoffArtifacts {
        manifest_path: layout.stage_handoff_path.display().to_string(),
        source_path: layout.source_snapshot_path.display().to_string(),
        tokens_path: layout.token_stream_path.display().to_string(),
    })
}

pub(crate) fn verify_compiler_stage_handoff(
    manifest_path: &Path,
    expected_ast: Option<&AstModule>,
    expected_nir: Option<&NirModule>,
    expected_yir: Option<&YirModule>,
) -> Result<CompilerStageHandoff, String> {
    let (handoff, payloads) =
        read_compiler_stage_handoff(manifest_path).map_err(|error| error.to_string())?;
    let source = payload_text(&payloads, CompilerStageKind::Source)?;
    let tokens = payload_text(&payloads, CompilerStageKind::Tokens)?;
    let ast_text = payload_text(&payloads, CompilerStageKind::Ast)?;
    let nir_text = payload_text(&payloads, CompilerStageKind::Nir)?;
    let yir_text = payload_text(&payloads, CompilerStageKind::Yir)?;

    frontend::verify_stage_neutral_token_stream(source, tokens)?;
    let reparsed_ast = frontend::parse_nuis_ast(source)?;
    if render::render_ast(&reparsed_ast) != ast_text {
        return Err("compiler stage AST projection does not match its source payload".to_owned());
    }
    if let Some(expected_ast) = expected_ast {
        if &reparsed_ast != expected_ast {
            return Err("compiler stage AST payload differs from the pipeline AST".to_owned());
        }
    }

    let expected_nir_header = format!(
        "nir mod {} unit {}",
        handoff.module_domain, handoff.module_unit
    );
    if nir_text
        .lines()
        .filter(|line| *line == expected_nir_header)
        .count()
        != 1
    {
        return Err("compiler stage NIR projection has a mismatched module identity".to_owned());
    }
    if let Some(expected_nir) = expected_nir {
        if render::render_nir(expected_nir) != nir_text {
            return Err("compiler stage NIR payload differs from the pipeline NIR".to_owned());
        }
    }

    let reparsed_yir = yir_syntax::parse_explicit_module(yir_text)?;
    yir_verify::verify_module(&reparsed_yir)?;
    if render::render_yir(&reparsed_yir) != yir_text {
        return Err("compiler stage YIR payload is not canonically rendered".to_owned());
    }
    // The portable boundary is the explicit text projection, not producer-private structs.
    if expected_yir.is_some_and(|expected| render::render_yir(expected) != yir_text) {
        return Err("compiler stage YIR payload differs from the pipeline YIR".to_owned());
    }
    Ok(handoff)
}

fn payload_text(
    payloads: &[VerifiedCompilerStagePayload],
    stage: CompilerStageKind,
) -> Result<&str, String> {
    let payload = payloads
        .iter()
        .find(|payload| payload.stage == stage)
        .ok_or_else(|| format!("compiler stage payload `{}` is missing", stage.as_str()))?;
    std::str::from_utf8(&payload.bytes)
        .map_err(|error| format!("compiler stage `{}` is not UTF-8: {error}", stage.as_str()))
}

fn write_text(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

fn file_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "compiler stage payload `{}` has no UTF-8 file name",
                path.display()
            )
        })
}
