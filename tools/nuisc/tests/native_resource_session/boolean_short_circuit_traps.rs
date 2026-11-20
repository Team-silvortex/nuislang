use super::boolean_short_circuit::helpers;
use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::compile;
use super::*;
use std::os::unix::process::ExitStatusExt;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct TemporaryRoot(PathBuf);
impl Drop for TemporaryRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bool_probe_operand(expression: &str) -> Option<&str> {
    let operand = expression.strip_prefix("word(")?.strip_suffix(')')?;
    let mut depth = 0usize;
    // These fixed fixtures contain no strings: accept one complete outer call,
    // not an eager arithmetic expression that merely begins with that call.
    for byte in operand.bytes() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            _ => {}
        }
    }
    (depth == 0).then_some(operand)
}

#[test]
fn standalone_bool_probe_requires_one_complete_word_call() {
    assert_eq!(
        bool_probe_operand("word(a && choose(b).value)"),
        Some("a && choose(b).value")
    );
    for expression in [
        "word(a) + 100 / zero",
        "word(a) + word(b)",
        "word(a",
        "word(a))",
    ] {
        assert!(bool_probe_operand(expression).is_none(), "{expression}");
    }
    assert_eq!(
        super::resource_record_operands::TRAPS
            .iter()
            .filter(|(_, value, _)| bool_probe_operand(value).is_some())
            .count(),
        16
    );
}

