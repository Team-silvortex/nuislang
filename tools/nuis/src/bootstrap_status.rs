use crate::{json_bool_field, json_field, json_optional_string_field, json_usize_field};
use std::{collections::BTreeSet, fs, path::Path};

pub(crate) const BOOTSTRAP_READINESS_PROTOCOL: &str = "nuis-self-hosting-readiness-v1";

const REQUIRED_GATES: &[(&str, &str)] = &[
    (
        "bootstrap-language-subset",
        "language-core/nuisc/bootstrap-language-subset",
    ),
    (
        "compiler-data-model",
        "standard-library/std/compiler-data-model",
    ),
    (
        "stage-neutral-ir-boundary",
        "language-core/nuisc/stage-neutral-ir-boundary",
    ),
    (
        "stage0-stage1-driver",
        "compiler-toolchain/bootstrap/stage0-stage1-driver",
    ),
    (
        "differential-reproducibility-gate",
        "developer-system/bootstrap/differential-reproducibility-gate",
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapReadinessGate {
    id: String,
    coordinate: String,
    status: String,
    progress: usize,
    required_before: String,
    next_action: String,
    validation_command: String,
    expected_artifact: String,
    blocker: String,
}

impl BootstrapReadinessGate {
    fn is_closed(&self) -> bool {
        self.status == "stable" && self.progress == 100
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapReadinessReport {
    protocol: String,
    release_line: String,
    migration_start: String,
    completion_window_start: String,
    completion_window_end: String,
    gates: Vec<BootstrapReadinessGate>,
}

impl BootstrapReadinessReport {
    fn closed_gate_count(&self) -> usize {
        self.gates.iter().filter(|gate| gate.is_closed()).count()
    }

    fn ready(&self) -> bool {
        self.closed_gate_count() == self.gates.len()
    }

    fn status(&self) -> &'static str {
        if self.ready() {
            "ready-for-stage0-stage1-migration"
        } else {
            "preparing-foundation"
        }
    }

    fn next_gate(&self) -> Option<&BootstrapReadinessGate> {
        self.gates
            .iter()
            .filter(|gate| !gate.is_closed())
            .min_by_key(|gate| {
                (
                    readiness_status_rank(&gate.status),
                    gate.progress,
                    gate.coordinate.as_str(),
                )
            })
    }
}

fn readiness_status_rank(status: &str) -> usize {
    match status {
        "early" => 0,
        "active" => 1,
        "usable" => 2,
        "stable" => 3,
        _ => usize::MAX,
    }
}

#[derive(Default)]
struct HeaderBuilder {
    protocol: Option<String>,
    release_line: Option<String>,
    migration_start: Option<String>,
    completion_window_start: Option<String>,
    completion_window_end: Option<String>,
    gate_count: Option<usize>,
}

#[derive(Default)]
struct GateBuilder {
    id: Option<String>,
    coordinate: Option<String>,
    status: Option<String>,
    progress: Option<usize>,
    required_before: Option<String>,
    next_action: Option<String>,
    validation_command: Option<String>,
    expected_artifact: Option<String>,
    blocker: Option<String>,
}

pub(crate) fn handle_bootstrap_status(input: &Path, json: bool) -> Result<(), String> {
    let source = fs::read_to_string(input).map_err(|error| {
        format!(
            "failed to read self-hosting readiness manifest {}: {error}",
            input.display()
        )
    })?;
    let report = parse_bootstrap_readiness(&source)?;
    if json {
        println!("{}", render_bootstrap_readiness_json(input, &report));
    } else {
        print!("{}", render_bootstrap_readiness_text(input, &report));
    }
    Ok(())
}

pub(crate) fn parse_bootstrap_readiness(source: &str) -> Result<BootstrapReadinessReport, String> {
    let mut header = HeaderBuilder::default();
    let mut gates = Vec::new();
    let mut gate = None::<GateBuilder>;
    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[gates]]" {
            if let Some(builder) = gate.take() {
                gates.push(finish_gate(builder, gates.len() + 1)?);
            }
            gate = Some(GateBuilder::default());
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            format!("self-hosting readiness line {line_number} must be key = value")
        })?;
        let key = key.trim();
        let value = value.trim();
        if let Some(builder) = gate.as_mut() {
            assign_gate_field(builder, key, value, line_number)?;
        } else {
            assign_header_field(&mut header, key, value, line_number)?;
        }
    }
    if let Some(builder) = gate {
        gates.push(finish_gate(builder, gates.len() + 1)?);
    }
    finish_report(header, gates)
}

