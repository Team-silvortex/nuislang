use std::path::PathBuf;

use crate::aot;
use crate::command_helpers::load_nuis_executable_envelope;

pub(crate) fn run_pack_envelope(input: PathBuf, output: PathBuf) -> Result<(), String> {
    let envelope = load_nuis_executable_envelope(&input)?;
    let encoded = aot::encode_nuis_executable_envelope_binary(&envelope)?;
    std::fs::write(&output, encoded)
        .map_err(|error| format!("failed to write `{}`: {error}", output.display()))?;
    println!("packed nuis envelope: {}", output.display());
    println!("  source: {}", input.display());
    println!("  schema: {}", envelope.schema);
    println!("  executable_kind: {}", envelope.executable_kind);
    println!("  package_count: {}", envelope.package_count);
    Ok(())
}

pub(crate) fn run_unpack_envelope(input: PathBuf, output: PathBuf) -> Result<(), String> {
    let envelope = load_nuis_executable_envelope(&input)?;
    aot::write_nuis_executable_envelope(&output, &envelope)?;
    println!("unpacked nuis envelope: {}", output.display());
    println!("  source: {}", input.display());
    println!("  schema: {}", envelope.schema);
    println!("  executable_kind: {}", envelope.executable_kind);
    println!("  package_count: {}", envelope.package_count);
    Ok(())
}

pub(crate) fn run_inspect_envelope(input: PathBuf) -> Result<(), String> {
    let envelope = load_nuis_executable_envelope(&input)?;
    println!("nuis envelope: {}", input.display());
    println!("  schema: {}", envelope.schema);
    println!("  executable_kind: {}", envelope.executable_kind);
    println!("  package_count: {}", envelope.package_count);
    println!("  domain_families: {}", envelope.domain_families.join(", "));
    println!(
        "  contract_families: {}",
        envelope.contract_families.join(", ")
    );
    println!("  function_kind: {}", envelope.function_kind);
    println!("  graph_kind: {}", envelope.graph_kind);
    println!("  default_time_mode: {}", envelope.default_time_mode);
    Ok(())
}
