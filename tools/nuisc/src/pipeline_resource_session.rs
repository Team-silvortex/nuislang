use super::*;
use yir_core::native_resource_session::ResourceSessionRegistration;

/// Preserve explicit static resource roots without pretending they satisfy the
/// scalar application_sessions contract. Project/NSB serialization is separate.
pub fn compile_resource_session_source(
    source: &str,
    registration: &ResourceSessionRegistration,
    options: &PipelineCompileOptions,
) -> Result<VerifiedYirArtifacts, String> {
    registration.validate()?;
    let ast = crate::frontend::parse_nuis_ast(source)?;
    let roots = registration.callbacks.iter().cloned().collect();
    let normalized = crate::resource_session_bindings::normalize(&ast, &roots);
    let nir = normalized.lower(&[])?;
    let prepared = prepare_pipeline(
        ast,
        nir,
        &BTreeSet::new(),
        ExternValidationMode::Source,
        |_, _, _| Ok(()),
    )?;
    let artifacts = lower_prepared_pipeline_with_host_entries(
        prepared,
        options.lowering_target.clone(),
        &roots,
    )?;
    registration.bind(artifacts.yir())?;
    Ok(artifacts)
}
