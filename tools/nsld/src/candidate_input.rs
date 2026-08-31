use std::path::Path;

use nuis_artifact::parse_compiler_candidate_nsld_input;

use crate::{
    cli::Command,
    json_fields::{json_bool_field, json_string_field, json_usize_field},
};

const CONSUMPTION_CONTRACT: &str = "nsld-candidate-materialization-input-consumption-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldCandidateInputReport {
    pub(crate) input_path: String,
    pub(crate) protocol: String,
    pub(crate) consumption_contract: String,
    pub(crate) valid: bool,
    pub(crate) target_selector: String,
    pub(crate) entry_symbol: String,
    pub(crate) source_identity: usize,
    pub(crate) yir_identity: usize,
    pub(crate) return_value: usize,
    pub(crate) materialization_fold: usize,
    pub(crate) candidate_owned_yir_materialization: bool,
    pub(crate) equivalent_nsld_input: bool,
    pub(crate) native_object: bool,
    pub(crate) next_action: String,
}

pub(crate) fn nsld_candidate_input_report(path: &Path) -> Result<NsldCandidateInputReport, String> {
    let input = parse_compiler_candidate_nsld_input(path)
        .map_err(|error| format!("nsld candidate input verification failed: {error}"))?;
    Ok(NsldCandidateInputReport {
        input_path: path.display().to_string(),
        protocol: input.protocol,
        consumption_contract: CONSUMPTION_CONTRACT.to_owned(),
        valid: true,
        target_selector: input.target_selector,
        entry_symbol: input.entry_symbol,
        source_identity: input.source_identity,
        yir_identity: input.yir_identity,
        return_value: input.return_value,
        materialization_fold: input.materialization_fold,
        candidate_owned_yir_materialization: input.candidate_owned_yir_materialization,
        equivalent_nsld_input: input.equivalent_nsld_input,
        native_object: input.native_object,
        next_action: "select-registered-object-writer".to_owned(),
    })
}

pub(crate) fn run_candidate_input_command(command: &Command) -> Result<bool, String> {
    let Command::CandidateInput { input, json } = command else {
        return Ok(false);
    };
    let report = nsld_candidate_input_report(input)?;
    if *json {
        println!("{}", nsld_candidate_input_report_json(&report));
    } else {
        print_nsld_candidate_input_report(&report);
    }
    Ok(true)
}

fn print_nsld_candidate_input_report(report: &NsldCandidateInputReport) {
    println!("Nsld candidate materialization input");
    println!("  input: {}", report.input_path);
    println!("  protocol: {}", report.protocol);
    println!("  consumption_contract: {}", report.consumption_contract);
    println!("  valid: {}", report.valid);
    println!("  target_selector: {}", report.target_selector);
    println!("  entry_symbol: {}", report.entry_symbol);
    println!("  source_identity: {}", report.source_identity);
    println!("  yir_identity: {}", report.yir_identity);
    println!("  return_value: {}", report.return_value);
    println!("  materialization_fold: {}", report.materialization_fold);
    println!(
        "  candidate_owned_yir_materialization: {}",
        report.candidate_owned_yir_materialization
    );
    println!("  equivalent_nsld_input: {}", report.equivalent_nsld_input);
    println!("  native_object: {}", report.native_object);
    println!("  next_action: {}", report.next_action);
}

fn nsld_candidate_input_report_json(report: &NsldCandidateInputReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "candidate_materialization_input"),
        json_string_field("input_path", &report.input_path),
        json_string_field("protocol", &report.protocol),
        json_string_field("consumption_contract", &report.consumption_contract),
        json_bool_field("valid", report.valid),
        json_string_field("target_selector", &report.target_selector),
        json_string_field("entry_symbol", &report.entry_symbol),
        json_usize_field("source_identity", report.source_identity),
        json_usize_field("yir_identity", report.yir_identity),
        json_usize_field("return_value", report.return_value),
        json_usize_field("materialization_fold", report.materialization_fold),
        json_bool_field(
            "candidate_owned_yir_materialization",
            report.candidate_owned_yir_materialization,
        ),
        json_bool_field("equivalent_nsld_input", report.equivalent_nsld_input),
        json_bool_field("native_object", report.native_object),
        json_string_field("next_action", &report.next_action),
    ];
    format!("{{{}}}", fields.join(","))
}

#[cfg(test)]
mod tests {
    use super::{nsld_candidate_input_report, run_candidate_input_command};
    use crate::cli::Command;
    use nuis_artifact::{
        build_compiler_candidate_nsld_input, render_compiler_candidate_nsld_input,
        COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT, COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL,
    };
    use std::{fs, path::PathBuf};

    fn fixture_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "nsld-candidate-input-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn report_consumes_candidate_owned_input_without_selecting_a_writer() {
        let path = fixture_path("ok");
        let input = build_compiler_candidate_nsld_input(COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT)
            .expect("build candidate input");
        fs::write(&path, render_compiler_candidate_nsld_input(&input))
            .expect("write candidate input");
        let report = nsld_candidate_input_report(&path).expect("consume candidate input");
        fs::remove_file(path).ok();

        assert_eq!(report.protocol, COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL);
        assert_eq!(report.yir_identity, 9_279_238_763);
        assert_eq!(report.next_action, "select-registered-object-writer");
        assert!(report.candidate_owned_yir_materialization);
        assert!(report.equivalent_nsld_input);
        assert!(!report.native_object);
    }

    #[test]
    fn dispatcher_claims_only_candidate_input_command() {
        let missing = fixture_path("missing");
        let error = run_candidate_input_command(&Command::CandidateInput {
            input: missing,
            json: false,
        })
        .expect_err("missing candidate input must fail");
        assert!(error.contains("candidate input verification failed"));
        assert!(!run_candidate_input_command(&Command::Status).unwrap());
    }
}