fn assign_header_field(
    header: &mut HeaderBuilder,
    key: &str,
    value: &str,
    line: usize,
) -> Result<(), String> {
    match key {
        "protocol" => set_string(&mut header.protocol, key, value, line),
        "release_line" => set_string(&mut header.release_line, key, value, line),
        "migration_start" => set_string(&mut header.migration_start, key, value, line),
        "completion_window_start" => {
            set_string(&mut header.completion_window_start, key, value, line)
        }
        "completion_window_end" => set_string(&mut header.completion_window_end, key, value, line),
        "gate_count" => set_usize(&mut header.gate_count, key, value, line),
        _ => Err(format!(
            "unknown self-hosting readiness header field {key} at line {line}"
        )),
    }
}

fn assign_gate_field(
    gate: &mut GateBuilder,
    key: &str,
    value: &str,
    line: usize,
) -> Result<(), String> {
    match key {
        "id" => set_string(&mut gate.id, key, value, line),
        "coordinate" => set_string(&mut gate.coordinate, key, value, line),
        "status" => set_string(&mut gate.status, key, value, line),
        "progress" => set_usize(&mut gate.progress, key, value, line),
        "required_before" => set_string(&mut gate.required_before, key, value, line),
        "next_action" => set_string(&mut gate.next_action, key, value, line),
        "validation_command" => set_string(&mut gate.validation_command, key, value, line),
        "expected_artifact" => set_string(&mut gate.expected_artifact, key, value, line),
        "blocker" => set_string(&mut gate.blocker, key, value, line),
        _ => Err(format!(
            "unknown self-hosting readiness gate field {key} at line {line}"
        )),
    }
}

fn set_string(
    slot: &mut Option<String>,
    key: &str,
    value: &str,
    line: usize,
) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!(
            "duplicate self-hosting readiness field {key} at line {line}"
        ));
    }
    *slot = Some(parse_string(value, key, line)?);
    Ok(())
}

fn set_usize(slot: &mut Option<usize>, key: &str, value: &str, line: usize) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!(
            "duplicate self-hosting readiness field {key} at line {line}"
        ));
    }
    *slot = Some(value.parse::<usize>().map_err(|error| {
        format!("invalid integer for self-hosting readiness field {key} at line {line}: {error}")
    })?);
    Ok(())
}

fn parse_string(value: &str, key: &str, line: usize) -> Result<String, String> {
    let Some(inner) = value
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
    else {
        return Err(format!(
            "self-hosting readiness field {key} at line {line} must be a quoted string"
        ));
    };
    if inner.contains('"') || inner.contains('\\') {
        return Err(format!(
            "self-hosting readiness field {key} at line {line} does not allow escapes or embedded quotes"
        ));
    }
    if inner.trim().is_empty() {
        return Err(format!(
            "self-hosting readiness field {key} at line {line} cannot be empty"
        ));
    }
    Ok(inner.to_owned())
}

fn finish_gate(builder: GateBuilder, index: usize) -> Result<BootstrapReadinessGate, String> {
    Ok(BootstrapReadinessGate {
        id: required(builder.id, "id", index)?,
        coordinate: required(builder.coordinate, "coordinate", index)?,
        status: required(builder.status, "status", index)?,
        progress: builder
            .progress
            .ok_or_else(|| format!("self-hosting readiness gate {index} is missing progress"))?,
        required_before: required(builder.required_before, "required_before", index)?,
        next_action: required(builder.next_action, "next_action", index)?,
        validation_command: required(builder.validation_command, "validation_command", index)?,
        expected_artifact: required(builder.expected_artifact, "expected_artifact", index)?,
        blocker: required(builder.blocker, "blocker", index)?,
    })
}

fn required(value: Option<String>, field: &str, index: usize) -> Result<String, String> {
    value.ok_or_else(|| format!("self-hosting readiness gate {index} is missing {field}"))
}

fn finish_report(
    header: HeaderBuilder,
    gates: Vec<BootstrapReadinessGate>,
) -> Result<BootstrapReadinessReport, String> {
    let protocol = header
        .protocol
        .ok_or_else(|| "self-hosting readiness manifest is missing protocol".to_owned())?;
    if protocol != BOOTSTRAP_READINESS_PROTOCOL {
        return Err(format!(
            "unsupported self-hosting readiness protocol {protocol}; expected {BOOTSTRAP_READINESS_PROTOCOL}"
        ));
    }
    let declared_count = header
        .gate_count
        .ok_or_else(|| "self-hosting readiness manifest is missing gate_count".to_owned())?;
    if declared_count != gates.len() || declared_count != REQUIRED_GATES.len() {
        return Err(format!(
            "self-hosting readiness gate_count mismatch: declared {declared_count}, parsed {}, required {}",
            gates.len(),
            REQUIRED_GATES.len()
        ));
    }
    validate_gates(&gates)?;
    Ok(BootstrapReadinessReport {
        protocol,
        release_line: header
            .release_line
            .ok_or_else(|| "self-hosting readiness manifest is missing release_line".to_owned())?,
        migration_start: header.migration_start.ok_or_else(|| {
            "self-hosting readiness manifest is missing migration_start".to_owned()
        })?,
        completion_window_start: header.completion_window_start.ok_or_else(|| {
            "self-hosting readiness manifest is missing completion_window_start".to_owned()
        })?,
        completion_window_end: header.completion_window_end.ok_or_else(|| {
            "self-hosting readiness manifest is missing completion_window_end".to_owned()
        })?,
        gates,
    })
}

