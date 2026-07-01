use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Status,
    Plan { input: PathBuf, json: bool },
    Check { input: PathBuf, json: bool },
    Closure { input: PathBuf, json: bool },
    Inputs { input: PathBuf, json: bool },
    VerifyInputs { input: PathBuf, json: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldCheckReport {
    manifest: String,
    valid: bool,
    checks: usize,
    failures: usize,
    artifact_lowering_alignment_consistent: bool,
    artifact_lowering_alignment_mismatches: usize,
    clock_protocol_valid: bool,
    clock_protocol_issues: Vec<String>,
    hetero_calculate_valid: bool,
    hetero_calculate_issues: Vec<String>,
    static_link: bool,
    lifecycle_driven: bool,
    sidecar_capability_valid: bool,
    sidecar_capability_issues: Vec<String>,
    final_stage_link_mode: String,
    domains: Vec<NsldDomainDiagnostic>,
    sidecar_capabilities: Vec<NsldSidecarCapabilityDiagnostic>,
    clock_edges: Vec<NsldClockEdgeDiagnostic>,
    data_segments: Vec<NsldDataSegmentDiagnostic>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldDomainDiagnostic {
    domain_family: String,
    package_id: String,
    kind: String,
    packaging_role: String,
    lowering_target: String,
    backend_family: String,
    alignment_consistent: bool,
    alignment_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldSidecarCapabilityDiagnostic {
    domain_family: String,
    package_id: String,
    path: String,
    content_bytes: usize,
    content_hash: String,
    valid: bool,
    capability_owner: String,
    frontend_ir: String,
    native_ir: String,
    dispatch_lowering: String,
    validation_contracts: Vec<String>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldClockEdgeDiagnostic {
    index: usize,
    from: String,
    to: String,
    relation: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldDataSegmentDiagnostic {
    index: usize,
    segment_id: String,
    domain_family: String,
    owner_package: String,
    order_key: String,
    access_phase: String,
    source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldClosureReport {
    manifest: String,
    closed: bool,
    internal_contracts: Vec<String>,
    link_inputs: Vec<NsldLinkInputDiagnostic>,
    link_input_count: usize,
    link_input_total_bytes: usize,
    link_input_table_hash: String,
    external_dependencies: Vec<String>,
    unresolved: Vec<String>,
    host_wrapper_required: bool,
    domain_count: usize,
    hetero_domain_count: usize,
    sidecar_capability_count: usize,
    clock_edge_count: usize,
    data_segment_count: usize,
    final_stage_link_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkInputDiagnostic {
    order_index: usize,
    input_id: String,
    input_kind: String,
    domain_family: String,
    package_id: String,
    path: String,
    native_ir: String,
    dispatch_lowering: String,
    contract_count: usize,
    content_bytes: usize,
    content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkInputsEmitReport {
    manifest: String,
    output_path: String,
    link_input_count: usize,
    link_input_total_bytes: usize,
    link_input_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkInputsVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_link_input_count: usize,
    expected_link_input_total_bytes: usize,
    expected_link_input_table_hash: String,
    actual_link_input_count: Option<usize>,
    actual_link_input_total_bytes: Option<usize>,
    actual_link_input_table_hash: Option<String>,
    issues: Vec<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        Command::Status => {
            println!("Nsld linker front-door");
            println!("  tool: nsld");
            println!("  phase: alpha-0.6.0 linker boundary");
            println!(
                "  current_role: link-plan inspection and hetero clock/link contract surfacing"
            );
            println!("  implementation: reuses nuisc::linker while linker ownership is split out");
            println!("  final_link_status: host-toolchain wrapper is still used for native launcher finalization");
        }
        Command::Plan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            if json {
                println!("{}", nuisc::linker::render_link_plan_json(&plan));
            } else {
                println!("Nsld link plan");
                println!("  input: {}", input.display());
                println!("  manifest: {}", manifest.display());
                println!("  role: alpha-0.6.0 linker front-door");
                for line in nuisc::linker::render_link_plan_summary(&plan) {
                    println!("  {line}");
                }
            }
        }
        Command::Check { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_check_report(&manifest, &plan);
            if json {
                println!("{}", nsld_check_report_json(&report));
            } else {
                print_nsld_check_report(&report);
            }
            if !report.valid {
                return Err("nsld check failed".to_owned());
            }
        }
        Command::Closure { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_closure_report(&manifest, &plan);
            if json {
                println!("{}", nsld_closure_report_json(&report));
            } else {
                print_nsld_closure_report(&report);
            }
        }
        Command::Inputs { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_link_inputs_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_link_inputs_emit_report_json(&report));
            } else {
                print_nsld_link_inputs_emit_report(&report);
            }
        }
        Command::VerifyInputs { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_link_inputs_report(&manifest, &plan);
            if json {
                println!("{}", nsld_link_inputs_verify_report_json(&report));
            } else {
                print_nsld_link_inputs_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld link input verification failed".to_owned());
            }
        }
    }
    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let Some(command) = args.next() else {
        return Ok(Command::Status);
    };
    match command.as_str() {
        "status" => Ok(Command::Status),
        "plan" | "check" | "closure" | "inputs" | "verify-inputs" => {
            let is_check = command == "check";
            let is_closure = command == "closure";
            let is_inputs = command == "inputs";
            let is_verify_inputs = command == "verify-inputs";
            let mut json = false;
            let mut input = None;
            for arg in args {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(format!("unexpected argument `{arg}`"));
                }
            }
            let input = input.ok_or_else(|| usage().to_owned())?;
            if is_check {
                Ok(Command::Check { input, json })
            } else if is_closure {
                Ok(Command::Closure { input, json })
            } else if is_inputs {
                Ok(Command::Inputs { input, json })
            } else if is_verify_inputs {
                Ok(Command::VerifyInputs { input, json })
            } else {
                Ok(Command::Plan { input, json })
            }
        }
        "--help" | "-h" | "help" => Err(usage().to_owned()),
        other => Err(format!("unknown nsld command `{other}`\n{}", usage())),
    }
}

fn resolve_manifest_input(input: &Path) -> Result<PathBuf, String> {
    if input.is_dir() {
        let candidate = input.join("nuis.build.manifest.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        return Err(format!(
            "directory `{}` does not contain `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    Ok(input.to_path_buf())
}

fn usage() -> &'static str {
    "usage:\n  nsld status\n  nsld plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld check <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld closure <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld inputs <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-inputs <nuis.build.manifest.toml|artifact-output-dir> [--json]"
}

fn nsld_check_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsldCheckReport {
    let artifact_lowering_alignment_consistent = plan.artifact_lowering_alignment.consistent;
    let artifact_lowering_alignment_mismatches = plan.artifact_lowering_alignment.mismatches;
    let clock_protocol_valid = plan.clock_protocol.validation.valid;
    let clock_protocol_issues = plan.clock_protocol.validation.issues.clone();
    let hetero_calculate_valid = plan.hetero_calculate.validation.valid;
    let hetero_calculate_issues = plan.hetero_calculate.validation.issues.clone();
    let static_link = plan.hetero_calculate.static_link;
    let lifecycle_driven = plan.hetero_calculate.lifecycle_driven;
    let domains = nsld_domain_diagnostics(plan);
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let sidecar_capability_issues = sidecar_capabilities
        .iter()
        .flat_map(|capability| {
            capability
                .issues
                .iter()
                .map(|issue| {
                    format!(
                        "{}:{}: {}",
                        capability.package_id, capability.domain_family, issue
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let sidecar_capability_valid = sidecar_capability_issues.is_empty();
    let clock_edges = plan
        .clock_protocol
        .edges
        .iter()
        .map(|edge| NsldClockEdgeDiagnostic {
            index: edge.index,
            from: edge.from.clone(),
            to: edge.to.clone(),
            relation: edge.relation.clone(),
            source: edge.source.clone(),
        })
        .collect::<Vec<_>>();
    let data_segments = plan
        .hetero_calculate
        .data_segments
        .iter()
        .map(|segment| NsldDataSegmentDiagnostic {
            index: segment.index,
            segment_id: segment.segment_id.clone(),
            domain_family: segment.domain_family.clone(),
            owner_package: segment.owner_package.clone(),
            order_key: segment.order_key.clone(),
            access_phase: segment.access_phase.clone(),
            source_path: segment
                .source_path
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
        })
        .collect::<Vec<_>>();
    let mut issues = Vec::new();

    if !artifact_lowering_alignment_consistent {
        issues.push(format!(
            "artifact lowering alignment has {} mismatch(es)",
            artifact_lowering_alignment_mismatches
        ));
        for check in &plan.artifact_lowering_alignment.checks {
            for issue in &check.issues {
                issues.push(format!(
                    "{}:{}: {}",
                    check.package_id, check.domain_family, issue
                ));
            }
        }
    }
    if !clock_protocol_valid {
        issues.push("clock protocol validation failed".to_owned());
        issues.extend(clock_protocol_issues.iter().cloned());
    }
    if !hetero_calculate_valid {
        issues.push("hetero calculate validation failed".to_owned());
        issues.extend(hetero_calculate_issues.iter().cloned());
    }
    if !static_link {
        issues.push("hetero calculate plan is not static-link".to_owned());
    }
    if !lifecycle_driven {
        issues.push("hetero calculate plan is not lifecycle-driven".to_owned());
    }
    if !sidecar_capability_valid {
        issues.push("sidecar capability validation failed".to_owned());
        issues.extend(sidecar_capability_issues.iter().cloned());
    }

    let checks = 6;
    let failures = issues.len();
    NsldCheckReport {
        manifest: manifest.display().to_string(),
        valid: failures == 0,
        checks,
        failures,
        artifact_lowering_alignment_consistent,
        artifact_lowering_alignment_mismatches,
        clock_protocol_valid,
        clock_protocol_issues,
        hetero_calculate_valid,
        hetero_calculate_issues,
        static_link,
        lifecycle_driven,
        sidecar_capability_valid,
        sidecar_capability_issues,
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        domains,
        sidecar_capabilities,
        clock_edges,
        data_segments,
        issues,
    }
}

fn nsld_closure_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsldClosureReport {
    let mut internal_contracts = vec![
        "build-manifest".to_owned(),
        "compiled-artifact-envelope".to_owned(),
        "artifact-lowering-alignment".to_owned(),
        "clock-protocol".to_owned(),
        "hetero-calculate-plan".to_owned(),
        "deterministic-data-segment-order".to_owned(),
    ];
    if plan.bridge_registry_path.is_some() {
        internal_contracts.push("bridge-registry".to_owned());
    }
    if plan.host_bridge_plan_index_path.is_some() {
        internal_contracts.push("host-bridge-plan-index".to_owned());
    }
    if plan.lowering_plan_index_path.is_some() {
        internal_contracts.push("lowering-plan-index".to_owned());
    }
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    if !sidecar_capabilities.is_empty()
        && sidecar_capabilities
            .iter()
            .all(|capability| capability.valid)
    {
        internal_contracts.push("lowering-sidecar-capabilities".to_owned());
        internal_contracts.push("link-input-sidecar-table".to_owned());
    }
    let link_inputs = nsld_link_input_diagnostics(&sidecar_capabilities);
    let link_input_count = link_inputs.len();
    let link_input_total_bytes = link_inputs
        .iter()
        .map(|input| input.content_bytes)
        .sum::<usize>();
    let link_input_table_hash = nsld_link_input_table_hash(&link_inputs);

    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let mut external_dependencies = Vec::new();
    if host_wrapper_required {
        external_dependencies.push(format!("final-stage:{}", plan.final_stage.driver));
    }
    if !plan.cpu_target.clang_target.is_empty() {
        external_dependencies.push(format!("clang-target:{}", plan.cpu_target.clang_target));
    }
    if plan.final_stage.link_mode == "bundle-packaging" {
        external_dependencies.push("host-launcher-wrapper".to_owned());
    }

    let mut unresolved = Vec::new();
    if host_wrapper_required {
        unresolved.push("self-owned-final-native-linker".to_owned());
    }
    if plan.compiled_artifact.container_kind.is_none() {
        unresolved.push("nuis-owned-container-kind".to_owned());
    }
    if !plan.artifact_lowering_alignment.consistent {
        unresolved.push("artifact-lowering-alignment-mismatch".to_owned());
    }
    if !plan.clock_protocol.validation.valid {
        unresolved.push("clock-protocol-validation".to_owned());
    }
    if !plan.hetero_calculate.validation.valid {
        unresolved.push("hetero-calculate-validation".to_owned());
    }
    for capability in &sidecar_capabilities {
        for issue in &capability.issues {
            unresolved.push(format!(
                "sidecar-capability:{}:{}:{}",
                capability.package_id, capability.domain_family, issue
            ));
        }
    }

    NsldClosureReport {
        manifest: manifest.display().to_string(),
        closed: unresolved.is_empty(),
        internal_contracts,
        link_inputs,
        link_input_count,
        link_input_total_bytes,
        link_input_table_hash,
        external_dependencies,
        unresolved,
        host_wrapper_required,
        domain_count: plan.domain_units.len(),
        hetero_domain_count: plan
            .domain_units
            .iter()
            .filter(|unit| unit.kind == "heterogeneous")
            .count(),
        sidecar_capability_count: sidecar_capabilities.len(),
        clock_edge_count: plan.clock_protocol.edges.len(),
        data_segment_count: plan.hetero_calculate.data_segments.len(),
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
    }
}

fn nsld_emit_link_inputs_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldLinkInputsEmitReport, String> {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let invalid = sidecar_capabilities
        .iter()
        .filter(|capability| !capability.valid)
        .flat_map(|capability| {
            capability.issues.iter().map(|issue| {
                format!(
                    "{}:{}:{}",
                    capability.package_id, capability.domain_family, issue
                )
            })
        })
        .collect::<Vec<_>>();
    if !invalid.is_empty() {
        return Err(format!(
            "cannot emit nsld link inputs while sidecar capabilities are invalid: {}",
            invalid.join(", ")
        ));
    }
    let link_inputs = nsld_link_input_diagnostics(&sidecar_capabilities);
    let link_input_total_bytes = link_inputs
        .iter()
        .map(|input| input.content_bytes)
        .sum::<usize>();
    let link_input_table_hash = nsld_link_input_table_hash(&link_inputs);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    fs::write(
        &output_path,
        render_nsld_link_input_table_toml(
            &link_inputs,
            link_input_total_bytes,
            &link_input_table_hash,
        ),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld link input table `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldLinkInputsEmitReport {
        manifest: manifest.display().to_string(),
        output_path: output_path.display().to_string(),
        link_input_count: link_inputs.len(),
        link_input_total_bytes,
        link_input_table_hash,
    })
}

fn nsld_verify_link_inputs_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkInputsVerifyReport {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_inputs = nsld_link_input_diagnostics(&sidecar_capabilities);
    let expected_link_input_count = link_inputs.len();
    let expected_link_input_total_bytes = link_inputs
        .iter()
        .map(|input| input.content_bytes)
        .sum::<usize>();
    let expected_link_input_table_hash = nsld_link_input_table_hash(&link_inputs);
    let expected = render_nsld_link_input_table_toml(
        &link_inputs,
        expected_link_input_total_bytes,
        &expected_link_input_table_hash,
    );
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_link_input_table `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_link_input_count, actual_link_input_total_bytes, actual_link_input_table_hash) =
        match actual.as_ref() {
            Ok(source) => (
                toml_usize_value(source, "link_input_count"),
                toml_usize_value(source, "link_input_total_bytes"),
                toml_string_value(source, "link_input_table_hash"),
            ),
            Err(error) => {
                issues.push(error.clone());
                (None, None, None)
            }
        };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("link-input-table-content-mismatch".to_owned());
        }
        if actual_link_input_count != Some(expected_link_input_count) {
            issues.push(format!(
                "link_input_count mismatch: expected {}, found {}",
                expected_link_input_count,
                actual_link_input_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_link_input_total_bytes != Some(expected_link_input_total_bytes) {
            issues.push(format!(
                "link_input_total_bytes mismatch: expected {}, found {}",
                expected_link_input_total_bytes,
                actual_link_input_total_bytes
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_link_input_table_hash.as_deref() != Some(expected_link_input_table_hash.as_str())
        {
            issues.push(format!(
                "link_input_table_hash mismatch: expected {}, found {}",
                expected_link_input_table_hash,
                actual_link_input_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldLinkInputsVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_link_input_count,
        expected_link_input_total_bytes,
        expected_link_input_table_hash,
        actual_link_input_count,
        actual_link_input_total_bytes,
        actual_link_input_table_hash,
        issues,
    }
}

fn print_nsld_closure_report(report: &NsldClosureReport) {
    println!("Nsld linker closure");
    println!("  manifest: {}", report.manifest);
    println!("  closed: {}", report.closed);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  domain_count: {}", report.domain_count);
    println!("  hetero_domain_count: {}", report.hetero_domain_count);
    println!(
        "  sidecar_capability_count: {}",
        report.sidecar_capability_count
    );
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  internal_contracts: {}", report.internal_contracts.len());
    for contract in &report.internal_contracts {
        println!("  internal_contract: {contract}");
    }
    println!("  link_inputs: {}", report.link_inputs.len());
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
    for input in &report.link_inputs {
        println!(
            "  link_input: order={} id={} kind={} domain={} package={} native={} dispatch={} contracts={} bytes={} hash={} path={}",
            input.order_index,
            input.input_id,
            input.input_kind,
            input.domain_family,
            input.package_id,
            input.native_ir,
            input.dispatch_lowering,
            input.contract_count,
            input.content_bytes,
            input.content_hash,
            input.path
        );
    }
    println!(
        "  external_dependencies: {}",
        report.external_dependencies.len()
    );
    for dependency in &report.external_dependencies {
        println!("  external_dependency: {dependency}");
    }
    println!("  unresolved: {}", report.unresolved.len());
    for item in &report.unresolved {
        println!("  unresolved_item: {item}");
    }
}

fn print_nsld_link_inputs_emit_report(report: &NsldLinkInputsEmitReport) {
    println!("Nsld link inputs");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
}

fn nsld_link_inputs_emit_report_json(report: &NsldLinkInputsEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

fn print_nsld_link_inputs_verify_report(report: &NsldLinkInputsVerifyReport) {
    println!("Nsld link inputs verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_link_input_count: {}",
        report.expected_link_input_count
    );
    println!(
        "  expected_link_input_total_bytes: {}",
        report.expected_link_input_total_bytes
    );
    println!(
        "  expected_link_input_table_hash: {}",
        report.expected_link_input_table_hash
    );
    println!(
        "  actual_link_input_count: {}",
        optional_usize_text(report.actual_link_input_count)
    );
    println!(
        "  actual_link_input_total_bytes: {}",
        optional_usize_text(report.actual_link_input_total_bytes)
    );
    println!(
        "  actual_link_input_table_hash: {}",
        report
            .actual_link_input_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn nsld_link_inputs_verify_report_json(report: &NsldLinkInputsVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_usize_field(
            "expected_link_input_total_bytes",
            report.expected_link_input_total_bytes,
        ),
        json_string_field(
            "expected_link_input_table_hash",
            &report.expected_link_input_table_hash,
        ),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_usize_field(
            "actual_link_input_total_bytes",
            report.actual_link_input_total_bytes,
        ),
        json_optional_string_field(
            "actual_link_input_table_hash",
            report.actual_link_input_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_closure_report_json(report: &NsldClosureReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("closed", report.closed),
        json_string_array_field("internal_contracts", &report.internal_contracts),
        format!(
            "\"link_inputs\":[{}]",
            nsld_link_inputs_json(&report.link_inputs)
        ),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_string_array_field("external_dependencies", &report.external_dependencies),
        json_string_array_field("unresolved", &report.unresolved),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("domain_count", report.domain_count),
        json_usize_field("hetero_domain_count", report.hetero_domain_count),
        json_usize_field("sidecar_capability_count", report.sidecar_capability_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_domain_diagnostics(plan: &nuisc::linker::LinkPlan) -> Vec<NsldDomainDiagnostic> {
    plan.domain_units
        .iter()
        .map(|unit| {
            let alignment = plan
                .artifact_lowering_alignment
                .checks
                .iter()
                .find(|check| {
                    check.package_id == unit.package_id && check.domain_family == unit.domain_family
                });
            NsldDomainDiagnostic {
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                kind: unit.kind.clone(),
                packaging_role: unit.packaging_role.clone(),
                lowering_target: unit
                    .selected_lowering_target
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                backend_family: unit
                    .backend_family
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                alignment_consistent: alignment.map(|check| check.consistent).unwrap_or(true),
                alignment_issues: alignment
                    .map(|check| check.issues.clone())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn nsld_sidecar_capability_diagnostics(
    plan: &nuisc::linker::LinkPlan,
) -> Vec<NsldSidecarCapabilityDiagnostic> {
    plan.domain_units
        .iter()
        .filter(|unit| unit.kind == "heterogeneous")
        .filter(|unit| unit.artifact_ir_sidecar_path.is_some())
        .map(|unit| {
            let path = unit
                .artifact_ir_sidecar_path
                .clone()
                .unwrap_or_else(|| "none".to_owned());
            let Some(source) = unit
                .artifact_ir_sidecar_path
                .as_deref()
                .and_then(|path| fs::read_to_string(path).ok())
            else {
                return NsldSidecarCapabilityDiagnostic {
                    domain_family: unit.domain_family.clone(),
                    package_id: unit.package_id.clone(),
                    path,
                    content_bytes: 0,
                    content_hash: "missing".to_owned(),
                    valid: false,
                    capability_owner: "missing".to_owned(),
                    frontend_ir: "missing".to_owned(),
                    native_ir: "missing".to_owned(),
                    dispatch_lowering: "missing".to_owned(),
                    validation_contracts: Vec::new(),
                    issues: vec!["missing_or_unreadable_ir_sidecar".to_owned()],
                };
            };

            let capability_owner =
                toml_string_value(&source, "capability_owner").unwrap_or_else(|| "missing".to_owned());
            let frontend_ir =
                toml_string_value(&source, "frontend_ir").unwrap_or_else(|| "missing".to_owned());
            let native_ir =
                toml_string_value(&source, "native_ir").unwrap_or_else(|| "missing".to_owned());
            let dispatch_lowering =
                toml_string_value(&source, "dispatch_lowering").unwrap_or_else(|| "missing".to_owned());
            let validation_contracts = toml_string_array_value(&source, "validation_contracts");
            let mut issues = Vec::new();
            let expected_owner = format!("{}-nustar", unit.domain_family);
            if capability_owner != expected_owner {
                issues.push(format!(
                    "capability_owner mismatch: expected `{expected_owner}`, found `{capability_owner}`"
                ));
            }
            let expected_frontend = format!("nuis-yir.{}", unit.domain_family);
            if frontend_ir != expected_frontend {
                issues.push(format!(
                    "frontend_ir mismatch: expected `{expected_frontend}`, found `{frontend_ir}`"
                ));
            }
            if native_ir == "missing" || native_ir == "unknown" || native_ir == "unimplemented" {
                issues.push(format!("native_ir is not link-ready: `{native_ir}`"));
            }
            if dispatch_lowering == "missing" || dispatch_lowering == "unimplemented" {
                issues.push(format!(
                    "dispatch_lowering is not link-ready: `{dispatch_lowering}`"
                ));
            }
            if validation_contracts.is_empty() {
                issues.push("validation_contracts is empty".to_owned());
            }

            NsldSidecarCapabilityDiagnostic {
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                path,
                content_bytes: source.len(),
                content_hash: fnv1a64_hex(source.as_bytes()),
                valid: issues.is_empty(),
                capability_owner,
                frontend_ir,
                native_ir,
                dispatch_lowering,
                validation_contracts,
                issues,
            }
        })
        .collect()
}

fn nsld_link_input_diagnostics(
    capabilities: &[NsldSidecarCapabilityDiagnostic],
) -> Vec<NsldLinkInputDiagnostic> {
    let mut capabilities = capabilities
        .iter()
        .filter(|capability| capability.valid)
        .collect::<Vec<_>>();
    capabilities.sort_by(|left, right| {
        left.domain_family
            .cmp(&right.domain_family)
            .then_with(|| left.package_id.cmp(&right.package_id))
            .then_with(|| left.path.cmp(&right.path))
    });
    capabilities
        .into_iter()
        .enumerate()
        .map(|(index, capability)| NsldLinkInputDiagnostic {
            order_index: index,
            input_id: format!(
                "li{:04}.{}.{}",
                index, capability.domain_family, capability.package_id
            ),
            input_kind: "lowering-ir-sidecar".to_owned(),
            domain_family: capability.domain_family.clone(),
            package_id: capability.package_id.clone(),
            path: capability.path.clone(),
            native_ir: capability.native_ir.clone(),
            dispatch_lowering: capability.dispatch_lowering.clone(),
            contract_count: capability.validation_contracts.len(),
            content_bytes: capability.content_bytes,
            content_hash: capability.content_hash.clone(),
        })
        .collect()
}

fn nsld_link_input_table_hash(inputs: &[NsldLinkInputDiagnostic]) -> String {
    let mut material = String::new();
    for input in inputs {
        material.push_str(&input.order_index.to_string());
        material.push('\t');
        material.push_str(&input.input_id);
        material.push('\t');
        material.push_str(&input.input_kind);
        material.push('\t');
        material.push_str(&input.domain_family);
        material.push('\t');
        material.push_str(&input.package_id);
        material.push('\t');
        material.push_str(&input.native_ir);
        material.push('\t');
        material.push_str(&input.dispatch_lowering);
        material.push('\t');
        material.push_str(&input.contract_count.to_string());
        material.push('\t');
        material.push_str(&input.content_bytes.to_string());
        material.push('\t');
        material.push_str(&input.content_hash);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn render_nsld_link_input_table_toml(
    inputs: &[NsldLinkInputDiagnostic],
    total_bytes: usize,
    table_hash: &str,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-link-input-table-v1\"\n");
    out.push_str("table_kind = \"lowering-sidecar-link-inputs\"\n");
    out.push_str(&format!("link_input_count = {}\n", inputs.len()));
    out.push_str(&format!("link_input_total_bytes = {total_bytes}\n"));
    out.push_str(&format!(
        "link_input_table_hash = \"{}\"\n",
        escape_toml_string(table_hash)
    ));
    for input in inputs {
        out.push_str("\n[[link_input]]\n");
        out.push_str(&format!("order_index = {}\n", input.order_index));
        out.push_str(&format!(
            "input_id = \"{}\"\n",
            escape_toml_string(&input.input_id)
        ));
        out.push_str(&format!(
            "input_kind = \"{}\"\n",
            escape_toml_string(&input.input_kind)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&input.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&input.package_id)
        ));
        out.push_str(&format!("path = \"{}\"\n", escape_toml_string(&input.path)));
        out.push_str(&format!(
            "native_ir = \"{}\"\n",
            escape_toml_string(&input.native_ir)
        ));
        out.push_str(&format!(
            "dispatch_lowering = \"{}\"\n",
            escape_toml_string(&input.dispatch_lowering)
        ));
        out.push_str(&format!("contract_count = {}\n", input.contract_count));
        out.push_str(&format!("content_bytes = {}\n", input.content_bytes));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            escape_toml_string(&input.content_hash)
        ));
    }
    out
}

fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("0x{hash:016x}")
}

fn toml_string_value(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        if found_key.trim() != key {
            return None;
        }
        let value = value.trim();
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(|value| {
                value
                    .replace("\\n", "\n")
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\")
            })
    })
}

fn toml_string_array_value(source: &str, key: &str) -> Vec<String> {
    let Some(value) = source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim().to_owned())
    }) else {
        return Vec::new();
    };
    let Some(body) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    body.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            entry
                .strip_prefix('"')
                .and_then(|entry| entry.strip_suffix('"'))
                .map(str::to_owned)
        })
        .collect()
}

fn toml_usize_value(source: &str, key: &str) -> Option<usize> {
    source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key)
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    })
}

fn print_nsld_check_report(report: &NsldCheckReport) {
    println!("Nsld linker check");
    println!("  manifest: {}", report.manifest);
    println!("  valid: {}", report.valid);
    println!("  checks: {}", report.checks);
    println!("  failures: {}", report.failures);
    println!(
        "  artifact_lowering_alignment: consistent={} mismatches={}",
        report.artifact_lowering_alignment_consistent,
        report.artifact_lowering_alignment_mismatches
    );
    println!("  clock_protocol: valid={}", report.clock_protocol_valid);
    println!(
        "  hetero_calculate: valid={}",
        report.hetero_calculate_valid
    );
    println!(
        "  hetero_static_lifecycle: static_link={} lifecycle_driven={}",
        report.static_link, report.lifecycle_driven
    );
    println!(
        "  sidecar_capabilities: valid={} issues={}",
        report.sidecar_capability_valid,
        report.sidecar_capability_issues.len()
    );
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  domains: {}", report.domains.len());
    for domain in &report.domains {
        println!(
            "  domain: {} package={} kind={} lowering={} backend={} alignment_consistent={}",
            domain.domain_family,
            domain.package_id,
            domain.kind,
            domain.lowering_target,
            domain.backend_family,
            domain.alignment_consistent
        );
        for issue in &domain.alignment_issues {
            println!("    domain_issue: {issue}");
        }
    }
    println!(
        "  sidecar_capabilities: {}",
        report.sidecar_capabilities.len()
    );
    for capability in &report.sidecar_capabilities {
        println!(
            "  sidecar_capability: {} package={} owner={} frontend={} native={} dispatch={} valid={} contracts={}",
            capability.domain_family,
            capability.package_id,
            capability.capability_owner,
            capability.frontend_ir,
            capability.native_ir,
            capability.dispatch_lowering,
            capability.valid,
            capability.validation_contracts.len()
        );
        for issue in &capability.issues {
            println!("    sidecar_capability_issue: {issue}");
        }
    }
    println!("  clock_edges: {}", report.clock_edges.len());
    for edge in &report.clock_edges {
        println!(
            "  clock_edge: index={} from={} to={} relation={} source={}",
            edge.index, edge.from, edge.to, edge.relation, edge.source
        );
    }
    println!("  data_segments: {}", report.data_segments.len());
    for segment in &report.data_segments {
        println!(
            "  data_segment: index={} id={} domain={} owner={} order={} phase={} source={}",
            segment.index,
            segment.segment_id,
            segment.domain_family,
            segment.owner_package,
            segment.order_key,
            segment.access_phase,
            segment.source_path
        );
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn nsld_check_report_json(report: &NsldCheckReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_check"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_usize_field("checks", report.checks),
        json_usize_field("failures", report.failures),
        json_bool_field(
            "artifact_lowering_alignment_consistent",
            report.artifact_lowering_alignment_consistent,
        ),
        json_usize_field(
            "artifact_lowering_alignment_mismatches",
            report.artifact_lowering_alignment_mismatches,
        ),
        json_bool_field("clock_protocol_valid", report.clock_protocol_valid),
        json_string_array_field("clock_protocol_issues", &report.clock_protocol_issues),
        json_bool_field("hetero_calculate_valid", report.hetero_calculate_valid),
        json_string_array_field("hetero_calculate_issues", &report.hetero_calculate_issues),
        json_bool_field("static_link", report.static_link),
        json_bool_field("lifecycle_driven", report.lifecycle_driven),
        json_bool_field("sidecar_capability_valid", report.sidecar_capability_valid),
        json_string_array_field(
            "sidecar_capability_issues",
            &report.sidecar_capability_issues,
        ),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        format!("\"domains\":[{}]", nsld_domains_json(&report.domains)),
        format!(
            "\"sidecar_capabilities\":[{}]",
            nsld_sidecar_capabilities_json(&report.sidecar_capabilities)
        ),
        format!(
            "\"clock_edges\":[{}]",
            nsld_clock_edges_json(&report.clock_edges)
        ),
        format!(
            "\"data_segments\":[{}]",
            nsld_data_segments_json(&report.data_segments)
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_domains_json(domains: &[NsldDomainDiagnostic]) -> String {
    domains
        .iter()
        .map(|domain| {
            let fields = vec![
                json_string_field("domain_family", &domain.domain_family),
                json_string_field("package_id", &domain.package_id),
                json_string_field("kind", &domain.kind),
                json_string_field("packaging_role", &domain.packaging_role),
                json_string_field("lowering_target", &domain.lowering_target),
                json_string_field("backend_family", &domain.backend_family),
                json_bool_field("alignment_consistent", domain.alignment_consistent),
                json_string_array_field("alignment_issues", &domain.alignment_issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_sidecar_capabilities_json(capabilities: &[NsldSidecarCapabilityDiagnostic]) -> String {
    capabilities
        .iter()
        .map(|capability| {
            let fields = vec![
                json_string_field("domain_family", &capability.domain_family),
                json_string_field("package_id", &capability.package_id),
                json_string_field("path", &capability.path),
                json_bool_field("valid", capability.valid),
                json_string_field("capability_owner", &capability.capability_owner),
                json_string_field("frontend_ir", &capability.frontend_ir),
                json_string_field("native_ir", &capability.native_ir),
                json_string_field("dispatch_lowering", &capability.dispatch_lowering),
                json_usize_field("content_bytes", capability.content_bytes),
                json_string_field("content_hash", &capability.content_hash),
                json_string_array_field("validation_contracts", &capability.validation_contracts),
                json_string_array_field("issues", &capability.issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_link_inputs_json(inputs: &[NsldLinkInputDiagnostic]) -> String {
    inputs
        .iter()
        .map(|input| {
            let fields = vec![
                json_usize_field("order_index", input.order_index),
                json_string_field("input_id", &input.input_id),
                json_string_field("input_kind", &input.input_kind),
                json_string_field("domain_family", &input.domain_family),
                json_string_field("package_id", &input.package_id),
                json_string_field("path", &input.path),
                json_string_field("native_ir", &input.native_ir),
                json_string_field("dispatch_lowering", &input.dispatch_lowering),
                json_usize_field("contract_count", input.contract_count),
                json_usize_field("content_bytes", input.content_bytes),
                json_string_field("content_hash", &input.content_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_clock_edges_json(edges: &[NsldClockEdgeDiagnostic]) -> String {
    edges
        .iter()
        .map(|edge| {
            let fields = vec![
                json_usize_field("index", edge.index),
                json_string_field("from", &edge.from),
                json_string_field("to", &edge.to),
                json_string_field("relation", &edge.relation),
                json_string_field("source", &edge.source),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_data_segments_json(segments: &[NsldDataSegmentDiagnostic]) -> String {
    segments
        .iter()
        .map(|segment| {
            let fields = vec![
                json_usize_field("index", segment.index),
                json_string_field("segment_id", &segment.segment_id),
                json_string_field("domain_family", &segment.domain_family),
                json_string_field("owner_package", &segment.owner_package),
                json_string_field("order_key", &segment.order_key),
                json_string_field("access_phase", &segment.access_phase),
                json_string_field("source_path", &segment.source_path),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{name}\":{value}")
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", json_escape(value))
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{name}\":{value}")
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => json_string_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{name}\":[{body}]")
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        fnv1a64_hex, nsld_link_input_diagnostics, nsld_link_input_table_hash,
        nsld_sidecar_capability_diagnostics, nsld_verify_link_inputs_report, parse_args,
        render_nsld_link_input_table_toml, Command,
    };
    use nuisc::linker::{
        ArtifactLoweringAlignmentSummary, LinkPlan, LinkPlanArtifact, LinkPlanClockProtocol,
        LinkPlanCpuTarget, LinkPlanEnvelope, LinkPlanFinalStage, LinkPlanHeteroCalculate,
        LinkPlanLifecycle,
    };
    use std::{
        env, fs,
        path::{Path, PathBuf},
    };

    #[test]
    fn parses_status_by_default() {
        assert_eq!(
            parse_args(Vec::<String>::new().into_iter()),
            Ok(Command::Status)
        );
    }

    #[test]
    fn parses_plan_input_and_json_flag() {
        let command =
            parse_args(vec!["plan".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
        assert_eq!(
            command,
            Ok(Command::Plan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_check_input_and_json_flag() {
        let command = parse_args(
            vec![
                "check".to_owned(),
                "nuis.build.manifest.toml".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Check {
                input: PathBuf::from("nuis.build.manifest.toml"),
                json: true
            })
        );
    }

    #[test]
    fn parses_closure_input_and_json_flag() {
        let command = parse_args(
            vec!["closure".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Closure {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_inputs_input_and_json_flag() {
        let command = parse_args(
            vec!["inputs".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Inputs {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_inputs_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-inputs".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifyInputs {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn sidecar_capability_check_skips_hetero_domains_without_ir_sidecars() {
        let path = env::temp_dir().join(format!("nsld-sidecar-cap-{}.toml", std::process::id()));
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.data".to_owned(),
            domain_family: "data".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: None,
            vendor: None,
            device_class: None,
            selected_lowering_target: None,
            contract_family: "nustar.data".to_owned(),
            packaging_role: "domain-sidecar".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });

        let diagnostics = nsld_sidecar_capability_diagnostics(&plan);
        fs::remove_file(path).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].domain_family, "shader");
        assert_eq!(diagnostics[0].content_bytes, sidecar_source.len());
        assert_eq!(
            diagnostics[0].content_hash,
            fnv1a64_hex(sidecar_source.as_bytes())
        );
        assert!(diagnostics[0].valid);
        let link_inputs = nsld_link_input_diagnostics(&diagnostics);
        assert_eq!(link_inputs.len(), 1);
        assert_eq!(link_inputs[0].order_index, 0);
        assert_eq!(link_inputs[0].input_id, "li0000.shader.official.shader");
        assert_eq!(link_inputs[0].input_kind, "lowering-ir-sidecar");
        assert_eq!(link_inputs[0].native_ir, "msl2.4");
        assert_eq!(
            link_inputs[0].dispatch_lowering,
            "command-encoder-draw-dispatch"
        );
        assert_eq!(link_inputs[0].content_bytes, sidecar_source.len());
        assert_eq!(
            link_inputs[0].content_hash,
            fnv1a64_hex(sidecar_source.as_bytes())
        );
        let expected_material = format!(
            "0\tli0000.shader.official.shader\tlowering-ir-sidecar\tshader\tofficial.shader\tmsl2.4\tcommand-encoder-draw-dispatch\t1\t{}\t{}\n",
            sidecar_source.len(),
            fnv1a64_hex(sidecar_source.as_bytes())
        );
        assert_eq!(
            nsld_link_input_table_hash(&link_inputs),
            fnv1a64_hex(expected_material.as_bytes())
        );
        let table = render_nsld_link_input_table_toml(
            &link_inputs,
            link_inputs
                .iter()
                .map(|input| input.content_bytes)
                .sum::<usize>(),
            &nsld_link_input_table_hash(&link_inputs),
        );
        assert!(table.contains("schema = \"nuis-nsld-link-input-table-v1\""));
        assert!(table.contains("link_input_count = 1"));
        assert!(table.contains("input_id = \"li0000.shader.official.shader\""));
        assert!(table.contains("native_ir = \"msl2.4\""));
        assert!(table.contains("content_hash = \""));
    }

    fn empty_link_plan() -> LinkPlan {
        LinkPlan {
            schema: "nuis-link-plan-v1".to_owned(),
            input: "in".to_owned(),
            output_dir: "out".to_owned(),
            packaging_mode: "executable".to_owned(),
            cpu_target: LinkPlanCpuTarget {
                abi: "nuis".to_owned(),
                machine_arch: "arm64".to_owned(),
                machine_os: "macos".to_owned(),
                object_format: "mach-o".to_owned(),
                calling_abi: "aapcs64".to_owned(),
                clang_target: "arm64-apple-macos".to_owned(),
                cross_compile: false,
            },
            lifecycle: LinkPlanLifecycle {
                bootstrap_entry: "main".to_owned(),
                tick_policy: "single".to_owned(),
                shutdown_policy: "return".to_owned(),
                yalivia_rpc: "none".to_owned(),
                hook_surface: Vec::new(),
                export_surface: Vec::new(),
                runtime_capability_flags: Vec::new(),
            },
            envelope: LinkPlanEnvelope {
                schema: "nuis-artifact-envelope-v1".to_owned(),
                package_count: 0,
                contract_families: Vec::new(),
                domain_families: Vec::new(),
                function_kind: "function".to_owned(),
                graph_kind: "static".to_owned(),
                default_time_mode: "logical".to_owned(),
            },
            compiled_artifact: LinkPlanArtifact {
                path: "out/nuis.compiled.artifact".to_owned(),
                binary_name: "demo".to_owned(),
                binary_path: "out/demo".to_owned(),
                binary_bytes: 0,
                build_manifest_bytes: 0,
                container_kind: Some("compiled-artifact-section-table-v2".to_owned()),
                container_version: Some(2),
                section_count: Some(0),
                section_names: Vec::new(),
                section_table_valid: Some(true),
                lowering_unit_count: Some(0),
                lowering_domain_families: Vec::new(),
                lowering_targets: Vec::new(),
                lowering_units: Vec::new(),
            },
            bridge_registry_path: None,
            host_bridge_plan_index_path: None,
            lowering_plan_index_path: None,
            domain_units: Vec::new(),
            artifact_lowering_alignment: ArtifactLoweringAlignmentSummary {
                checked: 0,
                mismatches: 0,
                consistent: true,
                checks: Vec::new(),
            },
            clock_protocol: LinkPlanClockProtocol {
                schema: "nuis-clock-protocol-v1".to_owned(),
                mode: "static".to_owned(),
                source: "test".to_owned(),
                default_time_mode: "logical".to_owned(),
                lifecycle_tick_policy: "single".to_owned(),
                domains: Vec::new(),
                edges: Vec::new(),
                validation: nuisc::linker::LinkPlanClockValidationSummary {
                    checked: 0,
                    valid: true,
                    issues: Vec::new(),
                },
            },
            hetero_calculate: LinkPlanHeteroCalculate {
                schema: "nuis-hetero-calculate-link-v1".to_owned(),
                mode: "static".to_owned(),
                static_link: true,
                lifecycle_driven: true,
                time_order_model: "partial-order".to_owned(),
                data_order_model: "deterministic".to_owned(),
                c_world_policy: "wrapped".to_owned(),
                nodes: Vec::new(),
                data_segments: Vec::new(),
                validation: nuisc::linker::LinkPlanHeteroValidationSummary {
                    checked: 0,
                    valid: true,
                    issues: Vec::new(),
                },
            },
            final_stage: LinkPlanFinalStage {
                kind: "native-executable".to_owned(),
                driver: "clang".to_owned(),
                link_mode: "host-toolchain-finalize".to_owned(),
                output_path: "out/demo".to_owned(),
                inputs: Vec::new(),
                notes: Vec::new(),
            },
        }
    }

    #[test]
    fn verify_link_inputs_accepts_matching_emitted_table() {
        let dir = env::temp_dir().join(format!("nsld-link-input-verify-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        let diagnostics = nsld_sidecar_capability_diagnostics(&plan);
        let inputs = nsld_link_input_diagnostics(&diagnostics);
        let total_bytes = inputs
            .iter()
            .map(|input| input.content_bytes)
            .sum::<usize>();
        let table_hash = nsld_link_input_table_hash(&inputs);
        fs::write(
            dir.join("nuis.nsld.link-inputs.toml"),
            render_nsld_link_input_table_toml(&inputs, total_bytes, &table_hash),
        )
        .unwrap();

        let report = nsld_verify_link_inputs_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_link_input_count, Some(1));
        assert_eq!(
            report.actual_link_input_total_bytes,
            Some(sidecar_source.len())
        );
        assert_eq!(report.actual_link_input_table_hash, Some(table_hash));
    }
}
