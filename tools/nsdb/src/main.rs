use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Status,
    Inspect { input: PathBuf, json: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsdbInspectReport {
    manifest: String,
    debug_model: String,
    native_debugger_visibility: String,
    nsdb_visibility: String,
    debug_readiness: String,
    yir_debuggable: bool,
    domain_count: usize,
    hetero_domain_count: usize,
    clock_edge_count: usize,
    data_segment_count: usize,
    lowering_unit_count: usize,
    sidecar_count: usize,
    domains: Vec<NsdbDomainDebugInfo>,
    clock_edges: Vec<NsdbClockEdgeDebugInfo>,
    data_segments: Vec<NsdbDataSegmentDebugInfo>,
    lowering_units: Vec<NsdbLoweringUnitDebugInfo>,
    sidecars: Vec<NsdbSidecarDebugInfo>,
    missing_metadata: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsdbDomainDebugInfo {
    domain_family: String,
    package_id: String,
    kind: String,
    lowering_target: String,
    backend_family: String,
    debug_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsdbClockEdgeDebugInfo {
    index: usize,
    from: String,
    to: String,
    relation: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsdbDataSegmentDebugInfo {
    index: usize,
    segment_id: String,
    domain_family: String,
    owner_package: String,
    order_key: String,
    access_phase: String,
    source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsdbLoweringUnitDebugInfo {
    index: usize,
    package_id: String,
    domain_family: String,
    backend_family: String,
    selected_lowering_target: String,
    artifact_ir_sidecar_path: String,
    contract_family: String,
    packaging_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsdbSidecarDebugInfo {
    domain_family: String,
    package_id: String,
    path: String,
    schema: String,
    capability_owner: String,
    frontend_ir: String,
    native_ir: String,
    pipeline_lowering: String,
    resource_lowering: String,
    dispatch_lowering: String,
    texture_lowering: String,
    transport_lowering: String,
    tensor_lowering: String,
    memory_lowering: String,
    result_lowering: String,
    validation_contracts: Vec<String>,
    entry_symbol: String,
    stage_kind: String,
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
            println!("Nsdb YIR debugger front-door");
            println!("  tool: nsdb");
            println!("  phase: alpha-0.6.0 debugger metadata boundary");
            println!("  debug_model: yir-metadata");
            println!("  native_debugger_visibility: host-shell-only");
            println!("  nsdb_visibility: yir domains, clock edges, data segments, lowering units");
        }
        Command::Inspect { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan);
            if json {
                println!("{}", nsdb_inspect_report_json(&report));
            } else {
                print_nsdb_inspect_report(&report);
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
        "inspect" => {
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
            Ok(Command::Inspect { input, json })
        }
        "--help" | "-h" | "help" => Err(usage().to_owned()),
        other => Err(format!("unknown nsdb command `{other}`\n{}", usage())),
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
    "usage:\n  nsdb status\n  nsdb inspect <nuis.build.manifest.toml|artifact-output-dir> [--json]"
}

fn nsdb_inspect_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsdbInspectReport {
    let domains = plan
        .domain_units
        .iter()
        .map(|unit| NsdbDomainDebugInfo {
            domain_family: unit.domain_family.clone(),
            package_id: unit.package_id.clone(),
            kind: unit.kind.clone(),
            lowering_target: unit
                .selected_lowering_target
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            backend_family: unit
                .backend_family
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            debug_scope: if unit.kind == "host" {
                "host-shell+yir".to_owned()
            } else {
                "yir-domain".to_owned()
            },
        })
        .collect::<Vec<_>>();
    let clock_edges = plan
        .clock_protocol
        .edges
        .iter()
        .map(|edge| NsdbClockEdgeDebugInfo {
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
        .map(|segment| NsdbDataSegmentDebugInfo {
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
    let lowering_units = plan
        .compiled_artifact
        .lowering_units
        .iter()
        .enumerate()
        .map(|(index, unit)| NsdbLoweringUnitDebugInfo {
            index,
            package_id: unit.package_id.clone(),
            domain_family: unit.domain_family.clone(),
            backend_family: unit
                .backend_family
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            selected_lowering_target: unit
                .selected_lowering_target
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            artifact_ir_sidecar_path: unit
                .artifact_ir_sidecar_path
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            contract_family: unit.contract_family.clone(),
            packaging_role: unit.packaging_role.clone(),
        })
        .collect::<Vec<_>>();
    let sidecars = lowering_units
        .iter()
        .filter(|unit| unit.artifact_ir_sidecar_path != "none")
        .filter_map(|unit| read_sidecar_debug_info(unit))
        .collect::<Vec<_>>();
    let mut missing_metadata = Vec::new();
    if !plan.clock_protocol.validation.valid {
        missing_metadata.push("valid-clock-protocol".to_owned());
    }
    if !plan.hetero_calculate.validation.valid {
        missing_metadata.push("valid-hetero-calculate-plan".to_owned());
    }
    if lowering_units.is_empty() {
        missing_metadata.push("compiled-artifact-lowering-units".to_owned());
    }
    let expected_sidecars = lowering_units
        .iter()
        .filter(|unit| unit.artifact_ir_sidecar_path != "none")
        .count();
    if sidecars.len() != expected_sidecars {
        missing_metadata.push("readable-ir-sidecars".to_owned());
    }

    NsdbInspectReport {
        manifest: manifest.display().to_string(),
        debug_model: "yir-metadata".to_owned(),
        native_debugger_visibility: "host-shell-only".to_owned(),
        nsdb_visibility: "domains+clock+segments+lowering-units".to_owned(),
        debug_readiness: if missing_metadata.is_empty() {
            "yir-debug-ready".to_owned()
        } else {
            "metadata-partial".to_owned()
        },
        yir_debuggable: missing_metadata.is_empty(),
        domain_count: plan.domain_units.len(),
        hetero_domain_count: plan
            .domain_units
            .iter()
            .filter(|unit| unit.kind == "heterogeneous")
            .count(),
        clock_edge_count: clock_edges.len(),
        data_segment_count: data_segments.len(),
        lowering_unit_count: lowering_units.len(),
        sidecar_count: sidecars.len(),
        domains,
        clock_edges,
        data_segments,
        lowering_units,
        sidecars,
        missing_metadata,
    }
}

fn read_sidecar_debug_info(unit: &NsdbLoweringUnitDebugInfo) -> Option<NsdbSidecarDebugInfo> {
    let source = fs::read_to_string(&unit.artifact_ir_sidecar_path).ok()?;
    Some(NsdbSidecarDebugInfo {
        domain_family: unit.domain_family.clone(),
        package_id: unit.package_id.clone(),
        path: unit.artifact_ir_sidecar_path.clone(),
        schema: toml_string_value(&source, "schema").unwrap_or_else(|| "unknown".to_owned()),
        capability_owner: toml_string_value(&source, "capability_owner")
            .unwrap_or_else(|| "unknown".to_owned()),
        frontend_ir: toml_string_value(&source, "frontend_ir")
            .unwrap_or_else(|| "unknown".to_owned()),
        native_ir: toml_string_value(&source, "native_ir").unwrap_or_else(|| "unknown".to_owned()),
        pipeline_lowering: toml_string_value(&source, "pipeline_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        resource_lowering: toml_string_value(&source, "resource_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        dispatch_lowering: toml_string_value(&source, "dispatch_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        texture_lowering: toml_string_value(&source, "texture_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        transport_lowering: toml_string_value(&source, "transport_lowering")
            .or_else(|| toml_string_value(&source, "transport_binding_model"))
            .unwrap_or_else(|| "none".to_owned()),
        tensor_lowering: toml_string_value(&source, "tensor_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        memory_lowering: toml_string_value(&source, "memory_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        result_lowering: toml_string_value(&source, "result_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        validation_contracts: toml_string_array_value(&source, "validation_contracts"),
        entry_symbol: sidecar_entry_symbol(&source),
        stage_kind: toml_string_value(&source, "stage_kind").unwrap_or_else(|| "none".to_owned()),
    })
}

fn sidecar_entry_symbol(source: &str) -> String {
    [
        "entry_symbol",
        "fragment",
        "vertex",
        "compute",
        "graph",
        "grid",
        "range",
        "connect",
    ]
    .into_iter()
    .find_map(|key| toml_string_value(source, key))
    .unwrap_or_else(|| "none".to_owned())
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

fn print_nsdb_inspect_report(report: &NsdbInspectReport) {
    println!("Nsdb YIR debug inspect");
    println!("  manifest: {}", report.manifest);
    println!("  debug_model: {}", report.debug_model);
    println!(
        "  native_debugger_visibility: {}",
        report.native_debugger_visibility
    );
    println!("  nsdb_visibility: {}", report.nsdb_visibility);
    println!("  debug_readiness: {}", report.debug_readiness);
    println!("  yir_debuggable: {}", report.yir_debuggable);
    println!("  domain_count: {}", report.domain_count);
    println!("  hetero_domain_count: {}", report.hetero_domain_count);
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  lowering_unit_count: {}", report.lowering_unit_count);
    println!("  sidecar_count: {}", report.sidecar_count);
    for domain in &report.domains {
        println!(
            "  domain: {} package={} kind={} lowering={} backend={} scope={}",
            domain.domain_family,
            domain.package_id,
            domain.kind,
            domain.lowering_target,
            domain.backend_family,
            domain.debug_scope
        );
    }
    for edge in &report.clock_edges {
        println!(
            "  clock_edge: index={} from={} to={} relation={} source={}",
            edge.index, edge.from, edge.to, edge.relation, edge.source
        );
    }
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
    for unit in &report.lowering_units {
        println!(
            "  lowering_unit: index={} package={} domain={} target={} backend={} sidecar={} role={}",
            unit.index,
            unit.package_id,
            unit.domain_family,
            unit.selected_lowering_target,
            unit.backend_family,
            unit.artifact_ir_sidecar_path,
            unit.packaging_role
        );
    }
    for sidecar in &report.sidecars {
        println!(
            "  sidecar: domain={} package={} schema={} owner={} frontend={} native={} dispatch={} transport={} entry={} stage={}",
            sidecar.domain_family,
            sidecar.package_id,
            sidecar.schema,
            sidecar.capability_owner,
            sidecar.frontend_ir,
            sidecar.native_ir,
            sidecar.dispatch_lowering,
            sidecar.transport_lowering,
            sidecar.entry_symbol,
            sidecar.stage_kind
        );
    }
    for item in &report.missing_metadata {
        println!("  missing_metadata: {item}");
    }
}

fn nsdb_inspect_report_json(report: &NsdbInspectReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_yir_debug_inspect"),
        json_string_field("manifest", &report.manifest),
        json_string_field("debug_model", &report.debug_model),
        json_string_field(
            "native_debugger_visibility",
            &report.native_debugger_visibility,
        ),
        json_string_field("nsdb_visibility", &report.nsdb_visibility),
        json_string_field("debug_readiness", &report.debug_readiness),
        json_bool_field("yir_debuggable", report.yir_debuggable),
        json_usize_field("domain_count", report.domain_count),
        json_usize_field("hetero_domain_count", report.hetero_domain_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_usize_field("lowering_unit_count", report.lowering_unit_count),
        json_usize_field("sidecar_count", report.sidecar_count),
        format!("\"domains\":[{}]", domains_json(&report.domains)),
        format!(
            "\"clock_edges\":[{}]",
            clock_edges_json(&report.clock_edges)
        ),
        format!(
            "\"data_segments\":[{}]",
            data_segments_json(&report.data_segments)
        ),
        format!(
            "\"lowering_units\":[{}]",
            lowering_units_json(&report.lowering_units)
        ),
        format!("\"sidecars\":[{}]", sidecars_json(&report.sidecars)),
        json_string_array_field("missing_metadata", &report.missing_metadata),
    ];
    format!("{{{}}}", fields.join(","))
}

fn domains_json(domains: &[NsdbDomainDebugInfo]) -> String {
    domains
        .iter()
        .map(|domain| {
            let fields = vec![
                json_string_field("domain_family", &domain.domain_family),
                json_string_field("package_id", &domain.package_id),
                json_string_field("kind", &domain.kind),
                json_string_field("lowering_target", &domain.lowering_target),
                json_string_field("backend_family", &domain.backend_family),
                json_string_field("debug_scope", &domain.debug_scope),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn clock_edges_json(edges: &[NsdbClockEdgeDebugInfo]) -> String {
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

fn data_segments_json(segments: &[NsdbDataSegmentDebugInfo]) -> String {
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

fn lowering_units_json(units: &[NsdbLoweringUnitDebugInfo]) -> String {
    units
        .iter()
        .map(|unit| {
            let fields = vec![
                json_usize_field("index", unit.index),
                json_string_field("package_id", &unit.package_id),
                json_string_field("domain_family", &unit.domain_family),
                json_string_field("backend_family", &unit.backend_family),
                json_string_field("selected_lowering_target", &unit.selected_lowering_target),
                json_string_field("artifact_ir_sidecar_path", &unit.artifact_ir_sidecar_path),
                json_string_field("contract_family", &unit.contract_family),
                json_string_field("packaging_role", &unit.packaging_role),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn sidecars_json(sidecars: &[NsdbSidecarDebugInfo]) -> String {
    sidecars
        .iter()
        .map(|sidecar| {
            let fields = vec![
                json_string_field("domain_family", &sidecar.domain_family),
                json_string_field("package_id", &sidecar.package_id),
                json_string_field("path", &sidecar.path),
                json_string_field("schema", &sidecar.schema),
                json_string_field("capability_owner", &sidecar.capability_owner),
                json_string_field("frontend_ir", &sidecar.frontend_ir),
                json_string_field("native_ir", &sidecar.native_ir),
                json_string_field("pipeline_lowering", &sidecar.pipeline_lowering),
                json_string_field("resource_lowering", &sidecar.resource_lowering),
                json_string_field("dispatch_lowering", &sidecar.dispatch_lowering),
                json_string_field("texture_lowering", &sidecar.texture_lowering),
                json_string_field("transport_lowering", &sidecar.transport_lowering),
                json_string_field("tensor_lowering", &sidecar.tensor_lowering),
                json_string_field("memory_lowering", &sidecar.memory_lowering),
                json_string_field("result_lowering", &sidecar.result_lowering),
                json_string_array_field("validation_contracts", &sidecar.validation_contracts),
                json_string_field("entry_symbol", &sidecar.entry_symbol),
                json_string_field("stage_kind", &sidecar.stage_kind),
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

#[cfg(test)]
mod tests {
    use super::{
        parse_args, read_sidecar_debug_info, sidecar_entry_symbol, Command,
        NsdbLoweringUnitDebugInfo,
    };
    use std::{env, fs, path::PathBuf};

    #[test]
    fn parses_status_by_default() {
        assert_eq!(
            parse_args(Vec::<String>::new().into_iter()),
            Ok(Command::Status)
        );
    }

    #[test]
    fn parses_inspect_input_and_json_flag() {
        let command = parse_args(
            vec!["inspect".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Inspect {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn normalizes_non_shader_entry_symbols() {
        let kernel_source = r#"
schema = "nuis-kernel-ir-sidecar-v1"
[entry_points]
graph = "infer_main"
batch = "infer_batch"
"#;
        let network_source = r#"
schema = "nuis-network-ir-sidecar-v1"
[entry_points]
connect = "open_fd_session"
send = "submit_send_recv"
"#;

        assert_eq!(sidecar_entry_symbol(kernel_source), "infer_main");
        assert_eq!(sidecar_entry_symbol(network_source), "open_fd_session");
    }

    #[test]
    fn reads_network_sidecar_capability_metadata() {
        let path =
            env::temp_dir().join(format!("nsdb-network-sidecar-{}.toml", std::process::id()));
        fs::write(
            &path,
            r#"
schema = "nuis-network-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "network-nustar"
frontend_ir = "nuis-yir.network"
native_ir = "posix-socket"
transport_lowering = "packet-poll-reactor"
dispatch_lowering = "poll-send-recv-submit"
validation_contracts = ["glm.fd-handle-lifetime", "time.poll-ready-order"]
[entry_points]
connect = "open_fd_session"
"#,
        )
        .unwrap();
        let unit = NsdbLoweringUnitDebugInfo {
            index: 0,
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            backend_family: "socket-abi".to_owned(),
            selected_lowering_target: "socket-abi.socket-io".to_owned(),
            artifact_ir_sidecar_path: path.display().to_string(),
            contract_family: "nustar.network".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        };

        let sidecar = read_sidecar_debug_info(&unit).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(sidecar.capability_owner, "network-nustar");
        assert_eq!(sidecar.frontend_ir, "nuis-yir.network");
        assert_eq!(sidecar.native_ir, "posix-socket");
        assert_eq!(sidecar.transport_lowering, "packet-poll-reactor");
        assert_eq!(sidecar.dispatch_lowering, "poll-send-recv-submit");
        assert_eq!(sidecar.entry_symbol, "open_fd_session");
        assert_eq!(
            sidecar.validation_contracts,
            vec![
                "glm.fd-handle-lifetime".to_owned(),
                "time.poll-ready-order".to_owned()
            ]
        );
    }
}