fn validate_gates(gates: &[BootstrapReadinessGate]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for gate in gates {
        let expected_coordinate = REQUIRED_GATES
            .iter()
            .find(|(id, _)| *id == gate.id)
            .map(|(_, coordinate)| *coordinate)
            .ok_or_else(|| format!("unknown self-hosting readiness gate {}", gate.id))?;
        if !seen.insert(gate.id.as_str()) {
            return Err(format!("duplicate self-hosting readiness gate {}", gate.id));
        }
        if gate.coordinate != expected_coordinate {
            return Err(format!(
                "self-hosting readiness gate {} must use coordinate {expected_coordinate}",
                gate.id
            ));
        }
        if !matches!(
            gate.status.as_str(),
            "early" | "active" | "usable" | "stable"
        ) {
            return Err(format!(
                "self-hosting readiness gate {} has unsupported status {}",
                gate.id, gate.status
            ));
        }
        if gate.progress > 100 || (gate.status == "stable") != (gate.progress == 100) {
            return Err(format!(
                "self-hosting readiness gate {} must use stable/100 only for a closed gate",
                gate.id
            ));
        }
    }
    if let Some((missing, _)) = REQUIRED_GATES.iter().find(|(id, _)| !seen.contains(id)) {
        return Err(format!("missing self-hosting readiness gate {missing}"));
    }
    Ok(())
}

pub(crate) fn render_bootstrap_readiness_json(
    input: &Path,
    report: &BootstrapReadinessReport,
) -> String {
    let gates = report
        .gates
        .iter()
        .map(|gate| format!("{{{}}}", render_gate_json(gate)))
        .collect::<Vec<_>>()
        .join(",");
    let mut fields = vec![
        json_field("protocol", &report.protocol),
        json_field("manifest", &input.display().to_string()),
        json_field("release_line", &report.release_line),
        json_field("migration_start", &report.migration_start),
        json_field("completion_window_start", &report.completion_window_start),
        json_field("completion_window_end", &report.completion_window_end),
        json_field("status", report.status()),
        json_bool_field("ready", report.ready()),
        json_usize_field("gate_count", report.gates.len()),
        json_usize_field("closed_gate_count", report.closed_gate_count()),
        json_optional_string_field("next_gate", report.next_gate().map(|gate| gate.id.as_str())),
        json_optional_string_field(
            "next_coordinate",
            report.next_gate().map(|gate| gate.coordinate.as_str()),
        ),
        json_optional_string_field(
            "next_action",
            report.next_gate().map(|gate| gate.next_action.as_str()),
        ),
        json_optional_string_field(
            "next_validation_command",
            report
                .next_gate()
                .map(|gate| gate.validation_command.as_str()),
        ),
        json_optional_string_field(
            "next_expected_artifact",
            report
                .next_gate()
                .map(|gate| gate.expected_artifact.as_str()),
        ),
        json_optional_string_field(
            "first_blocker",
            report.next_gate().map(|gate| gate.blocker.as_str()),
        ),
    ];
    fields.push(format!("\"gates\":[{gates}]"));
    format!("{{{}}}", fields.join(","))
}

fn render_gate_json(gate: &BootstrapReadinessGate) -> String {
    [
        json_field("id", &gate.id),
        json_field("coordinate", &gate.coordinate),
        json_field("status", &gate.status),
        json_usize_field("progress", gate.progress),
        json_bool_field("closed", gate.is_closed()),
        json_field("required_before", &gate.required_before),
        json_field("next_action", &gate.next_action),
        json_field("validation_command", &gate.validation_command),
        json_field("expected_artifact", &gate.expected_artifact),
        json_field("blocker", &gate.blocker),
    ]
    .join(",")
}

