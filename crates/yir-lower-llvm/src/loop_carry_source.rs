pub(crate) fn resolve_loop_carry_term(
    term: &str,
    carry_kind: &str,
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    match term {
        "current" => Ok(next_current.to_owned()),
        "prev_current" => Ok(current.to_owned()),
        other if other.starts_with("prev_carry") => {
            let source_index = other[10..].parse::<usize>().map_err(|_| {
                format!(
                    "cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                )
            })?;
            current_carries.get(source_index).cloned().ok_or_else(|| {
                format!(
                    "cpu.{loop_instruction} `{node_name}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                )
            })
        }
        other if other.starts_with("carry") => {
            let source_index = other[5..].parse::<usize>().map_err(|_| {
                format!(
                    "cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                )
            })?;
            next_carries.get(source_index).cloned().ok_or_else(|| {
                format!(
                    "cpu.{loop_instruction} `{node_name}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                )
            })
        }
        _ => Err(format!(
            "cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
        )),
    }
}
