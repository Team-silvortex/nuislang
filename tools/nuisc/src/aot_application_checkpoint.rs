use super::*;
use nuis_artifact::{
    build_compiler_stage_handoff, parse_compiler_stage_handoff_from_source,
    render_compiler_stage_handoff, CompilerStageKind, CompilerStagePayloadInput,
};

pub(super) fn verify(source: &str, path: &Path, inputs: &[(&str, Vec<u8>)]) -> Result<(), String> {
    let text = |kind| -> Result<&str, String> {
        let bytes = &inputs
            .iter()
            .find(|(key, _)| *key == kind)
            .ok_or_else(|| format!("missing application checkpoint `{kind}`"))?
            .1;
        std::str::from_utf8(bytes).map_err(|error| error.to_string())
    };
    let binary_name = unique_value(source, "artifact_binary_name", path)?;
    crate::aot_artifact::validate_artifact_binary_name("artifact_binary_name", &binary_name, path)?;
    let handoff_source = text("compiler_stage_handoff")?;
    let handoff = parse_compiler_stage_handoff_from_source(handoff_source, path)
        .map_err(|error| error.to_string())?;
    let stages = [
        (CompilerStageKind::Source, "compiler_source"),
        (CompilerStageKind::Tokens, "compiler_tokens"),
        (CompilerStageKind::Ast, "ast"),
        (CompilerStageKind::Nir, "nir"),
        (CompilerStageKind::Yir, "yir"),
    ];
    let names = stages.map(|(_, kind)| input_file_name(kind, &binary_name));
    let payloads = stages
        .iter()
        .zip(&names)
        .map(|((stage, kind), name)| {
            Ok(CompilerStagePayloadInput {
                stage: *stage,
                payload_file: name,
                bytes: text(kind)?.as_bytes(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let rebuilt = build_compiler_stage_handoff(
        &handoff.producer_id,
        &handoff.module_domain,
        &handoff.module_unit,
        &payloads,
    )
    .map_err(|error| error.to_string())?;
    if rebuilt != handoff || render_compiler_stage_handoff(&rebuilt) != handoff_source {
        return Err(
            "application compiler handoff does not match its source-to-YIR payloads".to_owned(),
        );
    }
    crate::frontend::verify_stage_neutral_token_stream(
        text("compiler_source")?,
        text("compiler_tokens")?,
    )?;
    let yir_source = text("yir")?;
    let yir = yir_syntax::parse_explicit_module(yir_source)?;
    yir_verify::verify_module(&yir)?;
    if crate::render::render_yir(&yir) != yir_source {
        return Err("application checkpoint YIR is not canonical".to_owned());
    }
    let mode = unique_value(source, "packaging_mode", path)?;
    if let Some(id) = crate::aot_native_session::registration_id(&mode)? {
        crate::aot_native_session::verify_checkpoint(
            &yir,
            id,
            text("llvm_ir")?,
            text("application_bundle")?,
        )?;
    }
    Ok(())
}