#[test]
fn short_circuit_rhs_traps_only_when_selected_in_a_separate_process() {
    const CASE: &str = "NUIS_RESOURCE_BOOL_TRAP_CASE";
    if let Ok(case) = std::env::var(CASE) {
        let inline = case.strip_prefix("inline_");
        let returned = case.strip_prefix("field_");
        let record_local = case.strip_prefix("record_");
        let literal_local = case.strip_prefix("literal_");
        let member_local = case.strip_prefix("member_");
        let bound_local = case.strip_prefix("bound_");
        let late_local = case.strip_prefix("late_");
        let scalar_operand = case.strip_prefix("scalar_operand_");
        let bool_operand = case.strip_prefix("bool_operand_");
        let condition_operand = case.strip_prefix("condition_operand_");
        let operand_local = case
            .strip_prefix("operand_")
            .or(scalar_operand)
            .or(bool_operand)
            .or(condition_operand);
        let error_return = case.strip_prefix("error_");
        let bare_error = error_return.and_then(|key| key.strip_prefix("bare_"));
        let key = inline
            .or(returned)
            .or(record_local)
            .or(literal_local)
            .or(member_local)
            .or(bound_local)
            .or(late_local)
            .or(operand_local)
            .or(bare_error)
            .or(error_return)
            .unwrap_or(&case);
        let integer = super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .find(|(name, _, _)| *name == key)
            .filter(|_| operand_local.is_none())
            .or_else(|| {
                operand_local.and_then(|_| {
                    super::resource_record_operands::TRAPS
                        .iter()
                        .find(|(name, _, _)| *name == key)
                })
            });
        let expression = match key {
            "and_skip" => "mode != 0 && choose(100 / mode > 1).value",
            "or_skip" => "mode == 0 || choose(100 / mode > 1).value",
            "and_take" => "mode == 0 && choose(100 / mode > 1).value",
            "or_take" => "mode != 0 || choose(100 / mode > 1).value",
            "not_and_skip" => "!(mode != 0 && choose(100 / mode > 1).value)",
            "not_or_skip" => "!(mode == 0 || choose(100 / mode > 1).value)",
            "not_and_take" => "!(mode == 0 && choose(100 / mode > 1).value)",
            "not_or_take" => "!(mode != 0 || choose(100 / mode > 1).value)",
            "read_and_skip" => "mode != 0 && load_at(pixels, 99) > 0",
            "read_or_skip" => "mode == 0 || load_at(pixels, 99) > 0",
            "read_and_take" => "mode == 0 && load_at(pixels, 99) > 0",
            "read_or_take" => "mode != 0 || load_at(pixels, 99) > 0",
            "read_not_and_skip" => "!(mode != 0 && load_at(pixels, 99) > 0)",
            "read_not_or_skip" => "!(mode == 0 || load_at(pixels, 99) > 0)",
            "read_not_and_take" => "!(mode == 0 && load_at(pixels, 99) > 0)",
            "read_not_or_take" => "!(mode != 0 || load_at(pixels, 99) > 0)",
            "nested_and_skip" => "mode != 0 && (bytes_len(saved) == 0 || load_at(pixels, 99) > 0)",
            "nested_or_skip" => "mode == 0 || (bytes_len(saved) > 0 && load_at(pixels, 99) > 0)",
            "nested_and_take" => "mode == 0 && (bytes_len(saved) == 0 || load_at(pixels, 99) > 0)",
            "nested_or_take" => "mode != 0 || (bytes_len(saved) > 0 && load_at(pixels, 99) > 0)",
            "nested_inner_and_skip" => "mode == 0 && (bytes_len(saved) == 0 && load_at(pixels, 99) > 0)",
            "nested_inner_or_skip" => "mode == 0 && (bytes_len(saved) > 0 || load_at(pixels, 99) > 0)",
            "nested_left_skip" => "(mode != 0 && load_at(pixels, 99) > 0) || bytes_len(saved) > 0",
            "nested_left_take" => "(mode == 0 && load_at(pixels, 99) > 0) || bytes_len(saved) > 0",
            "nested_not_and_skip" => "!(mode != 0 && (bytes_len(saved) == 0 || load_at(pixels, 99) > 0))",
            "nested_not_or_skip" => "!(mode == 0 || (bytes_len(saved) > 0 && load_at(pixels, 99) > 0))",
            "nested_not_and_take" => "!(mode == 0 && (bytes_len(saved) == 0 || load_at(pixels, 99) > 0))",
            "nested_not_or_take" => "!(mode != 0 || (bytes_len(saved) > 0 && load_at(pixels, 99) > 0))",
            "nested_not_left_skip" => "!((mode != 0 && load_at(pixels, 99) > 0) || bytes_len(saved) == 0)",
            "nested_not_left_take" => "!((mode == 0 && load_at(pixels, 99) > 0) || bytes_len(saved) == 0)",
            "nested_both_skip" => "(mode != 0 && load_at(pixels, 99) > 0) || (bytes_len(saved) > 0 || load_at(pixels, 99) > 0)",
            "nested_both_take" => "(mode != 0 && load_at(pixels, 99) > 0) || (bytes_len(saved) == 0 || load_at(pixels, 99) > 0)",
            "helper_and_skip" => "mode != 0 && positive(load_at(pixels, 99))",
            "helper_or_skip" => "mode == 0 || choose(load_at(pixels, 99) > 0).value",
            "helper_and_take" => "mode == 0 && positive(load_at(pixels, 99))",
            "helper_or_take" => "mode != 0 || choose(load_at(pixels, 99) > 0).value",
            "helper_not_and_skip" => "!(mode != 0 && positive(load_at(pixels, 99)))",
            "helper_not_or_skip" => "!(mode == 0 || choose(load_at(pixels, 99) > 0).value)",
            "helper_not_and_take" => "!(mode == 0 && positive(load_at(pixels, 99)))",
            "helper_not_or_take" => "!(mode != 0 || choose(load_at(pixels, 99) > 0).value)",
            "helper_body_and_skip" => "mode != 0 && divide(load_at(pixels, 0) - 9)",
            "helper_body_or_skip" => "mode == 0 || divide(load_at(pixels, 0) - 9)",
            "helper_body_and_take" => "mode == 0 && divide(load_at(pixels, 0) - 9)",
            "helper_body_or_take" => "mode != 0 || divide(load_at(pixels, 0) - 9)",
            "helper_bytes_and_skip" => "mode != 0 && divide(bytes_len(saved) - 48)",
            "helper_bytes_or_skip" => "mode == 0 || divide(bytes_len(saved) - 48)",
            "helper_bytes_and_take" => "mode == 0 && divide(bytes_len(saved) - 48)",
            "helper_bytes_or_take" => "mode != 0 || divide(bytes_len(saved) - 48)",
            "argument_and_skip" => "choose(mode != 0 && load_at(pixels, 99) > 0).value",
            "argument_or_skip" => "choose(mode == 0 || load_at(pixels, 99) > 0).value",
            "argument_and_take" => "choose(mode == 0 && load_at(pixels, 99) > 0).value",
            "argument_or_take" => "choose(mode != 0 || load_at(pixels, 99) > 0).value",
            "argument_not_and_skip" => "!choose(mode != 0 && load_at(pixels, 99) > 0).value",
            "argument_not_or_skip" => "!choose(mode == 0 || load_at(pixels, 99) > 0).value",
            "argument_not_and_take" => "!choose(mode == 0 && load_at(pixels, 99) > 0).value",
            "argument_not_or_take" => "!choose(mode != 0 || load_at(pixels, 99) > 0).value",
            "argument_outer_and_skip" => "mode != 0 && choose(mode == 0 && load_at(pixels, 99) > 0).value",
            "argument_outer_or_skip" => "mode == 0 || choose(mode != 0 || load_at(pixels, 99) > 0).value",
            "argument_outer_and_take" => "mode == 0 && choose(mode == 0 && load_at(pixels, 99) > 0).value",
            "argument_outer_or_take" => "mode != 0 || choose(mode != 0 || load_at(pixels, 99) > 0).value",
            "argument_left_take" => "arguments(load_at(pixels, 99), mode != 0 && bytes_len(saved) == 48, 9).value",
            "argument_right_take" => "arguments(9, mode != 0 && bytes_len(saved) == 48, load_at(pixels, 99)).value",
            "argument_nested_skip" => "choose(mode == 0 || choose(mode == 0 && load_at(pixels, 99) > 0).value).value",
            "argument_nested_take" => "choose(mode != 0 || choose(mode == 0 && load_at(pixels, 99) > 0).value).value",
            "argument_first_take" => "both(mode == 0 && load_at(pixels, 99) > 0, mode != 0 && bytes_len(saved) > 0).value",
            "argument_second_take" => "both(mode != 0 && bytes_len(saved) > 0, mode == 0 && load_at(pixels, 99) > 0).value",
            "argument_first_skip" => "both(mode != 0 && load_at(pixels, 99) > 0, mode == 0 || load_at(pixels, 99) > 0).value",
            "argument_second_skip" => "both(mode == 0 || load_at(pixels, 99) > 0, mode != 0 && load_at(pixels, 99) > 0).value",
            "select_and_skip" => "word(mode != 0 && load_at(pixels, 99) > 0) == 9",
            "select_or_skip" => "word(mode == 0 || load_at(pixels, 99) > 0) == 9",
            "select_and_take" => "word(mode == 0 && load_at(pixels, 99) > 0) == 9",
            "select_or_take" => "word(mode != 0 || load_at(pixels, 99) > 0) == 9",
            "select_not_and_skip" => "word(!(mode != 0 && load_at(pixels, 99) > 0)) == 9",
            "select_not_or_skip" => "word(!(mode == 0 || load_at(pixels, 99) > 0)) == 9",
            "select_not_and_take" => "word(!(mode == 0 && load_at(pixels, 99) > 0)) == 9",
            "select_not_or_take" => "word(!(mode != 0 || load_at(pixels, 99) > 0)) == 9",
            "select_body_and_skip" => "word(mode != 0 && divide(bytes_len(saved) - 48)) == 9",
            "select_body_and_take" => "word(mode == 0 && divide(bytes_len(saved) - 48)) == 9",
            "select_bool_and_skip" => "gate(mode != 0 && load_at(pixels, 99) > 0, mode >= 0)",
            "select_bool_and_take" => "gate(mode == 0 && load_at(pixels, 99) > 0, mode >= 0)",
            "select_conditional_and_skip" => "ratio(mode != 0, bytes_len(saved) - 48) > 0",
            "select_conditional_and_take" => "ratio(mode == 0, bytes_len(saved) - 48) > 0",
            "select_conditional_or_skip" => "inverse_ratio(mode == 0, bytes_len(saved) - 48) > 0",
            "select_conditional_or_take" => "inverse_ratio(mode != 0, bytes_len(saved) - 48) > 0",
            "wrap_eq_and_skip" => "(mode != 0 && load_at(pixels, 99) > 0) == (mode < 0)",
            "wrap_eq_and_take" => "(mode == 0 && load_at(pixels, 99) > 0) == (mode < 0)",
            "wrap_ne_or_skip" => "(mode == 0 || load_at(pixels, 99) > 0) != (mode < 0)",
            "wrap_ne_or_take" => "(mode != 0 || load_at(pixels, 99) > 0) != (mode < 0)",
            "wrap_xor_and_skip" => "(mode != 0 && load_at(pixels, 99) > 0) ^ (mode == 0)",
            "wrap_xor_and_take" => "(mode == 0 && load_at(pixels, 99) > 0) ^ (mode == 0)",
            "wrap_xor_or_skip" => "(mode == 0 || load_at(pixels, 99) > 0) ^ (mode == 0)",
            "wrap_xor_or_take" => "(mode != 0 || load_at(pixels, 99) > 0) ^ (mode == 0)",
            "wrap_both_skip" => "(mode != 0 && load_at(pixels, 99) > 0) ^ (mode == 0 || load_at(pixels, 99) > 0)",
            "wrap_left_take" => "(mode == 0 && load_at(pixels, 99) > 0) ^ (mode == 0 || load_at(pixels, 99) > 0)",
            "wrap_right_take" => "(mode != 0 && load_at(pixels, 99) > 0) ^ (mode != 0 || load_at(pixels, 99) > 0)",
            "wrap_eq_right_take" => "(mode != 0 && bytes_len(saved) > 0) == (load_at(pixels, 99) > 0)",
            "wrap_ne_right_take" => "(mode == 0 || bytes_len(saved) > 0) != (load_at(pixels, 99) > 0)",
            "wrap_eager_left_take" => "(load_at(pixels, 99) > 0) == (mode != 0 && bytes_len(saved) > 0)",
            "wrap_outer_and_skip" => "mode != 0 && ((mode == 0 && load_at(pixels, 99) > 0) ^ true)",
            "wrap_outer_and_take" => "mode == 0 && ((mode == 0 && load_at(pixels, 99) > 0) ^ true)",
            "wrap_outer_or_skip" => "mode == 0 || ((mode == 0 && load_at(pixels, 99) > 0) == true)",
            "wrap_outer_or_take" => "mode != 0 || ((mode == 0 && load_at(pixels, 99) > 0) == true)",
            "wrap_not_skip" => "!((mode != 0 && load_at(pixels, 99) > 0) ^ (mode != 0))",
            "wrap_not_take" => "!((mode == 0 && load_at(pixels, 99) > 0) ^ false)",
            "wrap_argument_skip" => "choose((mode != 0 && load_at(pixels, 99) > 0) ^ (bytes_len(saved) == 48)).value",
            "wrap_argument_take" => "choose((mode == 0 && load_at(pixels, 99) > 0) ^ (bytes_len(saved) == 48)).value",
            "wrap_div_skip" => "(mode != 0 && divide(bytes_len(saved) - 48)) == (mode < 0)",
            "wrap_div_take" => "(mode == 0 && divide(bytes_len(saved) - 48)) == (mode < 0)",
            "wrap_index_skip" => "load_at(pixels, word((mode != 0 && load_at(pixels, 99) > 0) == (mode == 0))) > 0",
            "wrap_index_take" => "load_at(pixels, word((mode == 0 && load_at(pixels, 99) > 0) == (mode == 0))) > 0",
            "wrap_order_skip" => "word(mode != 0 && load_at(pixels, 99) > 0) < load_at(pixels, 3)",
            "wrap_order_take" => "word(mode == 0 && load_at(pixels, 99) > 0) < load_at(pixels, 3)",
            "wrap_mandatory_div_take" => "(mode != 0 && bytes_len(saved) > 0) == divide(bytes_len(saved) - 48)",
            "wrap_mandatory_div_left_take" => "divide(bytes_len(saved) - 48) ^ (mode != 0 && bytes_len(saved) > 0)",
            _ if integer.is_some() || literal_local.is_some() || member_local.is_some() || bound_local.is_some() || late_local.is_some() => "choice == 9",
            _ => panic!("unknown child case"),
        };
        let source = helpers(&format!(
            r#"
          let saved: Bytes = copy_bytes(pixels);
          if mode >= 0 {{
            let decision: bool = {expression};
            if decision {{ return Result.Ok(Canvas {{ width: width + 10, pixels: move(saved) }}); }}
          }}
          drop_bytes(saved);
          store_at(pixels, 0, width);
          return Result.Ok(Canvas {{ width: width + 1, pixels: copy_bytes(pixels) }});
        "#
        ));
        let source = if operand_local.is_some() {
            let value = integer.unwrap().1;
            let zero = match key {
                "init_read_take" => "load_at(pixels, 99)",
                "init_div_take" => "100 / mode",
                _ => "mode",
            };
            let tail = if condition_operand.is_some() {
                format!("let original: i64 = prior.zero; if ({value}) != 0 {{ return Result.Ok(Canvas {{ width: width + 10 + original, pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: width + 1 + original, pixels: move(saved) }});")
            } else if bool_operand.is_some() {
                let predicate = bool_probe_operand(value).expect("a complete outer word call");
                format!("let original: i64 = prior.zero; let choice: bool = {predicate}; return Result.Ok(Canvas {{ width: width + 1 + word(choice) + original, pixels: move(saved) }});")
            } else if scalar_operand.is_some() {
                format!("let original: i64 = prior.zero; let choice: i64 = {value}; return Result.Ok(Canvas {{ width: width + 1 + choice + original, pixels: move(saved) }});")
            } else {
                format!("let local = ProbeEnvelope {{ after: {value}, inner: prior }}; return Result.Ok(Canvas {{ width: width + 1 + local.after + local.inner.zero, pixels: move(saved) }});")
            };
            helpers(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let prior = ProbeInput {{ enabled: mode != 0, index: 99, zero: {zero} }}; {tail}"))
                .replace("mod cpu Main {", "mod cpu Main { struct ProbeInput { enabled: bool, index: i64, zero: i64 } struct ProbeEnvelope { inner: ProbeInput, after: i64 }")
        } else if bound_local.is_some() || late_local.is_some() {
            let (prior, after) = if let Some((_, value, _)) = integer {
                ("record_value(0)", *value)
            } else {
                (
                    if key.contains("read") {
                        "record_value(load_at(pixels, 99))"
                    } else {
                        "record_div(bytes_len(saved) - 48)"
                    },
                    "word(mode != 0 && bytes_len(saved) > 0)",
                )
            };
            let fields = if late_local.is_some() {
                format!("after: {after}, inner: prior")
            } else {
                format!("inner: prior, after: {after}")
            };
            helpers(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let prior: ProbeNumber = {prior}; let local = ProbeEnvelope {{ {fields} }}; return Result.Ok(Canvas {{ width: width + 1 + local.inner.value + local.after, pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { struct ProbeNumber { value: i64, hidden: i64 } struct ProbeEnvelope { inner: ProbeNumber, after: i64 } fn record_value(value: i64) -> ProbeNumber { return ProbeNumber { value: value, hidden: 0 }; } fn record_div(value: i64) -> ProbeNumber { return ProbeNumber { value: 0, hidden: 100 / value }; }")
        } else if member_local.is_some() {
            let value = if let Some((_, value, _)) = integer {
                format!("ProbeEnvelope {{ inner: record_value({value}), after: 0 }}")
            } else {
                let eager = if key.contains("read") {
                    "record_value(load_at(pixels, 99))"
                } else {
                    "record_div(bytes_len(saved) - 48)"
                };
                let lazy = "word(mode != 0 && bytes_len(saved) > 0)";
                if key.contains("before") {
                    format!("ProbeEnvelope {{ inner: {eager}, after: {lazy} }}")
                } else {
                    format!("ProbeEnvelope {{ after: {lazy}, inner: {eager} }}")
                }
            };
            helpers(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let local: ProbeEnvelope = {value}; return Result.Ok(Canvas {{ width: width + 1 + local.inner.value, pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { struct ProbeNumber { value: i64, hidden: i64 } struct ProbeEnvelope { inner: ProbeNumber, after: i64 } fn record_value(value: i64) -> ProbeNumber { return ProbeNumber { value: value, hidden: 0 }; } fn record_div(value: i64) -> ProbeNumber { return ProbeNumber { value: 9, hidden: 100 / value }; }")
        } else if literal_local.is_some() {
            let value = if let Some((_, value, _)) = integer {
                format!("ProbeFields {{ eager: 0, value: {value} }}")
            } else {
                let eager = if key.contains("read") {
                    "load_at(pixels, 99)"
                } else {
                    "100 / mode"
                };
                let lazy = "word(mode != 0 && bytes_len(saved) > 0)";
                if key.contains("before") {
                    format!("ProbeFields {{ eager: {eager}, value: {lazy} }}")
                } else {
                    format!("ProbeFields {{ value: {lazy}, eager: {eager} }}")
                }
            };
            helpers(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let local: ProbeFields = {value}; return Result.Ok(Canvas {{ width: width + 1 + local.value, pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { struct ProbeFields { value: i64, eager: i64 }")
        } else if record_local.is_some() {
            let (_, value, _) = integer.unwrap();
            helpers(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let local: ProbeNumber = record_value({value}); return Result.Ok(Canvas {{ width: width + 1 + local.value, pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { struct ProbeNumber { value: i64 } fn record_value(value: i64) -> ProbeNumber { return ProbeNumber { value: value }; }")
        } else if error_return.is_some() {
            let (_, value, _) = integer.unwrap();
            let prefix = if bare_error.is_some() {
                ""
            } else {
                "let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 1, 99);"
            };
            helpers(&format!("{prefix} return Result.Err({value});"))
        } else if returned.is_some() {
            let (_, value, _) = integer.unwrap();
            helpers(&format!("let saved: Bytes = copy_bytes(pixels); return Result.Ok(Canvas {{ width: width + 1 + {value}, pixels: move(saved) }});"))
        } else {
            source
        };
        let source = source.replace(
            "mod cpu Main {",
            "mod cpu Main { fn divide(value: i64) -> bool { return 100 / value > 0; } fn arguments(left: i64, flag: bool, right: i64) -> Choice { return Choice { value: flag && left == right }; }",
        );
        let source = source.replace("mod cpu Main {", "mod cpu Main { fn both(first: bool, second: bool) -> Choice { return Choice { value: first && second }; }");
        let source = source.replace("mod cpu Main {", "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 9; } return 0; } fn gate(flag: bool, value: bool) -> bool { if flag { return value; } return false; }");
        let source = source.replace("mod cpu Main {", "mod cpu Main { fn ratio(flag: bool, value: i64) -> i64 { if flag { return 100 / value; } return 0; } fn inverse_ratio(flag: bool, value: i64) -> i64 { if flag { return 9; } return 100 / value; }");
        let source = if inline.is_some() {
            let condition = integer
                .map(|(_, value, _)| format!("{value} == 9"))
                .unwrap_or_else(|| expression.to_owned());
            source
                .replace(&format!("let decision: bool = {expression};"), "")
                .replace("if decision {", &format!("if {condition} {{"))
        } else if error_return.is_some()
            || record_local.is_some()
            || literal_local.is_some()
            || member_local.is_some()
            || bound_local.is_some()
            || late_local.is_some()
            || operand_local.is_some()
        {
            source
        } else if let Some((_, value, _)) = integer.filter(|_| returned.is_none()) {
            source.replace(
                "let decision: bool = choice == 9;",
                &format!("let choice: i64 = {value}; let decision: bool = choice == 9;"),
            )
        } else {
            source
        };
        let program = compile(&source);
        let bridge = bridge(&program);
        let library = native::Library::build(&source, &program, &bridge.llvm_ir);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        eprintln!("before-short-circuit-event");
        if error_return.is_some() {
            let code = if integer.unwrap().2 { 9 } else { 0 };
            let error = session.event(0).unwrap_err();
            assert!(error.contains(&format!("code={code}")), "{error}");
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(session.scalar_state(), [3]);
            assert_eq!(library.live_blobs(), 0);
            session.close(0).unwrap();
            assert!(session.completion_status().is_err());
            assert_eq!(library.live_blobs(), 0);
            eprintln!("after-short-circuit-event");
            return;
        }
        session.event(0).unwrap();
        if integer
            .map(|(_, _, decision)| !decision)
            .unwrap_or_else(|| {
                matches!(
                    key.strip_prefix("read_")
                        .or_else(|| key.strip_prefix("nested_"))
                        .or_else(|| key.strip_prefix("helper_"))
                        .or_else(|| key.strip_prefix("argument_"))
                        .or_else(|| key.strip_prefix("select_"))
                        .or_else(|| key.strip_prefix("wrap_"))
                        .unwrap_or(key),
                    "and_skip"
                        | "not_or_skip"
                        | "inner_and_skip"
                        | "body_and_skip"
                        | "bytes_and_skip"
                        | "outer_and_skip"
                        | "first_skip"
                        | "second_skip"
                        | "bool_and_skip"
                        | "conditional_and_skip"
                        | "xor_or_skip"
                )
            })
        {
            assert_eq!(
                session.state(),
                if returned.is_some()
                    || record_local.is_some()
                    || literal_local.is_some()
                    || member_local.is_some()
                    || bound_local.is_some()
                    || late_local.is_some()
                    || operand_local.is_some()
                {
                    [9; 6]
                } else {
                    [3, 9, 9, 9, 9, 9]
                }
            );
            assert_eq!(session.scalar_state(), [4]);
        } else {
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(session.scalar_state(), [13]);
        }
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
        eprintln!("after-short-circuit-event");
        return;
    }
    for case in [
        "and_skip",
        "or_skip",
        "and_take",
        "or_take",
        "not_and_skip",
        "not_or_skip",
        "not_and_take",
        "not_or_take",
        "read_and_skip",
        "read_or_skip",
        "read_and_take",
        "read_or_take",
        "read_not_and_skip",
        "read_not_or_skip",
        "read_not_and_take",
        "read_not_or_take",
        "nested_and_skip",
        "nested_or_skip",
        "nested_and_take",
        "nested_or_take",
        "nested_inner_and_skip",
        "nested_inner_or_skip",
        "nested_left_skip",
        "nested_left_take",
        "nested_not_and_skip",
        "nested_not_or_skip",
        "nested_not_and_take",
        "nested_not_or_take",
        "nested_not_left_skip",
        "nested_not_left_take",
        "nested_both_skip",
        "nested_both_take",
        "helper_and_skip",
        "helper_or_skip",
        "helper_and_take",
        "helper_or_take",
        "helper_not_and_skip",
        "helper_not_or_skip",
        "helper_not_and_take",
        "helper_not_or_take",
        "helper_body_and_skip",
        "helper_body_or_skip",
        "helper_body_and_take",
        "helper_body_or_take",
        "helper_bytes_and_skip",
        "helper_bytes_or_skip",
        "helper_bytes_and_take",
        "helper_bytes_or_take",
        "argument_and_skip",
        "argument_or_skip",
        "argument_and_take",
        "argument_or_take",
        "argument_not_and_skip",
        "argument_not_or_skip",
        "argument_not_and_take",
        "argument_not_or_take",
        "argument_outer_and_skip",
        "argument_outer_or_skip",
        "argument_outer_and_take",
        "argument_outer_or_take",
        "argument_left_take",
        "argument_right_take",
        "argument_nested_skip",
        "argument_nested_take",
        "argument_first_take",
        "argument_second_take",
        "argument_first_skip",
        "argument_second_skip",
        "select_and_skip",
        "select_or_skip",
        "select_and_take",
        "select_or_take",
        "select_not_and_skip",
        "select_not_or_skip",
        "select_not_and_take",
        "select_not_or_take",
        "select_body_and_skip",
        "select_body_and_take",
        "select_bool_and_skip",
        "select_bool_and_take",
        "select_conditional_and_skip",
        "select_conditional_and_take",
        "select_conditional_or_skip",
        "select_conditional_or_take",
        "wrap_eq_and_skip",
        "wrap_eq_and_take",
        "wrap_ne_or_skip",
        "wrap_ne_or_take",
        "wrap_xor_and_skip",
        "wrap_xor_and_take",
        "wrap_xor_or_skip",
        "wrap_xor_or_take",
        "wrap_both_skip",
        "wrap_left_take",
        "wrap_right_take",
        "wrap_eq_right_take",
        "wrap_ne_right_take",
        "wrap_eager_left_take",
        "wrap_outer_and_skip",
        "wrap_outer_and_take",
        "wrap_outer_or_skip",
        "wrap_outer_or_take",
        "wrap_not_skip",
        "wrap_not_take",
        "wrap_argument_skip",
        "wrap_argument_take",
        "wrap_div_skip",
        "wrap_div_take",
        "wrap_index_skip",
        "wrap_index_take",
        "wrap_order_skip",
        "wrap_order_take",
        "wrap_mandatory_div_take",
        "wrap_mandatory_div_left_take",
    ]
    .into_iter()
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| *name),
    )
    .chain(super::resource_conditions::TRAPS.iter().copied())
    .map(str::to_owned)
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| format!("field_{name}")),
    )
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| format!("error_{name}")),
    )
    .chain(["error_bare_i64_and_skip", "error_bare_i64_and_take"].map(str::to_owned))
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| format!("record_{name}")),
    )
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| format!("literal_{name}")),
    )
    .chain(
        [
            "literal_field_before_read_take",
            "literal_field_after_read_take",
            "literal_field_before_div_take",
            "literal_field_after_div_take",
        ]
        .map(str::to_owned),
    )
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| format!("member_{name}")),
    )
    .chain(
        [
            "member_field_before_read_take",
            "member_field_after_read_take",
            "member_field_before_div_take",
            "member_field_after_div_take",
        ]
        .map(str::to_owned),
    )
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| format!("bound_{name}")),
    )
    .chain(["bound_init_read_take", "bound_init_div_take"].map(str::to_owned))
    .chain(
        super::resource_i64_initializers::TRAPS
            .iter()
            .chain(super::resource_buffer_lengths::TRAPS)
            .map(|(name, _, _)| format!("late_{name}")),
    )
    .chain(["late_init_read_take", "late_init_div_take"].map(str::to_owned))
    .chain(
        super::resource_record_operands::TRAPS
            .iter()
            .map(|(name, _, _)| format!("operand_{name}")),
    )
    .chain(
        super::resource_record_operands::TRAPS
            .iter()
            .map(|(name, _, _)| format!("scalar_operand_{name}")),
    )
    .chain(
        super::resource_record_operands::TRAPS
            .iter()
            .filter(|(_, value, _)| bool_probe_operand(value).is_some())
            .map(|(name, _, _)| format!("bool_operand_{name}")),
    )
    .chain(
        super::resource_record_operands::TRAPS
            .iter()
            .map(|(name, _, _)| format!("condition_operand_{name}")),
    ) {
        let root = TemporaryRoot(
            std::env::temp_dir().join(format!("nuis-short-circuit-{}-{case}", std::process::id())),
        );
        fs::create_dir(&root.0).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "boolean_short_circuit_traps::short_circuit_rhs_traps_only_when_selected_in_a_separate_process", "--nocapture"])
            .env(CASE, &case).env("NUIS_RESOURCE_TRAP_ROOT", &root.0)
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("short-circuit probe exceeded deadline: {case}");
            }
            thread::sleep(Duration::from_millis(20));
        }
        let output = child.wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("before-short-circuit-event"),
            "{case}: {stderr}"
        );
        if case.ends_with("skip") {
            assert!(
                output.status.success(),
                "{case}: {} {stderr}",
                output.status
            );
            assert!(stderr.contains("after-short-circuit-event"), "{stderr}");
        } else {
            assert!(
                matches!(output.status.signal(), Some(4 | 5)),
                "{case}: {} {stderr}",
                output.status
            );
            assert!(!stderr.contains("after-short-circuit-event"));
        }
    }
}