pub(crate) fn render_bootstrap_readiness_text(
    input: &Path,
    report: &BootstrapReadinessReport,
) -> String {
    let mut out = format!(
        "nuis self-hosting readiness\n  protocol: {}\n  manifest: {}\n  release_line: {}\n  migration_start: {}\n  completion_window_start: {}\n  completion_window_end: {}\n  status: {}\n  ready: {}\n  gates: {}/{}\n",
        report.protocol,
        input.display(),
        report.release_line,
        report.migration_start,
        report.completion_window_start,
        report.completion_window_end,
        report.status(),
        report.ready(),
        report.closed_gate_count(),
        report.gates.len()
    );
    if let Some(gate) = report.next_gate() {
        out.push_str(&format!(
            "  next_gate: {}\n  next_coordinate: {}\n  first_blocker: {}\n  next_action: {}\n  next_validation_command: {}\n  next_expected_artifact: {}\n",
            gate.id,
            gate.coordinate,
            gate.blocker,
            gate.next_action,
            gate.validation_command,
            gate.expected_artifact
        ));
    } else {
        out.push_str(
            "  next_gate: <none>\n  next_coordinate: <none>\n  first_blocker: <none>\n  next_action: <none>\n  next_validation_command: <none>\n  next_expected_artifact: <none>\n",
        );
    }
    for gate in &report.gates {
        out.push_str(&format!(
            "  gate: {} {}/{} {}\n",
            gate.id, gate.status, gate.progress, gate.coordinate
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECKED_IN_MANIFEST: &str =
        include_str!("../../../docs/reference/nuis-self-hosting-readiness.toml");

    #[test]
    fn checked_in_manifest_is_valid_and_names_the_weakest_real_gate() {
        let report = parse_bootstrap_readiness(CHECKED_IN_MANIFEST).expect("manifest parses");
        assert_eq!(report.gates.len(), REQUIRED_GATES.len());
        assert_eq!(report.closed_gate_count(), 1);
        assert!(!report.ready());
        assert_eq!(report.status(), "preparing-foundation");
        assert_eq!(report.completion_window_start, "gamma-0.5.*");
        assert_eq!(report.completion_window_end, "gamma-0.10.*");
        assert_eq!(
            report.next_gate().map(|gate| gate.id.as_str()),
            Some("stage-neutral-ir-boundary")
        );
    }

    #[test]
    fn renderers_expose_readiness_without_claiming_migration_ready() {
        let report = parse_bootstrap_readiness(CHECKED_IN_MANIFEST).expect("manifest parses");
        let path = Path::new("docs/reference/nuis-self-hosting-readiness.toml");
        let json = render_bootstrap_readiness_json(path, &report);
        let text = render_bootstrap_readiness_text(path, &report);
        assert!(json.contains("\"protocol\":\"nuis-self-hosting-readiness-v1\""));
        assert!(json.contains("\"ready\":false"));
        assert!(json.contains("\"gate_count\":5"));
        assert!(json.contains("\"completion_window_start\":\"gamma-0.5.*\""));
        assert!(json.contains("\"completion_window_end\":\"gamma-0.10.*\""));
        assert!(text.contains("status: preparing-foundation"));
        assert!(text.contains("gates: 1/5"));
    }

    #[test]
    fn duplicate_required_gate_is_rejected() {
        let duplicate = CHECKED_IN_MANIFEST.replacen(
            "id = \"compiler-data-model\"",
            "id = \"bootstrap-language-subset\"",
            1,
        );
        let error = parse_bootstrap_readiness(&duplicate).expect_err("duplicate must fail");
        assert!(error.contains("duplicate self-hosting readiness gate"));
    }

    #[test]
    fn stable_status_requires_exactly_one_hundred_progress() {
        let invalid = CHECKED_IN_MANIFEST.replacen(
            "status = \"usable\"\nprogress = 92",
            "status = \"stable\"\nprogress = 92",
            1,
        );
        let error = parse_bootstrap_readiness(&invalid).expect_err("status drift must fail");
        assert!(error.contains("stable/100"));
    }

    #[test]
    fn next_gate_follows_status_progress_coordinate_order() {
        let adjusted = CHECKED_IN_MANIFEST.replacen("progress = 91", "progress = 93", 1);
        let report = parse_bootstrap_readiness(&adjusted).expect("manifest parses");
        assert_eq!(
            report.next_gate().map(|gate| gate.id.as_str()),
            Some("stage0-stage1-driver")
        );
    }

    #[test]
    fn completed_manifest_has_stable_null_next_gate_shape() {
        let complete = CHECKED_IN_MANIFEST
            .replace("status = \"usable\"", "status = \"stable\"")
            .replace("progress = 91", "progress = 100")
            .replace("progress = 92", "progress = 100");
        let report = parse_bootstrap_readiness(&complete).expect("complete manifest parses");
        let json = render_bootstrap_readiness_json(Path::new("readiness.toml"), &report);
        assert!(report.ready());
        assert!(json.contains("\"next_gate\":null"));
        assert!(json.contains("\"first_blocker\":null"));
    }
}
