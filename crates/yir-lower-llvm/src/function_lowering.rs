use super::*;

#[macro_use]
mod loop_async_cond_chain;
#[macro_use]
mod loop_cond_chain;
#[macro_use]
mod loop_async_chain;
#[macro_use]
mod loop_flow_chain;
#[macro_use]
mod loop_async_flow_chain;
#[macro_use]
mod loop_async_flow_cond_chain;
#[macro_use]
mod loop_post_flow_chain;
#[macro_use]
mod loop_async_post_flow_chain;
#[macro_use]
mod loop_post_flow_cond_chain;
#[macro_use]
mod loop_flow_cond_chain;

pub(super) fn emit_cpu_function(
    module: &YirModule,
    resources: &BTreeMap<String, &Resource>,
    ordered_node_names: &[String],
    param_bindings: &BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    function_return_kind: CpuCallScalarKind,
    global_counter: &mut usize,
) -> Result<EmittedCpuFunction, String> {
    let mut state = LlvmLoweringState {
        body: Vec::new(),
        globals: Vec::new(),
        registers: BTreeMap::new(),
        buffer_lengths: BTreeMap::new(),
        next_reg: 0,
        next_global: *global_counter,
        next_block: 0,
        last_cpu_value: None,
        ends_with_terminal_return: false,
    };

    for (node_name, value) in param_bindings {
        state.registers.insert(node_name.clone(), value.clone());
        match value {
            LlvmValueRef::Bool { i1, i64 } => {
                state.body.push(format!("  {i64} = zext i1 {i1} to i64"));
                state.last_cpu_value = Some(i64.clone());
            }
            LlvmValueRef::I32(reg) => {
                let widened = fresh_reg(&mut state.next_reg);
                state
                    .body
                    .push(format!("  {widened} = sext i32 {reg} to i64"));
                state.last_cpu_value = Some(widened);
            }
            LlvmValueRef::I64(reg) => state.last_cpu_value = Some(reg.clone()),
            _ => {}
        }
    }

    for node_name in ordered_node_names {
        let node = module
            .nodes
            .iter()
            .find(|node| &node.name == node_name)
            .ok_or_else(|| format!("lowering references unknown node `{node_name}`"))?;
        let resource = resources
            .get(&node.resource)
            .copied()
            .ok_or_else(|| format!("unknown resource `{}`", node.resource))?;
        if state.ends_with_terminal_return {
            continue;
        }

        if resource.kind.is_family("network") && lower_network_observer_node(node, &mut state) {
            continue;
        }

        if !resource.kind.is_family("cpu") {
            state.body.push(format!(
                "  ; deferred lowering for {} on {} ({})",
                node.op.full_name(),
                node.resource,
                resource.kind.raw
            ));
            continue;
        }

        if lower_cpu_async_resource_node(node, &mut state) {
            continue;
        }

        match node.op.cpu_llvm_lowering_class() {
            CpuLlvmLoweringClass::Literal => {
                if lower_cpu_literal_node(node, &mut state) {
                    continue;
                }
            }
            CpuLlvmLoweringClass::Aggregate => {
                if lower_cpu_aggregate_node(node, &mut state) {
                    continue;
                }
            }
            CpuLlvmLoweringClass::Pointer => {
                if lower_cpu_pointer_node(node, &mut state) {
                    continue;
                }
            }
            _ => {}
        }

        let mut body = &mut state.body;
        let _globals = &mut state.globals;
        let registers = &mut state.registers;
        let buffer_lengths = &mut state.buffer_lengths;
        let mut next_reg = &mut state.next_reg;
        let mut next_block = &mut state.next_block;
        let _next_global = &mut state.next_global;
        let last_cpu_value = &mut state.last_cpu_value;

        if lower_cpu_extern_call_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_memory_node(
            node,
            body,
            registers,
            buffer_lengths,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        if lower_cpu_cast_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_static_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_call_node(
            node,
            body,
            registers,
            helper_signatures,
            &mut next_reg,
            last_cpu_value,
        )? {
            continue;
        }

        match lower_cpu_return_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            ReturnLoweringOutcome::NotReturn => {}
            ReturnLoweringOutcome::Deferred => continue,
            ReturnLoweringOutcome::Returned => {
                state.ends_with_terminal_return = true;
                break;
            }
        }

        if lower_cpu_print_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_bitwise_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_param_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_select_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_scalar_equality_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_scalar_order_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        if lower_cpu_scalar_node(node, body, registers, &mut next_reg, last_cpu_value)? {
            continue;
        }

        match lower_cpu_guard_return_node(
            node,
            body,
            registers,
            &mut next_reg,
            &mut next_block,
            function_return_kind,
        )? {
            GuardReturnLoweringOutcome::NotGuard => {}
            GuardReturnLoweringOutcome::Continue => continue,
            GuardReturnLoweringOutcome::TerminalReturn => {
                state.ends_with_terminal_return = true;
                continue;
            }
        }

        if lower_cpu_simple_loop_node(
            node,
            body,
            registers,
            &mut next_reg,
            &mut next_block,
            last_cpu_value,
        )? {
            continue;
        }

        match (node.op.module.as_str(), node.op.instruction.as_str()) {
            ("cpu", "text")
            | ("cpu", "const_bool")
            | ("cpu", "const_i32")
            | ("cpu", "const")
            | ("cpu", "const_i64")
            | ("cpu", "const_f32")
            | ("cpu", "const_f64")
            | ("cpu", "struct")
            | ("cpu", "field")
            | ("cpu", "variant_is")
            | ("cpu", "variant_field")
            | ("cpu", "null")
            | ("cpu", "borrow")
            | ("cpu", "borrow_end")
            | ("cpu", "move_ptr") => unreachable!(
                "preclassified CPU LLVM lowering op `{}` should have been handled earlier",
                node.op.full_name()
            ),
            ("cpu", "loop_while_i64_chain" | "loop_while_scalar_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let (Some(initial_value), Some(limit_value), Some(step_value)) =
                    (initial_value, limit_value, step_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let mut carry_initial_values = Vec::new();
                let mut carry_specs_raw = Vec::new();
                let mut cursor = 5usize;
                while cursor < node.op.args.len() {
                    let carry_initial_name = &node.op.args[cursor];
                    let carry_kind = &node.op.args[cursor + 1];
                    let payload_len = loop_carry_payload_len(carry_kind);
                    let payload_names = &node.op.args[cursor + 2..cursor + 2 + payload_len];
                    let carry_initial_value = registers.get(carry_initial_name).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        continue;
                    };
                    let mut payload_values = Vec::new();
                    let mut missing_payload = false;
                    for payload_name in payload_names {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            missing_payload = true;
                            break;
                        };
                        payload_values.push(payload_value);
                    }
                    if missing_payload {
                        continue;
                    }
                    carry_initial_values.push(carry_initial_value);
                    carry_specs_raw.push((carry_kind.clone(), payload_values));
                    cursor += 2 + payload_len;
                }
                let Some(loop_scalar_kind) = infer_loop_scalar_kind(
                    [&initial_value, &limit_value, &step_value]
                        .into_iter()
                        .chain(carry_initial_values.iter()),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its loop values are not representable as one scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_loop_scalar(
                    &initial_value,
                    loop_scalar_kind,
                    &mut body,
                    &mut next_reg,
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) =
                    coerce_to_loop_scalar(&limit_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) =
                    coerce_to_loop_scalar(&step_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let mut carry_initials = Vec::new();
                for carry_initial_value in &carry_initial_values {
                    let Some(carry_initial) = coerce_to_loop_scalar(
                        carry_initial_value,
                        loop_scalar_kind,
                        &mut body,
                        &mut next_reg,
                    ) else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to the selected loop scalar kind",
                            node.name,
                        ));
                        continue;
                    };
                    carry_initials.push(carry_initial);
                }
                let mut carry_specs = Vec::new();
                for (carry_kind, payload_values) in &carry_specs_raw {
                    let mut payloads = Vec::new();
                    for payload_value in payload_values {
                        let Some(payload) = coerce_to_loop_scalar(
                            payload_value,
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        ) else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are not coercible to the selected loop scalar kind",
                                node.name,
                            ));
                            continue;
                        };
                        payloads.push(payload);
                    }
                    carry_specs.push((carry_kind.clone(), payloads));
                }
                let scalar_ty = loop_scalar_llvm_type(loop_scalar_kind);
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca {scalar_ty}"));
                body.push(format!("  store {scalar_ty} {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca {scalar_ty}"));
                        body.push(format!(
                            "  store {scalar_ty} {carry_initial}, ptr {carry_slot}"
                        ));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let cmp = emit_loop_compare(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    cmp_kind,
                    &current,
                    &limit,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = emit_loop_numeric_op(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    step_kind,
                    &current,
                    &step,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {carry_before} = load {scalar_ty}, ptr {carry_slot}"
                    ));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, ((carry_kind, raw_payloads), (_, payloads))) in
                    carry_specs_raw.iter().zip(carry_specs.iter()).enumerate()
                {
                    let (source, op) = if carry_kind == "add_current" {
                        (next_current.clone(), "add")
                    } else if carry_kind == "add_prev_current" {
                        (current.clone(), "add")
                    } else if carry_kind == "mul_current" {
                        (next_current.clone(), "mul")
                    } else if carry_kind == "mul_prev_current" {
                        (current.clone(), "mul")
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_value_fixed" | "mul_read_value_fixed"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed read pointer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {ptr}"));
                        let source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_value_fixed_plus_invariant"
                            | "mul_read_value_fixed_plus_invariant"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed read pointer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let offset = payloads.last().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing invariant payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {ptr}"));
                        let read_source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "add",
                            &read_source,
                            offset,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_at_fixed" | "mul_read_at_fixed"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read buffer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let index_value = raw_payloads
                            .get(1)
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg))
                            .ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read index payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        let slot = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index_value}"
                        ));
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {slot}"));
                        let source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed indexed-read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_at_fixed_plus_invariant" | "mul_read_at_fixed_plus_invariant"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read buffer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let index_value = raw_payloads
                            .get(1)
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg))
                            .ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read index payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        let offset = payloads.last().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing invariant payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let slot = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index_value}"
                        ));
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {slot}"));
                        let read_source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed indexed-read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "add",
                            &read_source,
                            offset,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_at_dynamic_current"
                            | "add_read_at_dynamic_prev_current"
                            | "mul_read_at_dynamic_current"
                            | "mul_read_at_dynamic_prev_current"
                            | "add_read_at_dynamic_current_plus_invariant"
                            | "add_read_at_dynamic_prev_current_plus_invariant"
                            | "mul_read_at_dynamic_current_plus_invariant"
                            | "mul_read_at_dynamic_prev_current_plus_invariant"
                    ) || carry_kind
                        .strip_prefix("add_read_at_dynamic_prev_carry")
                        .is_some()
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_prev_carry")
                            .is_some()
                        || carry_kind
                            .strip_prefix("add_read_at_dynamic_carry")
                            .is_some()
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_carry")
                            .is_some()
                        || carry_kind
                            .strip_prefix("add_read_at_dynamic_prev_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_prev_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                        || carry_kind
                            .strip_prefix("add_read_at_dynamic_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                    {
                        let buffer_ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing dynamic read buffer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let dynamic_kind = carry_kind
                            .strip_suffix("_plus_invariant")
                            .unwrap_or(carry_kind.as_str());
                        let index_value = if dynamic_kind.ends_with("_prev_current") {
                            current.clone()
                        } else if dynamic_kind.ends_with("_current") {
                            next_current.clone()
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("add_read_at_dynamic_prev_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("mul_read_at_dynamic_prev_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("add_read_at_dynamic_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("mul_read_at_dynamic_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            ));
                        };
                        let slot = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {slot} = getelementptr inbounds i64, ptr {buffer_ptr}, i64 {index_value}"
                        ));
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {slot}"));
                        let read_source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce dynamic read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let source = if carry_kind.ends_with("_plus_invariant") {
                            let offset = payloads.last().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing invariant payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &read_source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            read_source
                        };
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if let Some(rest) = carry_kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some(prefix) = carry_kind
                        .strip_prefix("mul_scaled_by_")
                        .and_then(|prefix| prefix.split_once("_plus_factor_invariant_"))
                    {
                        let (factor_term, terms_part) = prefix;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let factor = resolve_term(factor_term)?;
                        let factor_offset = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let factor = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "add",
                            &factor,
                            factor_offset,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "mul",
                            &source,
                            &factor,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (source, "mul")
                    } else if let Some(prefix) = carry_kind.strip_prefix("mul_scaled_by_") {
                        let (factor_term, terms_part) = prefix.split_once('_').ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let factor = resolve_term(factor_term)?;
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let offset = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "mul",
                            &source,
                            &factor,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (source, "mul")
                    } else if carry_kind.starts_with("mul_scaled_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_scaled_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let factor = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "mul",
                            &source,
                            factor,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (source, "mul")
                    } else if carry_kind.starts_with("mul_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let payload = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                payload,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        (source, "mul")
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = emit_loop_numeric_op(
                        &mut body,
                        &mut next_reg,
                        loop_scalar_kind,
                        op,
                        &current_carries[index],
                        &source,
                    )
                    .map_err(|error| {
                        format!(
                            "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                            node.name,
                        )
                    })?;
                    next_carries.push(reg);
                }
                body.push(format!(
                    "  store {scalar_ty} {next_current}, ptr {current_slot}"
                ));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!(
                        "  store {scalar_ty} {next_carry}, ptr {carry_slot}"
                    ));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {final_current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {final_carry} = load {scalar_ty}, ptr {carry_slot}"
                        ));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                insert_scalar_loop_chain_result(
                    &mut body,
                    &mut next_reg,
                    registers,
                    &node.name,
                    loop_scalar_kind,
                    final_current,
                    final_carries,
                    last_cpu_value,
                );
            }
            ("cpu", "loop_while_i64_async_chain" | "loop_while_scalar_async_chain") => {
                lower_loop_async_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            ("cpu", "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain") => {
                lower_loop_async_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            ("cpu", "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain") => {
                lower_loop_cond_chain!(node, body, registers, next_reg, next_block, last_cpu_value);
            }
            ("cpu", "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain") => {
                lower_loop_flow_chain!(node, body, registers, next_reg, next_block, last_cpu_value);
            }
            ("cpu", "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain") => {
                lower_loop_async_flow_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            (
                "cpu",
                "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain",
            ) => {
                lower_loop_async_flow_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            ("cpu", "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain") => {
                lower_loop_post_flow_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value
                );
            }
            (
                "cpu",
                "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain",
            ) => {
                lower_loop_async_post_flow_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value,
                    helper_signatures
                );
            }
            (
                "cpu",
                "loop_while_i64_async_post_flow_cond_chain"
                | "loop_while_scalar_async_post_flow_cond_chain",
            ) => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let (flow_expr, carry_start_index) =
                    parse_loop_flow_expr_for_llvm(&node.op.args, 4, &node.name, loop_instruction)?;
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable", node.name, callee));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`", node.name, callee));
                    continue;
                }
                let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice", node.name));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64", node.name));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64", node.name));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let Some(resolved_flow_expr) = resolve_loop_flow_expr_for_llvm(
                    &flow_expr,
                    &registers,
                    &mut body,
                    &mut next_reg,
                    &node.name,
                    loop_instruction,
                ) else {
                    continue;
                };
                let carry_source_payload_len = |kind: &str| -> Option<usize> {
                    let carry_state_fragment_is_valid = |fragment: &str| -> bool {
                        match fragment {
                            "current" | "prev_current" => true,
                            other => other
                                .strip_prefix("prev_carry")
                                .or_else(|| other.strip_prefix("carry"))
                                .is_some_and(|index| index.parse::<usize>().is_ok()),
                        }
                    };
                    let add_state_list_payload_len = |kind: &str| -> Option<usize> {
                        let (terms_part, payload_len) = if let Some(prefix) =
                            kind.strip_prefix("add_scaled_by_")
                        {
                            if let Some((lhs_group, rest)) =
                                prefix.split_once("_times_factor_group_")
                            {
                                let parse_group = |group: &str| -> Option<bool> {
                                    if let Some(group) =
                                        group.strip_suffix("_plus_factor_invariant")
                                    {
                                        let terms = group.split("_plus_").collect::<Vec<_>>();
                                        if terms.is_empty()
                                            || !terms
                                                .iter()
                                                .all(|term| carry_state_fragment_is_valid(term))
                                        {
                                            return None;
                                        }
                                        Some(true)
                                    } else {
                                        let terms = group.split("_plus_").collect::<Vec<_>>();
                                        if terms.is_empty()
                                            || !terms
                                                .iter()
                                                .all(|term| carry_state_fragment_is_valid(term))
                                        {
                                            return None;
                                        }
                                        Some(false)
                                    }
                                };
                                let lhs_offset = parse_group(lhs_group)?;
                                if let Some((rhs_group, rest)) =
                                    rest.split_once("_times_factor_invariant_times_terms_")
                                {
                                    let rhs_offset = parse_group(rhs_group)?;
                                    if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                        (
                                            rest,
                                            usize::from(lhs_offset)
                                                + usize::from(rhs_offset)
                                                + 2usize,
                                        )
                                    } else {
                                        (
                                            rest,
                                            usize::from(lhs_offset)
                                                + usize::from(rhs_offset)
                                                + 1usize,
                                        )
                                    }
                                } else {
                                    let (rhs_group, rest) = rest.split_once("_times_terms_")?;
                                    let rhs_offset = parse_group(rhs_group)?;
                                    if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                        (
                                            rest,
                                            usize::from(lhs_offset)
                                                + usize::from(rhs_offset)
                                                + 1usize,
                                        )
                                    } else {
                                        (rest, usize::from(lhs_offset) + usize::from(rhs_offset))
                                    }
                                }
                            } else if let Some((factor_terms, rest)) = prefix
                                .split_once("_plus_factor_invariant_times_factor_invariant_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.is_empty()
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 3usize)
                                } else {
                                    (rest, 2usize)
                                }
                            } else if let Some((factor_terms, rest)) =
                                prefix.split_once("_times_factor_invariant_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.len() < 2
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 2usize)
                                } else {
                                    (rest, 1usize)
                                }
                            } else if let Some((factor_terms, rest)) =
                                prefix.split_once("_plus_factor_invariant_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.is_empty()
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 2usize)
                                } else {
                                    (rest, 1usize)
                                }
                            } else if let Some((factor_terms, rest)) = prefix.split_once("_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.len() < 2
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 1usize)
                                } else {
                                    (rest, 0usize)
                                }
                            } else if let Some((factor, rest)) =
                                prefix.split_once("_plus_factor_invariant_")
                            {
                                if !carry_state_fragment_is_valid(factor) {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 2usize)
                                } else {
                                    (rest, 1usize)
                                }
                            } else if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                let (_, rest) = prefix.split_once('_')?;
                                (rest, 1usize)
                            } else {
                                let (_, rest) = prefix.split_once('_')?;
                                (rest, 0usize)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_scaled_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (prefix, 2usize)
                            } else {
                                (prefix, 1usize)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (prefix, 1usize)
                            } else {
                                (prefix, 0usize)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("mul_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (prefix, 1usize)
                            } else {
                                (prefix, 0usize)
                            }
                        } else {
                            return None;
                        };
                        let terms = terms_part.split("_plus_").collect::<Vec<_>>();
                        if terms.len() < 2 {
                            return None;
                        }
                        if terms.iter().all(|term| carry_state_fragment_is_valid(term)) {
                            Some(payload_len)
                        } else {
                            None
                        }
                    };
                    let zero_payload_indexed_prefixes =
                        ["add_prev_carry", "mul_prev_carry", "add_carry", "mul_carry"];
                    let one_payload_zero_payload_indexed_prefixes =
                        ["add_prev_carry", "add_carry", "mul_prev_carry", "mul_carry"];
                    let one_payload_indexed_prefixes = [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ];
                    if matches!(
                        kind,
                        "keep"
                            | "keep_prev_carry"
                            | "add_current"
                            | "add_prev_current"
                            | "mul_current"
                            | "mul_prev_current"
                    ) || zero_payload_indexed_prefixes.iter().any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        Some(0)
                    } else if one_payload_indexed_prefixes.iter().any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        Some(1)
                    } else if one_payload_zero_payload_indexed_prefixes
                        .iter()
                        .any(|prefix| {
                            kind.strip_prefix(prefix).is_some_and(|suffix| {
                                suffix
                                    .strip_suffix("_plus_invariant")
                                    .is_some_and(|index| index.parse::<usize>().is_ok())
                            })
                        })
                    {
                        Some(1)
                    } else if let Some(payload_len) = add_state_list_payload_len(kind) {
                        Some(payload_len)
                    } else if matches!(
                        kind,
                        "add_read_value_fixed"
                            | "mul_read_value_fixed"
                            | "add_read_value_fixed_plus_invariant"
                            | "mul_read_value_fixed_plus_invariant"
                            | "add_invariant"
                            | "add_current_plus_invariant"
                            | "add_prev_current_plus_invariant"
                            | "mul_invariant"
                            | "mul_current_plus_invariant"
                            | "mul_prev_current_plus_invariant"
                    ) {
                        Some(1)
                    } else if matches!(
                        kind,
                        "add_read_at_fixed"
                            | "mul_read_at_fixed"
                            | "add_read_at_fixed_plus_invariant"
                            | "mul_read_at_fixed_plus_invariant"
                    ) {
                        Some(if kind.ends_with("_plus_invariant") {
                            3
                        } else {
                            2
                        })
                    } else if matches!(
                        kind,
                        "add_read_at_dynamic_current_plus_invariant"
                            | "add_read_at_dynamic_prev_current_plus_invariant"
                            | "mul_read_at_dynamic_current_plus_invariant"
                            | "mul_read_at_dynamic_prev_current_plus_invariant"
                    ) {
                        Some(2)
                    } else if matches!(
                        kind,
                        "add_read_at_dynamic_current"
                            | "add_read_at_dynamic_prev_current"
                            | "mul_read_at_dynamic_current"
                            | "mul_read_at_dynamic_prev_current"
                            | "add_source_plus_invariant"
                            | "mul_source_plus_invariant"
                    ) {
                        Some(1)
                    } else if [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ]
                    .iter()
                    .any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        Some(1)
                    } else if [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ]
                    .iter()
                    .any(|prefix| {
                        kind.strip_prefix(prefix).is_some_and(|suffix| {
                            suffix
                                .strip_suffix("_plus_invariant")
                                .is_some_and(|index| index.parse::<usize>().is_ok())
                        })
                    }) {
                        Some(2)
                    } else {
                        None
                    }
                };
                let mut carry_initials = Vec::new();
                let mut carry_specs =
                    Vec::<(String, Option<String>, Vec<String>, Vec<String>)>::new();
                let mut deferred = false;
                let mut cursor = carry_start_index;
                while cursor < node.op.args.len() {
                    let chunk0 = node.op.args.get(cursor);
                    let chunk1 = node.op.args.get(cursor + 1);
                    let chunk2 = node.op.args.get(cursor + 2);
                    let Some(initial_name) = chunk0 else {
                        break;
                    };
                    let Some(cond_kind_name) = chunk1 else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated carry spec during LLVM lowering", node.name));
                    };
                    let Some(cond_rhs_name) = chunk2 else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated carry spec during LLVM lowering", node.name));
                    };
                    let Some(iv) = registers.get(initial_name).cloned() else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice", node.name));
                        deferred = true;
                        break;
                    };
                    let Some(init) = coerce_to_i64(&iv, &mut body, &mut next_reg) else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64", node.name));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if cond_kind_name == "always" {
                        None
                    } else {
                        let Some(v) = registers.get(cond_rhs_name).cloned() else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice", node.name));
                            deferred = true;
                            break;
                        };
                        let Some(rhs) = coerce_to_i64(&v, &mut body, &mut next_reg) else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64", node.name));
                            deferred = true;
                            break;
                        };
                        Some(rhs)
                    };
                    let then_start = cursor + 3;
                    let Some(then_kind) = node.op.args.get(then_start).cloned() else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated then carry source during LLVM lowering", node.name));
                    };
                    let Some(then_payload_len) = carry_source_payload_len(&then_kind) else {
                        return Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{then_kind}` during LLVM lowering", node.name));
                    };
                    let then_end = then_start + 1 + then_payload_len;
                    if then_end > node.op.args.len() {
                        return Err(format!("cpu.{loop_instruction} `{}` is missing payload for carry kind `{then_kind}` during LLVM lowering", node.name));
                    }
                    let mut then_source = vec![then_kind.clone()];
                    for payload_name in &node.op.args[then_start + 1..then_end] {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is outside the current CPU LLVM slice", node.name));
                            deferred = true;
                            break;
                        };
                        let Some(payload) = coerce_to_i64(&payload_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is not coercible to i64", node.name));
                            deferred = true;
                            break;
                        };
                        then_source.push(payload);
                    }
                    if deferred {
                        break;
                    }
                    let Some(else_kind) = node.op.args.get(then_end).cloned() else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated else carry source during LLVM lowering", node.name));
                    };
                    let Some(else_payload_len) = carry_source_payload_len(&else_kind) else {
                        return Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{else_kind}` during LLVM lowering", node.name));
                    };
                    let else_end = then_end + 1 + else_payload_len;
                    if else_end > node.op.args.len() {
                        return Err(format!("cpu.{loop_instruction} `{}` is missing payload for carry kind `{else_kind}` during LLVM lowering", node.name));
                    }
                    let mut else_source = vec![else_kind.clone()];
                    for payload_name in &node.op.args[then_end + 1..else_end] {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is outside the current CPU LLVM slice", node.name));
                            deferred = true;
                            break;
                        };
                        let Some(payload) = coerce_to_i64(&payload_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is not coercible to i64", node.name));
                            deferred = true;
                            break;
                        };
                        else_source.push(payload);
                    }
                    if deferred {
                        break;
                    }
                    carry_initials.push(init);
                    carry_specs.push((cond_kind_name.clone(), cond_rhs, then_source, else_source));
                    cursor = else_end;
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|init| {
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!("  {s} = alloca i64"));
                        body.push(format!("  store i64 {init}, ptr {s}"));
                        s
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_continue =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred=match cmp_kind { "eq"=>"eq","ne"=>"ne","lt"=>"slt","le"=>"sle","gt"=>"sgt","ge"=>"sge",other=>return Err(format!("cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering", node.name)), };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for slot in &carry_slots {
                    let r = fresh_reg(&mut next_reg);
                    body.push(format!("  {r} = load i64, ptr {slot}"));
                    current_carries.push(r);
                }
                fn resolve_state_term_for_async_post_flow(
                    term: &str,
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
                            let i = other[10..].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering"))?;
                            current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry term `{other}` during LLVM lowering"))
                        }
                        other if other.starts_with("carry") => {
                            let i = other[5..].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering"))?;
                            next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry term `{other}` during LLVM lowering"))
                        }
                        other => Err(format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering")),
                    }
                }
                fn resolve_source_for_async_post_flow(
                    source_spec: &[String],
                    current: &str,
                    next_current: &str,
                    current_carries: &[String],
                    next_carries: &[String],
                    body: &mut Vec<String>,
                    next_reg: &mut usize,
                    node_name: &str,
                    loop_instruction: &str,
                ) -> Result<String, String> {
                    let Some(kind) = source_spec.first() else {
                        return Err(format!("cpu.{loop_instruction} `{node_name}` has empty carry source during LLVM lowering"));
                    };
                    if matches!(kind.as_str(), "keep" | "keep_prev_carry") {
                        return Ok(String::new());
                    }
                    if kind == "add_current" {
                        return Ok(next_current.to_owned());
                    }
                    if let Some(rest) = kind.strip_prefix("add_carry") {
                        let i = rest.parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{kind}` during LLVM lowering"))?;
                        return next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry source `{kind}` during LLVM lowering"));
                    }
                    let parse_factor_group = |group: &str| -> Option<(Vec<String>, bool)> {
                        if let Some(group) = group.strip_suffix("_plus_factor_invariant") {
                            let terms =
                                group.split("_plus_").map(str::to_owned).collect::<Vec<_>>();
                            if terms.is_empty()
                                || !terms.iter().all(|term| {
                                    matches!(term.as_str(), "current" | "prev_current")
                                        || term.starts_with("prev_carry")
                                        || term.starts_with("carry")
                                })
                            {
                                return None;
                            }
                            Some((terms, true))
                        } else {
                            let terms =
                                group.split("_plus_").map(str::to_owned).collect::<Vec<_>>();
                            if terms.is_empty()
                                || !terms.iter().all(|term| {
                                    matches!(term.as_str(), "current" | "prev_current")
                                        || term.starts_with("prev_carry")
                                        || term.starts_with("carry")
                                })
                            {
                                return None;
                            }
                            Some((terms, false))
                        }
                    };
                    if let Some(prefix) = kind.strip_prefix("add_scaled_by_") {
                        if let Some((lhs_group, rest)) = prefix.split_once("_times_factor_group_") {
                            let (lhs_terms, lhs_has_offset) = parse_factor_group(lhs_group)
                                .ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` has unsupported factor group in `{kind}` during LLVM lowering"))?;
                            let (rhs_group, terms_part, has_factor_scale) = if let Some((
                                rhs_group,
                                terms_part,
                            )) =
                                rest.split_once("_times_factor_invariant_times_terms_")
                            {
                                (rhs_group, terms_part, true)
                            } else if let Some((rhs_group, terms_part)) =
                                rest.split_once("_times_terms_")
                            {
                                (rhs_group, terms_part, false)
                            } else {
                                return Err(format!(
                                        "cpu.{loop_instruction} `{node_name}` has malformed factor-group carry kind `{kind}` during LLVM lowering"
                                    ));
                            };
                            let (rhs_terms, rhs_has_offset) = parse_factor_group(rhs_group)
                                .ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` has unsupported factor group in `{kind}` during LLVM lowering"))?;
                            let (terms_part, has_invariant) = if let Some(terms_part) =
                                terms_part.strip_suffix("_plus_invariant")
                            {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                            let terms = terms_part
                                .split("_plus_")
                                .map(str::to_owned)
                                .collect::<Vec<_>>();
                            let mut payload_index = 1usize;
                            let resolve_group =
                                |group_terms: &[String],
                                 has_offset: bool,
                                 payload_index: &mut usize,
                                 body: &mut Vec<String>,
                                 next_reg: &mut usize|
                                 -> Result<String, String> {
                                    let mut acc = resolve_state_term_for_async_post_flow(
                                        &group_terms[0],
                                        current,
                                        next_current,
                                        current_carries,
                                        next_carries,
                                        node_name,
                                        loop_instruction,
                                    )?;
                                    for term in group_terms.iter().skip(1) {
                                        let rhs = resolve_state_term_for_async_post_flow(
                                            term,
                                            current,
                                            next_current,
                                            current_carries,
                                            next_carries,
                                            node_name,
                                            loop_instruction,
                                        )?;
                                        let sum = fresh_reg(next_reg);
                                        body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                                        acc = sum;
                                    }
                                    if has_offset {
                                        let offset_name = source_spec.get(*payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor-group offset payload for `{kind}` during LLVM lowering"))?;
                                        *payload_index += 1;
                                        let sum = fresh_reg(next_reg);
                                        body.push(format!(
                                            "  {sum} = add i64 {acc}, {offset_name}"
                                        ));
                                        acc = sum;
                                    }
                                    Ok(acc)
                                };
                            let lhs = resolve_group(
                                &lhs_terms,
                                lhs_has_offset,
                                &mut payload_index,
                                body,
                                next_reg,
                            )?;
                            let rhs = resolve_group(
                                &rhs_terms,
                                rhs_has_offset,
                                &mut payload_index,
                                body,
                                next_reg,
                            )?;
                            let mut factor = fresh_reg(next_reg);
                            body.push(format!("  {factor} = mul i64 {lhs}, {rhs}"));
                            if has_factor_scale {
                                let factor_scale_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor-scale payload for `{kind}` during LLVM lowering"))?;
                                payload_index += 1;
                                let scaled_factor = fresh_reg(next_reg);
                                body.push(format!(
                                    "  {scaled_factor} = mul i64 {factor}, {factor_scale_name}"
                                ));
                                factor = scaled_factor;
                            }
                            let mut acc = resolve_state_term_for_async_post_flow(
                                &terms[0],
                                current,
                                next_current,
                                current_carries,
                                next_carries,
                                node_name,
                                loop_instruction,
                            )?;
                            for term in terms.iter().skip(1) {
                                let rhs = resolve_state_term_for_async_post_flow(
                                    term,
                                    current,
                                    next_current,
                                    current_carries,
                                    next_carries,
                                    node_name,
                                    loop_instruction,
                                )?;
                                let sum = fresh_reg(next_reg);
                                body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                                acc = sum;
                            }
                            if has_invariant {
                                let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
                                let sum = fresh_reg(next_reg);
                                body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                                acc = sum;
                            }
                            let scaled = fresh_reg(next_reg);
                            body.push(format!("  {scaled} = mul i64 {acc}, {factor}"));
                            return Ok(scaled);
                        }
                    }
                    let parse_add_terms = |kind: &str| -> Option<(
                        Option<Vec<String>>,
                        bool,
                        bool,
                        bool,
                        Vec<String>,
                        bool,
                    )> {
                        let carry_state_fragment_is_valid = |fragment: &str| -> bool {
                            matches!(fragment, "current" | "prev_current")
                                || fragment.starts_with("prev_carry")
                                || fragment.starts_with("carry")
                        };
                        let (
                            factor_term,
                            scaled_by_payload,
                            factor_invariant_payload,
                            factor_scale_payload,
                            terms_part,
                            has_invariant,
                        ) = if let Some(prefix) = kind.strip_prefix("add_scaled_by_") {
                            let (prefix, has_invariant) =
                                if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                    (prefix, true)
                                } else {
                                    (prefix, false)
                                };
                            if let Some((factor_terms, terms_part)) = prefix
                                .split_once("_plus_factor_invariant_times_factor_invariant_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    true,
                                    true,
                                    terms_part,
                                    has_invariant,
                                )
                            } else if let Some((factor_terms, terms_part)) =
                                prefix.split_once("_times_factor_invariant_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    false,
                                    true,
                                    terms_part,
                                    has_invariant,
                                )
                            } else if let Some((factor_terms, terms_part)) =
                                prefix.split_once("_plus_factor_invariant_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    true,
                                    false,
                                    terms_part,
                                    has_invariant,
                                )
                            } else if let Some((factor_terms, terms_part)) =
                                prefix.split_once("_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    false,
                                    false,
                                    terms_part,
                                    has_invariant,
                                )
                            } else {
                                let (factor_term, factor_invariant_payload, terms_part) =
                                    if let Some((factor_term, terms_part)) =
                                        prefix.split_once("_plus_factor_invariant_")
                                    {
                                        (Some(vec![factor_term.to_owned()]), true, terms_part)
                                    } else {
                                        let (factor_term, terms_part) = prefix.split_once('_')?;
                                        (Some(vec![factor_term.to_owned()]), false, terms_part)
                                    };
                                (
                                    factor_term,
                                    false,
                                    factor_invariant_payload,
                                    false,
                                    terms_part,
                                    has_invariant,
                                )
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_scaled_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (None, true, false, false, prefix, true)
                            } else {
                                (None, true, false, false, prefix, false)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (None, false, false, false, prefix, true)
                            } else {
                                (None, false, false, false, prefix, false)
                            }
                        } else {
                            return None;
                        };
                        let terms = terms_part
                            .split("_plus_")
                            .map(|term| term.to_owned())
                            .collect::<Vec<_>>();
                        if terms.iter().all(|term| {
                            matches!(term.as_str(), "current" | "prev_current")
                                || term.starts_with("prev_carry")
                                || term.starts_with("carry")
                        }) {
                            if let Some(factor_terms) = factor_term.as_ref() {
                                if factor_terms.is_empty()
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                            }
                            Some((
                                factor_term,
                                scaled_by_payload,
                                factor_invariant_payload,
                                factor_scale_payload,
                                terms,
                                has_invariant,
                            ))
                        } else {
                            None
                        }
                    };
                    if let Some((
                        factor_term,
                        scaled_by_payload,
                        factor_invariant_payload,
                        factor_scale_payload,
                        terms,
                        has_invariant,
                    )) = parse_add_terms(kind)
                    {
                        let mut payload_index = 1usize;
                        let factor = if let Some(factor_terms) = factor_term {
                            let mut factor = resolve_state_term_for_async_post_flow(
                                &factor_terms[0],
                                current,
                                next_current,
                                current_carries,
                                next_carries,
                                node_name,
                                loop_instruction,
                            )?;
                            for factor_term in factor_terms.iter().skip(1) {
                                let rhs = resolve_state_term_for_async_post_flow(
                                    factor_term,
                                    current,
                                    next_current,
                                    current_carries,
                                    next_carries,
                                    node_name,
                                    loop_instruction,
                                )?;
                                let sum = fresh_reg(next_reg);
                                body.push(format!("  {sum} = add i64 {factor}, {rhs}"));
                                factor = sum;
                            }
                            if factor_scale_payload {
                                let factor_scale_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor scale payload for `{kind}` during LLVM lowering"))?;
                                payload_index += 1;
                                let scaled = fresh_reg(next_reg);
                                body.push(format!(
                                    "  {scaled} = mul i64 {factor}, {factor_scale_name}"
                                ));
                                factor = scaled;
                            }
                            if factor_invariant_payload {
                                let factor_offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor invariant payload for `{kind}` during LLVM lowering"))?;
                                payload_index += 1;
                                let sum = fresh_reg(next_reg);
                                body.push(format!(
                                    "  {sum} = add i64 {factor}, {factor_offset_name}"
                                ));
                                factor = sum;
                            }
                            Some(factor)
                        } else if scaled_by_payload {
                            let factor_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing scaled carry factor for `{kind}` during LLVM lowering"))?;
                            payload_index += 1;
                            Some(factor_name.clone())
                        } else {
                            None
                        };
                        let mut acc = resolve_state_term_for_async_post_flow(
                            &terms[0],
                            current,
                            next_current,
                            current_carries,
                            next_carries,
                            node_name,
                            loop_instruction,
                        )?;
                        for term in terms.iter().skip(1) {
                            let rhs = resolve_state_term_for_async_post_flow(
                                term,
                                current,
                                next_current,
                                current_carries,
                                next_carries,
                                node_name,
                                loop_instruction,
                            )?;
                            let sum = fresh_reg(next_reg);
                            body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                            acc = sum;
                        }
                        if factor.is_some() && has_invariant {
                            let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
                            payload_index += 1;
                            let sum = fresh_reg(next_reg);
                            body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                            acc = sum;
                        }
                        if let Some(factor) = factor {
                            let scaled_reg = fresh_reg(next_reg);
                            body.push(format!("  {scaled_reg} = mul i64 {acc}, {factor}"));
                            acc = scaled_reg;
                        } else if has_invariant {
                            let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
                            let sum = fresh_reg(next_reg);
                            body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                            acc = sum;
                        }
                        return Ok(acc);
                    }
                    Err(format!("cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{kind}` during LLVM lowering"))
                }
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_source_spec, else_source_spec)) in
                    carry_specs.iter().enumerate()
                {
                    let then_value =
                        if matches!(then_source_spec[0].as_str(), "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let src = resolve_source_for_async_post_flow(
                                then_source_spec,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &mut body,
                                &mut next_reg,
                                &node.name,
                                loop_instruction,
                            )?;
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        };
                    let else_value =
                        if matches!(else_source_spec[0].as_str(), "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let src = resolve_source_for_async_post_flow(
                                else_source_spec,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &mut body,
                                &mut next_reg,
                                &node.name,
                                loop_instruction,
                            )?;
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let rhs=cond_rhs.clone().ok_or_else(|| format!("cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering", node.name))?;
                        let (lhs,pred)=match cond_kind.as_str() {
                            "current_eq"=>(next_current.clone(),"eq"), "current_ne"=>(next_current.clone(),"ne"), "current_lt"=>(next_current.clone(),"slt"), "current_le"=>(next_current.clone(),"sle"), "current_gt"=>(next_current.clone(),"sgt"), "current_ge"=>(next_current.clone(),"sge"),
                            "prev_current_eq"=>(current.clone(),"eq"), "prev_current_ne"=>(current.clone(),"ne"), "prev_current_lt"=>(current.clone(),"slt"), "prev_current_le"=>(current.clone(),"sle"), "prev_current_gt"=>(current.clone(),"sgt"), "prev_current_ge"=>(current.clone(),"sge"),
                            other if other.starts_with("prev_carry") && other.ends_with("_eq") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ne") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("prev_carry") && other.ends_with("_lt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_le") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("prev_carry") && other.ends_with("_gt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ge") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other if other.starts_with("carry") && other.ends_with("_eq") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("carry") && other.ends_with("_ne") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("carry") && other.ends_with("_lt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("carry") && other.ends_with("_le") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("carry") && other.ends_with("_gt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("carry") && other.ends_with("_ge") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other => return Err(format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name)),
                        };
                        let c = fresh_reg(&mut next_reg);
                        body.push(format!("  {c} = icmp {pred} i64 {lhs}, {rhs}"));
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {s} = select i1 {c}, i64 {then_value}, i64 {else_value}"
                        ));
                        s
                    };
                    next_carries.push(next_carry);
                }
                let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
                collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
                let condition_blocks = (0..flow_leaves.len())
                    .map(|index| {
                        if index == 0 {
                            None
                        } else {
                            Some(fresh_block(&mut next_block, "loop_async_post_flow_rhs"))
                        }
                    })
                    .collect::<Vec<_>>();
                for (index, (condition, action)) in flow_leaves.iter().enumerate() {
                    if let Some(block) = &condition_blocks[index] {
                        body.push(format!("{block}:"));
                    }
                    let no_match_block = condition_blocks
                        .get(index + 1)
                        .and_then(|block| block.clone())
                        .unwrap_or_else(|| loop_continue.clone());
                    let control_cond = emit_loop_flow_control_expr(
                        condition,
                        &next_current,
                        &next_carries,
                        &mut body,
                        &mut next_reg,
                        &node.name,
                        loop_instruction,
                    )?;
                    let action_block = fresh_block(&mut next_block, "loop_async_post_flow_action");
                    body.push(format!(
                        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
                    ));
                    body.push(format!("{action_block}:"));
                    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                    for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
                        body.push(format!("  store i64 {val}, ptr {slot}"));
                    }
                    match *action {
                        "break" => body.push(format!("  br label %{loop_exit}")),
                        "continue" => body.push(format!("  br label %{loop_cond}")),
                        other => {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported flow action `{other}` during LLVM lowering",
                                node.name,
                            ));
                        }
                    }
                }
                body.push(format!("{loop_continue}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {val}, ptr {slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|slot| {
                        let r = fresh_reg(&mut next_reg);
                        body.push(format!("  {r} = load i64, ptr {slot}"));
                        r
                    })
                    .collect::<Vec<_>>();
                insert_i64_loop_chain_result(
                    registers,
                    &node.name,
                    final_current,
                    final_carries,
                    last_cpu_value,
                );
            }
            (
                "cpu",
                "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain",
            ) => {
                lower_loop_post_flow_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value
                );
            }
            ("cpu", "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain") => {
                lower_loop_flow_cond_chain!(
                    node,
                    body,
                    registers,
                    next_reg,
                    next_block,
                    last_cpu_value
                );
            }
            _ => {
                body.push(format!(
                    "  ; deferred lowering for {} on {} ({})",
                    node.op.full_name(),
                    node.resource,
                    resource.kind.raw
                ));
            }
        }
    }

    *global_counter = state.next_global;
    let ret = state.last_cpu_value.unwrap_or_else(|| "0".to_owned());
    let body = if state.ends_with_terminal_return {
        state.body.join("\n")
    } else {
        let mut body = state.body;
        emit_typed_return_from_last_value(
            &mut body,
            &mut state.next_reg,
            function_return_kind,
            &ret,
        );
        body.join("\n")
    };

    Ok(EmittedCpuFunction {
        globals: state.globals,
        body,
    })
}
